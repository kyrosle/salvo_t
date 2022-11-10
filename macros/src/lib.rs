mod extract;
mod handler;
mod shared;

use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs, DeriveInput, Item};

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
            #[salvo_t::async_trait]
            impl salvo_t::Handler for hello {
                async fn handle(
                    &self,
                    req: &mut salvo_t::Request,
                    depot: &mut salvo_t::Depot,
                    res: &mut salvo_t::Response,
                    ctrl: &mut salvo_t::routing::FlowCtrl
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
            #[salvo_t::async_trait]
            impl salvo_t::Handler for hello {
                async fn handle(
                    &self,
                    req: &mut salvo_t::Request,
                    depot: &mut salvo_t::Depot,
                    res: &mut salvo_t::Response,
                    ctrl: &mut salvo_t::routing::FlowCtrl
                ) {
                    salvo_t::Writer::write(Self::hello(req, depot, res, ctrl).await, req, depot, res).await;
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
            #[salvo_t::async_trait]
            impl salvo_t::Handler for Hello {
                async fn handle(
                    &self,
                    req: &mut salvo_t::Request,
                    depot: &mut salvo_t::Depot,
                    res: &mut salvo_t::Response,
                    ctrl: &mut salvo_t::routing::FlowCtrl
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
            static __salvo_extract_BadMan: salvo_t::__private::once_cell::sync::Lazy<salvo_t::extract::Metadata> = salvo_t::__private::once_cell::sync::Lazy::new(|| {
                let mut metadata = salvo_t::extract::Metadata::new("BadMan");
                metadata = metadata.add_default_source(salvo_t::extract::metadata::Source::new(
                    salvo_t::extract::metadata::SourceFrom::Body,
                    salvo_t::extract::metadata::SourceFormat::MultiMap
                ));
                let mut field = salvo_t::extract::metadata::Field::new("id");
                field = field.add_source(salvo_t::extract::metadata::Source::new(
                    salvo_t::extract::metadata::SourceFrom::Query,
                    salvo_t::extract::metadata::SourceFormat::MultiMap
                ));
                metadata = metadata.add_field(field);
                let mut field = salvo_t::extract::metadata::Field::new("username");
                metadata = metadata.add_field(field);
                metadata
            });
            impl<'a> salvo_t::extract::Extractible<'a> for BadMan<'a> {
                fn metadata() -> &'static salvo_t::extract::Metadata {
                    &*__salvo_extract_BadMan
                }
            }
        };

        assert_eq!(
            extract::generate(item).unwrap().to_string(),
            right.to_string()
        );
    }

    #[test]
    fn test_extract_lifetime() {
        let input = quote! {
            #[extract(
                default_source(from = "query"),
                default_source(from = "param"),
                default_source(from = "body"),
            )]
            struct BadMan<'a> {
                id: i64,
                username: String,
                first_name: &'a str,
                last_name: String,
                lovers: Vec<String>
            }
        };
        let item = parse2(input).unwrap();
        let right = quote! {
            #[allow(non_upper_case_globals)]
            static __salvo_extract_BadMan:
            salvo_t::__private::once_cell::sync::Lazy<salvo_t::extract::Metadata> =
            salvo_t::__private::once_cell::sync::Lazy::new(|| {
                let mut metadata = salvo_t::extract::Metadata::new("BadMan");
                metadata = metadata.add_default_source(salvo_t::extract::metadata::Source::new(
                    salvo_t::extract::metadata::SourceFrom::Query,
                    salvo_t::extract::metadata::SourceFormat::MultiMap
                ));
                metadata = metadata.add_default_source(salvo_t::extract::metadata::Source::new(
                    salvo_t::extract::metadata::SourceFrom::Param,
                    salvo_t::extract::metadata::SourceFormat::MultiMap
                ));
                metadata = metadata.add_default_source(salvo_t::extract::metadata::Source::new(
                    salvo_t::extract::metadata::SourceFrom::Body,
                    salvo_t::extract::metadata::SourceFormat::MultiMap
                ));

                let mut field = salvo_t::extract::metadata::Field::new("id");
                metadata = metadata.add_field(field);
                let mut field = salvo_t::extract::metadata::Field::new("username");
                metadata = metadata.add_field(field);
                let mut field = salvo_t::extract::metadata::Field::new("first_name");
                metadata = metadata.add_field(field);
                let mut field = salvo_t::extract::metadata::Field::new("last_name");
                metadata = metadata.add_field(field);
                let mut field = salvo_t::extract::metadata::Field::new("lovers");
                metadata = metadata.add_field(field);
                metadata
            });
            impl<'a> salvo_t::extract::Extractible<'a> for BadMan<'a> {
                fn metadata() -> &'static salvo_t::extract::Metadata {
                    &*__salvo_extract_BadMan
                }
            }
        };

        assert_eq!(
            extract::generate(item).unwrap().to_string(),
            right.to_string()
        );
    }
}
