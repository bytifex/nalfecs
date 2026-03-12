mod access_type;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;

use crate::container_iter_map::access_type::AccessType;

struct Input {
    generic_types: Vec<AccessType>,
    container: syn::Expr,
    view_desc: syn::Expr,
}

impl syn::parse::Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<syn::Token![<]>()?;

        let generics =
            syn::punctuated::Punctuated::<AccessType, syn::Token![,]>::parse_separated_nonempty(
                input,
            )?
            .into_iter()
            .collect();

        input.parse::<syn::Token![>]>()?;

        input.parse::<syn::Token![,]>()?;

        let container = input.parse::<syn::Expr>()?;

        input.parse::<syn::Token![,]>()?;

        let view_desc = input.parse::<syn::Expr>()?;

        input.parse::<Option<syn::Token![,]>>()?;

        Ok(Self {
            generic_types: generics,
            container,
            view_desc,
        })
    }
}

impl quote::ToTokens for Input {
    // This method creates something like the following:

    // {
    //     let view_desc = &view_desc;
    //     assert_eq!(
    //         view_desc.number_of_components(),
    //         3,
    //         "number of components do not match with view descriptor",
    //     );
    //     container
    //         .iter_object_container_view_iters(view_desc)
    //         .map(|mut iter| {
    //             let container_0 = iter.component_container_unchecked::<component::Transform>(0);
    //             let container_1 = iter.component_container_mut_unchecked::<component::RigidBody>(1);
    //             let container_2 =
    //                 iter.component_container_mut_unchecked::<component::Appearance>(2);
    //             assert_eq!(
    //                 container_0.len(),
    //                 container_1.len(),
    //                 "component length mismatch, component indices = (0, 1)"
    //             );
    //             assert_eq!(
    //                 container_0.len(),
    //                 container_2.len(),
    //                 "component length mismatch, component indices = (0, 2)"
    //             );
    //             container_0
    //                 .iter_with_index()
    //                 .zip(container_1.iter_mut().zip(container_2.iter_mut()))
    //                 .map(move |(component_0, (component_1, component_2))| {
    //                     let (object_index_in_object_container, component_0) = component_0;
    //                     let object_index = iter.object_index(object_index_in_object_container);
    //                     (object_index, component_0, component_1, component_2)
    //                 })
    //         })
    //         .flatten()
    // }
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let container = &self.container;
        let view_desc = &self.view_desc;

        let container_definitions: TokenStream2 = self
            .generic_types
            .iter()
            .enumerate()
            .map(|(index, generic_type)| {
                let (generic_type, is_mutable) = match generic_type {
                    AccessType::Immutable(ty) => (ty, false),
                    AccessType::Mutable(ty) => (ty, true),
                };
                let container_varname =
                    syn::Ident::new(&format!("container_{}", index), Span::call_site());
                let index_literal = syn::LitInt::new(&index.to_string(), Span::call_site());

                if is_mutable {
                    quote! {
                        let #container_varname = iter
                            .component_container_mut_unchecked::<#generic_type>(#index_literal);
                    }
                } else {
                    quote! {
                        let #container_varname = iter
                            .component_container_unchecked::<#generic_type>(#index_literal);
                    }
                }
            })
            .collect();

        let container_len_asserts: TokenStream2 = self
            .generic_types
            .iter()
            .enumerate()
            .skip(1)
            .map(|(index, _generic_type)| {
                let container_varname =
                    syn::Ident::new(&format!("container_{}", index), Span::call_site());
                let msg_literal = format!(
                    "component length mismatch, component indices = (0, {})",
                    index
                );

                quote! {
                    assert_eq!(container_0.len(), #container_varname.len(), #msg_literal);
                }
            })
            .collect();

        let number_of_components_literal =
            syn::LitInt::new(&self.generic_types.len().to_string(), Span::call_site());

        let mut iter_calls: Vec<TokenStream2> = self
            .generic_types
            .iter()
            .enumerate()
            .map(|(index, generic_type)| {
                let container_varname =
                    syn::Ident::new(&format!("container_{}", index), Span::call_site());

                let container_iter_method = match (index, generic_type) {
                    (0, AccessType::Immutable(_)) => {
                        quote! {
                            iter_with_index
                        }
                    }
                    (0, AccessType::Mutable(_)) => {
                        quote! {
                            iter_mut_with_index
                        }
                    }
                    (_, AccessType::Immutable(_)) => {
                        quote! {
                            iter
                        }
                    }
                    (_, AccessType::Mutable(_)) => {
                        quote! {
                            iter_mut
                        }
                    }
                };

                quote! {
                    #container_varname.#container_iter_method()
                }
            })
            .collect();

        let last_component_iter_call = iter_calls
            .pop()
            .expect("at least one type has to be defined");

        let container_iter_zips: TokenStream2 =
            iter_calls
                .into_iter()
                .rev()
                .fold(last_component_iter_call, |acc, iter_call| {
                    quote! {
                        #iter_call.zip(#acc)
                    }
                });

        let component_varnames: Vec<TokenStream2> = self
            .generic_types
            .iter()
            .enumerate()
            .map(|(index, _)| {
                let component_varname =
                    syn::Ident::new(&format!("component_{}", index), Span::call_site());

                quote! {
                    #component_varname
                }
            })
            .collect();

        let component_enumeration: TokenStream2 = component_varnames
            .iter()
            .map(|component_varname| {
                quote! {
                    #component_varname,
                }
            })
            .collect();

        let first_component_name = component_varnames
            .first()
            .expect("at least one component has to be defined")
            .clone();

        let last_component_name = component_varnames
            .last()
            .expect("at least one component has to be defined")
            .clone();

        // .map(|(component_0, (component_1, component_2))| {
        let container_iter_zip_map_args: TokenStream2 = component_varnames
            .into_iter()
            .rev()
            .skip(1)
            .fold(last_component_name, |acc, component_varname| {
                quote! {
                    (#component_varname, #acc)
                }
            });

        tokens.extend(quote! {
            {
                let view_desc = #view_desc;

                assert_eq!(
                    view_desc.number_of_components(),
                    #number_of_components_literal,
                    "number of components do not match with view descriptor",
                );

                #container
                    .iter_object_container_view_iters(view_desc)
                    .map(|mut iter| {
                        #container_definitions

                        #container_len_asserts

                        #container_iter_zips.map(move |#container_iter_zip_map_args| {
                            let (object_index_in_object_container, #first_component_name) = #first_component_name;
                            let object_index = iter.object_index(object_index_in_object_container);
                            (object_index, #component_enumeration)
                        })
                    })
                    .flatten()
            }
        });
    }
}

pub fn container_iter_map(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as Input);

    let tokens = quote! {
        #input
    };

    tokens.into()
}
