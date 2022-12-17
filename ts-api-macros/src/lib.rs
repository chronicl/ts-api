extern crate proc_macro;

use std::marker::PhantomData;

use darling::{util::SpannedValue, FromMeta};
use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{ quote};
use syn::{ NestedMeta, Token, parse2,  Attribute, FnArg, Ident, ItemFn, Error, Result, ReturnType};
use ts_api_core::ApiMethod;

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
    println!("{:#?}", input);
    let poem_crate = get_crate_name("poem", false);
    let self_crate = get_crate_name("ts_api", false);

    let item_fn = parse2::<ItemFn>(input)?;
    println!("{:#?}", item_fn);

    // Check if the function is async
    if item_fn.sig.asyncness.is_none() {
        return Err(Error::new_spanned(&item_fn.sig.ident, "Must be async"));
    }

    let attr: V<NestedMeta> = parse2(attr)?;
    let api_attrs = ApiAttributes::from_list(&attr.0)?;

    fn option_string_to_tokens(option: Option<String>) -> TokenStream {
        match option {
            Some(s) => {
                let ident = Ident::new(&s, Span::call_site());
                quote!(#ident)
            }
            None => quote!(()),
        }
    }
    let request_type = option_string_to_tokens(parse_request_type(&item_fn)?);

    let response_type = match &item_fn.sig.output {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => Some(parse_type_inside_json(ty).ok_or_else(|| {
            Error::new_spanned(ty, "Response type must be Json<...>")
        })?),
    };
    let response_type = option_string_to_tokens(response_type);

    Ok(quote! {
        #[#poem_crate::handler]
        #item_fn

        impl #self_crate::ApiHandler for a {
            type Request = #request_type;
            type Response = #response_type;

            const API: #self_crate::ApiRoute = #self_crate::ApiRoute {
                #api_attrs,
            };
        }
    })
}

// Todo: This should instead be done with extractors
fn parse_request_type(item_fn: &ItemFn) -> Result<Option<String>> {
    let Some(first_input) = item_fn.sig.inputs.first() else { return Ok(None) };
    let FnArg::Typed(first_input) = first_input else { 
        return Err(Error::new_spanned(
            first_input,
            "First argument must not be self",
        )) 
    };
    Ok(parse_type_inside_json(&first_input.ty))
}

fn parse_type_inside_json(path: &syn::Type) -> Option<String> {
    let syn::Type::Path(path) = path else { return None };

    let mut type_name = None;
    if let Some(json) = path.path.segments.iter().find(|s| s.ident == "Json") {
        if let syn::PathArguments::AngleBracketed( data) = &json.arguments {
            let ty = &data.args[0];
            type_name = Some(quote!(#ty).to_string());
        }
    }

    type_name
}

#[test]
fn test_type() {
    #[derive(ts_rs::TS)]
    struct A<T> {
        a: T
    }

    mod c {
        #[derive(ts_rs::TS)]
        pub struct B {
            b: String
        }
    }

    let a = { use ts_rs :: TS as __TS ; < A < c :: B > as __TS > :: name_with_type_args (vec ! [< c :: B as __TS > :: name ()]) };

    let ty = quote!(A<c::B>);
    let ty = parse2::<syn::Type>(ty).unwrap();
    let ty = Type::from_syn(&ty).unwrap();
    println!("{}", a);
}

#[derive(Debug)]
struct Type {
    ident: Ident,
    path: syn::TypePath,
    generics: Vec<Type>,
}

#[test]
fn test_enum_ts() {
    use ts_rs::TS;
    #[derive(ts_rs::TS)]
    enum A<K> {
        A(F),
        C(K),
    }

    #[derive(ts_rs::TS)]
    struct K {
        a: u16
    }

    #[derive(ts_rs::TS)]
    struct F(K);

    // Some weird behaviour that should be fixed
    println!("{:#?}", A::<std::result::Result<u32, String>>::dependencies());
    println!("{:#?}", F::dependencies());
}

#[test]
fn test_generic_ts() {
    use ts_rs::TS;

    #[derive(ts_rs::TS)]
    struct A {
        a: B
    }

    #[derive(ts_rs::TS)]
    struct B {
        b: c::C
    }

    mod c {
        #[derive(ts_rs::TS)]
        pub struct C {
            c: u16
        }
    }

    use poem::web::Json;

    let mut deps = ts_rs::Dependencies::new();
    deps.add::<Json<(B, A)>>();
    let export = deps.values().map(|d| format!("{}\n\n", d.ts_type)).collect::<String>();
    
    println!("{}", export);
}




impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ident)?;
        if !self.generics.is_empty() {
            write!(f, "<")?;
            for (i, ty) in self.generics.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{ty}")?;
            }
            write!(f, ">")?;
        }
        Ok(())
    }
}

