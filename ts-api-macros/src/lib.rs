extern crate proc_macro;

use darling::{util::SpannedValue, FromMeta};
use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{parse2, Attribute, Error, FnArg, Ident, ItemFn, NestedMeta, Result, ReturnType, Token};
use ts_api_core::{ApiMethod, ApiRequest};

#[proc_macro_attribute]
pub fn api(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match api_inner(TokenStream::from(attr), TokenStream::from(input)) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn api_inner(attr: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let poem_crate = get_crate_name("poem", false);
    let self_crate = get_crate_name("ts_api", false);

    let item_fn = parse2::<ItemFn>(input)?;
    let fn_name = &item_fn.sig.ident;
    println!("{:?}", fn_name.to_string());

    // Check if the function is async
    if item_fn.sig.asyncness.is_none() {
        return Err(Error::new_spanned(&item_fn.sig.ident, "Must be async"));
    }

    let attr: V<NestedMeta> = parse2(attr)?;
    let api_attrs = ApiAttributes::from_list(&attr.0)?;
    let method = api_attrs.method_tokens();
    let path = api_attrs.path_tokens();

    let mut request_types = Vec::new();
    for input in item_fn.sig.inputs.iter() {
        match input {
            FnArg::Typed(input) => {
                request_types.push(&input.ty);
            }
            FnArg::Receiver(_) => {
                return Err(Error::new_spanned(
                    input,
                    "Function argument must not be self",
                ))
            }
        }
    }

    let register_response_type = match &item_fn.sig.output {
        ReturnType::Default => quote!(),
        ReturnType::Type(_, ty) => quote!(request.register_response_type::<#ty>();),
    };

    Ok(quote! {
        #[#poem_crate::handler]
        #item_fn

        impl #self_crate::ApiHandler for #fn_name {
            const METHOD: #self_crate::ApiMethod = #method;
            const PATH: &'static str = #path;

            fn typescript(server_url: impl AsRef<str>) -> String {
                let mut request = #self_crate::ApiRequest::new();
                #(request.register_param::<#request_types>();)*
                #register_response_type
                request.finish(server_url, Self::METHOD.as_str(), Self::PATH)
            }
        }
    })
}

#[test]
fn test_macro() {
    let attrs = quote!(method = "get", path = "/");
    let input = quote! {
        async fn a(b: Json<String>) -> Json<u32> {
            Json(0)
        }
    };

    let output = api_inner(attrs, input).unwrap();
    println!("{}", rustfmt(output.to_string()));
}

#[test]
fn test_generic_ts() {
    use ts_rs::TS;

    #[derive(ts_rs::TS)]
    struct A {
        a: B,
    }

    #[derive(ts_rs::TS)]
    struct B {
        b: c::C,
    }

    mod c {
        #[derive(ts_rs::TS)]
        pub struct C {
            c: u16,
        }
    }

    use poem::web::Json;

    let mut request = ApiRequest::new();
    request.register_param::<Json<(B, A)>>();
    request.register_param::<poem::web::Path<c::C>>();
    request.register_response_type::<Json<c::C>>();
    let request = request.finish("http://localhost:3000", "POST", "/backend/a");
    println!("{}", request);
}

#[derive(Debug, Clone, FromMeta)]
struct ApiAttributes {
    method: SpannedValue<ApiMethod>,
    path: SpannedValue<String>,
}

impl ApiAttributes {
    fn method_tokens(&self) -> TokenStream {
        let self_crate = get_crate_name("ts_api", false);
        match self.method.as_ref() {
            ApiMethod::Get => quote!(#self_crate::ApiMethod::Get),
            ApiMethod::Post => quote!(#self_crate::ApiMethod::Post),
            ApiMethod::Put => quote!(#self_crate::ApiMethod::Put),
            ApiMethod::Delete => quote!(#self_crate::ApiMethod::Delete),
            ApiMethod::Head => quote!(#self_crate::ApiMethod::Head),
            ApiMethod::Options => quote!(#self_crate::ApiMethod::Options),
            ApiMethod::Connect => quote!(#self_crate::ApiMethod::Connect),
            ApiMethod::Patch => quote!(#self_crate::ApiMethod::Patch),
            ApiMethod::Trace => quote!(#self_crate::ApiMethod::Trace),
        }
    }

    fn path_tokens(&self) -> TokenStream {
        let path = match &self.path.as_ref() {
            s if s.starts_with('/') => quote!(#s),
            s => quote!("/" #s),
        };
        quote!(#path)
    }
}

// Not sure why Parse isn't implemented for Vec<T> where T: Parse
#[derive(Debug)]
struct V<T>(Vec<T>);
impl<T> syn::parse::Parse for V<T>
where
    T: syn::parse::Parse,
{
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut v = Vec::new();
        while !input.is_empty() {
            v.push(input.parse()?);
            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }
        Ok(V(v))
    }
}

#[test]
fn test_api() {
    let attr = quote! {
        method = "get", path = "/"
    };
    let input = quote! {
        async fn a(b: Json<a::B>) -> Json<C> {}
    };

    let output = api_inner(attr, input).unwrap();
    println!("{}", rustfmt(quote!(#output).to_string()));
}

fn rustfmt(s: impl AsRef<[u8]>) -> String {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new("rustfmt")
        .args(["--edition", "2021"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(s.as_ref()).unwrap();
    }

    String::from_utf8(child.wait_with_output().unwrap().stdout).unwrap()
}

pub(crate) fn get_crate_name(name: &str, internal: bool) -> TokenStream {
    if internal {
        quote! { crate }
    } else {
        let name = match crate_name(name) {
            Ok(FoundCrate::Name(name)) => name,
            Ok(FoundCrate::Itself) | Err(_) => name.to_string(),
        };
        let name = Ident::new(&name, Span::call_site());
        quote!(#name)
    }
}
