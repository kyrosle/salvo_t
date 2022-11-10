use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{ImplItem, Item, Pat, Signature, Type, ReturnType};

use crate::shared::*;

pub(crate) fn generate(internal: bool, input: Item) -> syn::Result<TokenStream> {
    let salvo_t = salvo_crate(internal);
    match input {
        Item::Fn(mut item_fn) => {
            let attrs = &item_fn.attrs;
            let vis = &item_fn.vis;
            let sig = &mut item_fn.sig;
            let body = &item_fn.block;
            let name = &sig.ident;
            let docs = item_fn
                .attrs
                .iter()
                .filter(|attr| attr.path.is_ident("doc"))
                .cloned()
                .collect::<Vec<_>>();

            let sdef = quote! {
                #(#docs)*
                #[allow(non_camel_case_types)]
                #[derive(Debug)]
                #vis struct #name;
                impl #name {
                    #(#attrs)*
                    #sig {
                        #body
                    }
                }
            };

            let hfn = handle_fn(&salvo_t, sig)?;
            Ok(quote! {
                #sdef
                #[#salvo_t::async_trait]
                impl #salvo_t::Handler for #name {
                    #hfn
                }
            })
        }
        Item::Impl(item_impl) => {
            let mut hmtd = None;
            for item in &item_impl.items {
                if let ImplItem::Method(method) = item {
                    if method.sig.ident == Ident::new("handle", Span::call_site()) {
                        hmtd = Some(method);
                    }
                }
            }
            if hmtd.is_none() {
                return Err(syn::Error::new_spanned(
                    item_impl.impl_token,
                    "missing handle function",
                ));
            }
            let hmtd = hmtd.unwrap();
            let hfn = handle_fn(&salvo_t, &hmtd.sig)?;
            let ty = &item_impl.self_ty;
            let (impl_generics, ty_generics, where_clause) = &item_impl.generics.split_for_impl();

            Ok(quote! {
                #item_impl
                #[#salvo_t::async_trait]
                impl #impl_generics #salvo_t::Handler for #ty #ty_generics #where_clause {
                    #hfn
                }
            })
        }
        _ => Err(syn::Error::new_spanned(
            input,
            "#[handler] must added to `impl` or `fn`",
        )),
    }
}

fn handle_fn(salvo_t: &Ident, sig: &Signature) -> syn::Result<TokenStream> {
    let name = &sig.ident;
    let mut extract_ts = Vec::with_capacity(sig.inputs.len());
    let mut call_args: Vec<Ident> = Vec::with_capacity(sig.inputs.len());
    for input in &sig.inputs {
        match parse_input_type(input) {
            InputType::Request(_pat) => {
                call_args.push(Ident::new("req", Span::call_site()));
            }
            InputType::Depot(_pat) => {
                call_args.push(Ident::new("depot", Span::call_site()));
            }
            InputType::Response(_pat) => {
                call_args.push(Ident::new("res", Span::call_site()));
            }
            InputType::FlowCtrl(_pat) => {
                call_args.push(Ident::new("ctrl", Span::call_site()));
            }
            InputType::Unknown => {
                return Err(syn::Error::new_spanned(
                    &sig.inputs,
                    "the inputs parameters must be Request, Depot, Response or FlowCtrl",
                ))
            }
            InputType::NoReference(pat) => {
                if let (Pat::Ident(ident), Type::Path(ty)) = (&*pat.pat, &*pat.ty) {
                    call_args.push(ident.ident.clone());
                    let id = &pat.pat;
                    let ty = omit_type_path_lifetimes(ty);

                    extract_ts.push(quote!{
                        let #id: #ty = match req.extract().await {
                            Ok(data) => data,
                            Err(e) => {
                                #salvo_t::__private::tracing::error!(error = ?e, "failed to extract data");
                                res.set_status_error(#salvo_t::http::errors::StatusError::bad_request().with_detail(
                                    "Extract data failed"
                                ));
                                return;
                            }
                        };
                    });
                } else {
                    return Err(syn::Error::new_spanned(pat, "Invalid param definition"));
                }
            }
            InputType::LazyExtract(pat) => {
                if let (Pat::Ident(ident), Type::Path(ty)) = (&*pat.pat, &*pat.ty) {
                    call_args.push(ident.ident.clone());

                    let id = &pat.pat;
                    let ty = omit_type_path_lifetimes(ty);

                    extract_ts.push(quote! {
                        let #id: #ty = #salvo_t::extract::LazyExtract::new();
                    });
                } else {
                    return Err(syn::Error::new_spanned(pat, "Invalid param definition"));
                }
            }
            InputType::Receiver(_) => {
                call_args.push(Ident::new("self", Span::call_site()));
            }
        }
    }

    match sig.output {
        ReturnType::Default => {
            if sig.asyncness.is_none() {
                Ok(quote!{
                    async fn handle(&self, req: &mut #salvo_t::Request, depot: &mut #salvo_t::Depot, res: &mut #salvo_t::Response, ctrl: &mut #salvo_t::routing::FlowCtrl) {
                        #(#extract_ts)*
                        Self::#name(#(#call_args),*)
                    } 
                })
            } else {
                Ok(quote!{
                    async fn handle(&self, req: &mut #salvo_t::Request, depot: &mut #salvo_t::Depot, res: &mut #salvo_t::Response, ctrl: &mut #salvo_t::routing::FlowCtrl) {
                        #(#extract_ts)*
                        Self::#name(#(#call_args),*).await
                    } 
                })
            }
        }
        ReturnType::Type(_,_ ) => {
            if sig.asyncness.is_none() {
                Ok(quote!{
                    async fn handle(&self, req: &mut #salvo_t::Request, depot: &mut #salvo_t::Depot, res: &mut #salvo_t::Response, ctrl: &mut #salvo_t::routing::FlowCtrl) {
                        #(#extract_ts)*
                        #salvo_t::Writer::write(Self::#name(#(#call_args),*), req, depot, res).await;
                    } 
                })
            } else {
                Ok(quote!{
                    async fn handle(&self, req: &mut #salvo_t::Request, depot: &mut #salvo_t::Depot, res: &mut #salvo_t::Response, ctrl: &mut #salvo_t::routing::FlowCtrl) {
                        #(#extract_ts)*
                        #salvo_t::Writer::write(Self::#name(#(#call_args),*).await, req, depot, res).await;
                    } 
                })
            }
        }
    }
}
