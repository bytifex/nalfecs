mod container_iter_map;
mod object;

use proc_macro::TokenStream;

// fn create_generic_idents(generics: &syn::Generics) -> syn::Generics {
//     syn::Generics {
//         params: generics
//             .params
//             .iter()
//             .map(|param| match param {
//                 syn::GenericParam::Lifetime(lifetime_param) => {
//                     syn::GenericParam::Lifetime(syn::LifetimeParam {
//                         attrs: vec![],
//                         lifetime: lifetime_param.lifetime.clone(),
//                         colon_token: None,
//                         bounds: syn::punctuated::Punctuated::default(),
//                     })
//                 }
//                 syn::GenericParam::Type(type_param) => syn::GenericParam::Type(syn::TypeParam {
//                     attrs: vec![],
//                     ident: type_param.ident.clone(),
//                     colon_token: None,
//                     bounds: syn::punctuated::Punctuated::default(),
//                     eq_token: None,
//                     default: None,
//                 }),
//                 syn::GenericParam::Const(const_param) => syn::GenericParam::Type(syn::TypeParam {
//                     attrs: vec![],
//                     ident: const_param.ident.clone(),
//                     colon_token: None,
//                     bounds: syn::punctuated::Punctuated::default(),
//                     eq_token: None,
//                     default: None,
//                 }),
//             })
//             .collect(),
//         ..Default::default()
//     }
// }

// fn create_generics_for_impl(generics: &syn::Generics) -> syn::Generics {
//     let mut generics = generics.clone();
//     for param in &mut generics.params {
//         match param {
//             syn::GenericParam::Lifetime(_) => continue,
//             syn::GenericParam::Type(type_param) => {
//                 type_param.eq_token = None;
//                 type_param.default = None;
//             }
//             syn::GenericParam::Const(const_param) => {
//                 const_param.eq_token = None;
//                 const_param.default = None;
//             }
//         }
//     }
//     generics
// }

#[proc_macro_attribute]
pub fn object(arg: TokenStream, input: TokenStream) -> TokenStream {
    object::object(arg, input)
}

#[proc_macro]
pub fn container_iter_map(input: TokenStream) -> TokenStream {
    container_iter_map::container_iter_map(input)
}

#[proc_macro]
pub fn container_iter_map_debug(input: TokenStream) -> TokenStream {
    let tokens = container_iter_map::container_iter_map(input);
    eprintln!("{}", tokens);
    tokens
}