impl Type {
    fn typescript_type(&self) -> TokenStream {
        let inner = self.typescript_type_inner();
        let tsrs_crate  = get_crate_name("ts_rs", false);

        quote! {
            {
                use #tsrs_crate::TS as __TS;

                #inner
            }
        }
    }

    fn typescript_type_inner(&self) -> TokenStream {
        let path = &self.path;
       
        if self.generics.is_empty() {
            quote! {
                <#path as __TS>::name()
            }
        } else {
            let generics = self.generics.iter().map(|ty| ty.typescript_type_inner());
            quote! {
                <#path as __TS>::name_with_type_args(vec![#(#generics),*])
            }
        }
    }

    fn from_syn(ty: &syn::Type) -> Result<Self> {
        let syn::Type::Path(path) = ty else { 
            return Err(Error::new_spanned(ty, "Only syn::Type::Path types are supported.")) 
        };

        let Some(ty) = path.path.segments.last() else {
            return Err(Error::new_spanned(ty, "Only types with ident names are supported."))
        };
        
        let mut generics = Vec::new();
        if let syn::PathArguments::AngleBracketed(data) = &ty.arguments {
            for arg in &data.args {
                let syn::GenericArgument::Type(arg) = arg else {
                    return Err(Error::new_spanned(arg, "Only types are supported as generics."))
                };
                generics.push(Self::from_syn(arg)?);
            }
        }

        Ok(Self {
            ident: ty.ident.clone(),
            path: path.clone(),
            generics,
        })
    }
}

fn _remove_attr(attrs: &mut Vec<Attribute>, attr: &str) -> Option<Attribute> {
    attrs.iter().position(|a| a.path.is_ident(attr)).map(|i| attrs.remove(i)) 
}

#[derive(Debug, Clone, FromMeta)]
struct ApiAttributes {
    method: SpannedValue<ApiMethod>,
    path: SpannedValue<String>,
}

impl quote::ToTokens for ApiAttributes {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let self_crate = get_crate_name("ts_api", false);
        let method = match self.method.as_ref() {
            ApiMethod::Get => quote!(#self_crate::ApiMethod::Get),
            ApiMethod::Post => quote!(#self_crate::ApiMethod::Post),
            ApiMethod::Put => quote!(#self_crate::ApiMethod::Put),
            ApiMethod::Delete => quote!(#self_crate::ApiMethod::Delete),
            ApiMethod::Head => quote!(#self_crate::ApiMethod::Head),
            ApiMethod::Options => quote!(#self_crate::ApiMethod::Options),
            ApiMethod::Connect => quote!(#self_crate::ApiMethod::Connect),
            ApiMethod::Patch => quote!(#self_crate::ApiMethod::Patch),
            ApiMethod::Trace => quote!(#self_crate::ApiMethod::Trace),
        };
        let path = match &self.path.as_ref() {
            s if s.starts_with('/') => quote!(#s),
            s => quote!("/" #s),
        };
        tokens.extend(quote! {
                method: #method,
                path: #path
        });
    }
}

// Not sure why Parse isn't implemented for Vec<T> where T: Parse
#[derive(Debug)]
struct V<T>(Vec<T>);
impl<T> syn::parse::Parse for V<T> where
    T: syn::parse::Parse
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
    println!("{}", _rustfmt(quote!(#output).to_string()));
}

fn _rustfmt(s: impl AsRef<[u8]>) -> String {
    use std::io::{ Write};
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
