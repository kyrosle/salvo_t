mod extract;
mod handler;
mod shared;

use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs, Item, DeriveInput};

#[proc_macro_attribute]
pub fn handler(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let internal = shared::is_internal(args.iter());
    let item = parse_macro_input!(input as Item);
    match handler::generate(internal, item) {
        Ok(stream) => stream.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(Extractible, attributes(extract))]
pub fn derive_extractible(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as DeriveInput);
    match extract::generate(args) {
        Ok(stream) => stream.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[cfg(test)]
mod tests {

    use quote::quote;
    use syn::parse2;

    use super::*;

    #[test]
    fn test_handler_for_fn() {
        let input = quote! {
            #[handler]
            async fn hello(req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
                res.render_plan_text("Hello world");
            }
        };

        let item = parse2(input).unwrap();

        let right = quote! {
            #[allow(non_camel_case_types)]
            #[derive(Debug)]
            struct hello;
            impl hello {
                #[handler]
                async fn hello(req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
                    {
                        res.render_plan_text("Hello world");
                    }
                }
            }
            #[salvo::async_trait]
            impl salvo::Handler for hello {
                async fn handle(
                    &self,
                    req: &mut salvo::Request,
                    depot: &mut salvo::Depot,
                    res: &mut salvo::Response,
                    ctrl: &mut salvo::routing::FlowCtrl
                ) {
                    Self::hello(req, depot, res, ctrl).await
                }
            }
        };

        assert_eq!(
            handler::generate(false, item).unwrap().to_string(),
            right.to_string()
        );
    }

    #[test]
    fn test_handler_for_fn_return_result() {
        let input = quote! {
            #[handler]
            async fn hello(req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) -> Result<(), Error> {
                Ok(())
            }
        };

        let item = parse2(input).unwrap();

        let right = quote! {
            #[allow(non_camel_case_types)]
            #[derive(Debug)]
            struct hello;
            impl hello {
                #[handler]
                async fn hello(
                    req: &mut Request,
                    depot: &mut Depot,
                    res: &mut Response,
                    ctrl: &mut FlowCtrl
                ) -> Result<(), Error> {
                    {
                        Ok(())
                    }
                }
            }
            #[salvo::async_trait]
            impl salvo::Handler for hello {
                async fn handle(
                    &self,
                    req: &mut salvo::Request,
                    depot: &mut salvo::Depot,
                    res: &mut salvo::Response,
                    ctrl: &mut salvo::routing::FlowCtrl
                ) {
                    salvo::Writer::write(Self::hello(req, depot, res, ctrl).await, req, depot, res).await;
                }
            }
        };
        assert_eq!(
            handler::generate(false, item).unwrap().to_string(),
            right.to_string()
        );
    }

    #[test]
    fn test_handler_for_impl() {
        let input = quote! {
            #[handler]
            impl Hello {
                fn handle(req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
                    res.render_plan_text("Hello World");
                }
            }
        };
        let item = parse2(input).unwrap();

        let right = quote! {
            #[handler]
            impl Hello {
                fn handle(req: &mut Request, depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
                    res.render_plan_text("Hello World");
                }
            }
            #[salvo::async_trait]
            impl salvo::Handler for Hello {
                async fn handle(
                    &self,
                    req: &mut salvo::Request,
                    depot: &mut salvo::Depot,
                    res: &mut salvo::Response,
                    ctrl: &mut salvo::routing::FlowCtrl
                ) {
                    Self::handle(req, depot, res, ctrl)
                }
            }
        };

        assert_eq!(
            handler::generate(false, item).unwrap().to_string(),
            right.to_string()
        );
    }
    #[test]
    fn test_extract_simple() {
        let input = quote! {
            #[extract(default_source(from = "body"))]
            struct BadMan<'a> {
                #[extract(source(from = "query"))]
                id: i64,
                username: String,
            }
        };

        let item = parse2(input).unwrap();

        let right = quote! { 
            #[allow(non_upper_case_globals)]
            static __salvo_extract_BadMan: salvo::__private::once_cell::sync::Lazy<salvo::extract::Metadata> = salvo::__private::once_cell::sync::Lazy::new(|| {
                let mut metadata = salvo::extract::Metadata::new("BadMan");
                metadata = metadata.add_default_source(salvo::extract::metadata::Source::new(
                    salvo::extract::metadata::SourceFrom::Body,
                    salvo::extract::metadata::SourceFormat::MultiMap
                ));
                let mut field = salvo::extract::metadata::Field::new("id");
                field = field.add_source(salvo::extract::metadata::Source::new(
                    salvo::extract::metadata::SourceFrom::Query,
                    salvo::extract::metadata::SourceFormat::MultiMap
                ));
                metadata = metadata.add_field(field);
                let mut field = salvo::extract::metadata::Field::new("username");
                metadata = metadata.add_field(field);
                metadata
            });
            impl<'a> salvo::extract::Extractible<'a> for BadMan<'a> {
                fn metadata() -> &'static salvo::extract::Metadata {
                    &*__salvo_extract_BadMan
                }
            }
        };

        assert_eq!(extract::generate(item).unwrap().to_string(), right.to_string());
    }
}
