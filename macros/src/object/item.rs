use proc_macro2::Span;
use quote::quote;

use crate::object::{
    container_add_calls::ContainerAddCalls, container_field_definitions::ContainerFieldDefinitions,
};

type ContainerField<'a> = (syn::Ident, &'a syn::Ident, &'a syn::Type);

pub struct Item(syn::ItemStruct);

impl syn::parse::Parse for Item {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let span = input.span();

        let item_struct: syn::ItemStruct = input.parse()?;

        if !matches!(item_struct.fields, syn::Fields::Named(_)) {
            return Err(syn::Error::new(span, "expected struct with named fields"));
        }

        Ok(Item(item_struct))
    }
}

impl Item {
    fn index_lit(index: usize) -> syn::LitInt {
        syn::LitInt::new(&index.to_string(), Span::call_site())
    }

    fn type_id_tokens(ty: &syn::Type) -> proc_macro2::TokenStream {
        quote! {
            ::std::any::TypeId::of::<#ty>()
        }
    }

    fn container_remove_calls(container_fields: &[ContainerField<'_>]) -> proc_macro2::TokenStream {
        container_fields
            .iter()
            .map(|(container_field_ident, field_ident, _field_type)| {
                quote! {
                    let #field_ident = self.#container_field_ident.write().remove(index.into())?;
                }
            })
            .collect()
    }

    fn object_builder_fields(container_fields: &[ContainerField<'_>]) -> proc_macro2::TokenStream {
        container_fields
            .iter()
            .map(|(_container_field_ident, field_ident, _field_type)| {
                quote! {
                    #field_ident,
                }
            })
            .collect()
    }

    fn view_descriptor_component_access_checks(
        container_fields: &[ContainerField<'_>],
    ) -> proc_macro2::TokenStream {
        container_fields
            .iter()
            .enumerate()
            .map(|(index, (_container_field_ident, _field_ident, ty))| {
                let index_lit = Self::index_lit(index);
                let type_id = Self::type_id_tokens(ty);

                quote! {
                    if *type_id == #type_id {
                        return Some((#index_lit, component_access));
                    }
                }
            })
            .collect()
    }

    fn iter_for_match_cases(container_fields: &[ContainerField<'_>]) -> proc_macro2::TokenStream {
        container_fields
            .iter()
            .enumerate()
            .map(|(index, (container_field_ident, _field_ident, ty))| {
                let index_literal = Self::index_lit(index);
                let type_id = Self::type_id_tokens(ty);

                quote! {
                    #index_literal => Some((
                        #type_id,
                        if matches!(access_type, nalfecs::ComponentAccessType::Mutable) {
                            nalfecs::ComponentContainerGuard::mutable(&self.#container_field_ident)
                        } else {
                            nalfecs::ComponentContainerGuard::immutable(&self.#container_field_ident)
                        },
                    )),
                }
            })
            .collect()
    }

    fn semantic_duplicate_component_type_check(
        container_fields: &[ContainerField<'_>],
    ) -> proc_macro2::TokenStream {
        let uniqueness_impls: proc_macro2::TokenStream = container_fields
            .iter()
            .map(|(_container_field_ident, _field_ident, ty)| {
                quote! {
                    impl __NalfecsUniqueComponentType<#ty> for __NalfecsUniqueComponentTypeChecker {}
                }
            })
            .collect();

        quote! {
            const _: () = {
                trait __NalfecsUniqueComponentType<T> {}
                struct __NalfecsUniqueComponentTypeChecker;

                #uniqueness_impls
            };
        }
    }

    pub fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream, container_name: &syn::Ident) {
        let item_struct = &self.0;
        let item_name = &item_struct.ident;

        let container_fields = item_struct
            .fields
            .iter()
            .map(|field| {
                let field_ident = field
                    .ident
                    .as_ref()
                    .expect("only structs with named fields are allowed");

                let container_field_ident =
                    syn::Ident::new(&format!("{}_container", field_ident), field_ident.span());

                (container_field_ident, field_ident, &field.ty)
            })
            .collect::<Vec<ContainerField<'_>>>();

        let container_field_definitions = ContainerFieldDefinitions(&container_fields);
        let container_add_calls = ContainerAddCalls(&container_fields);
        let container_remove_calls = Self::container_remove_calls(&container_fields);
        let object_builder_fields = Self::object_builder_fields(&container_fields);

        let container_type_ids = container_fields
            .iter()
            .map(|(_container_field_ident, _field_ident, ty)| Self::type_id_tokens(ty))
            .collect::<syn::punctuated::Punctuated<proc_macro2::TokenStream, syn::Token![,]>>();
        let item_field_count = Self::index_lit(container_fields.len());

        let view_descriptor_component_access_checks =
            Self::view_descriptor_component_access_checks(&container_fields);

        let iter_for_match_cases = Self::iter_for_match_cases(&container_fields);
        let semantic_duplicate_component_type_check =
            Self::semantic_duplicate_component_type_check(&container_fields);

        tokens.extend(quote! {
            #item_struct
            #semantic_duplicate_component_type_check
        });

        tokens.extend(quote! {
            #[derive(Default)]
            pub struct #container_name {
                lock: nalfecs::parking_lot::Mutex<()>,
                #container_field_definitions
            }

            impl nalfecs::ObjectContainer for #container_name {
                fn view_descriptor(
                    &self,
                    component_accesses: &[nalfecs::ComponentAccess],
                ) -> Option<nalfecs::ComponentViewDescriptorForObjectContainer> {
                    let component_indices = component_accesses
                        .iter()
                        .map(|component_access| {
                            let (type_id, component_access) = match component_access {
                                nalfecs::ComponentAccess::Immutable(type_id) => {
                                    (type_id, nalfecs::ComponentAccessType::Immutable)
                                }
                                nalfecs::ComponentAccess::Mutable(type_id) => {
                                    (type_id, nalfecs::ComponentAccessType::Mutable)
                                }
                            };

                            #view_descriptor_component_access_checks
                            None
                        })
                        .collect::<Option<_>>()?;

                    Some(nalfecs::ComponentViewDescriptorForObjectContainer::new::<Self>(component_indices))
                }

                fn iter_for(
                    &self,
                    desc: &nalfecs::ComponentViewDescriptorForObjectContainer,
                ) -> Option<nalfecs::ComponentViewIterator<'_>> {
                    let _guard = self.lock.lock();

                    if desc.object_container_type_id() != ::std::any::TypeId::of::<Self>() {
                        return None;
                    }

                    let component_containers = desc
                        .component_container_accesses()
                        .iter()
                        .map(|(component_container_id, access_type)| {
                            match component_container_id {
                                #iter_for_match_cases
                                _ => None,
                            }
                        })
                        .collect::<Option<_>>()?;

                    Some(nalfecs::ComponentViewIterator::new(component_containers))
                }
            }

            impl #container_name {
                pub const fn component_type_ids() -> &'static [::std::any::TypeId] {
                    static TYPE_IDS: [::std::any::TypeId; #item_field_count] = [
                        #container_type_ids
                    ];

                    &TYPE_IDS
                }

                pub fn new() -> Self {
                    Default::default()
                }

                pub fn add(&self, object: #item_name) -> nalfecs::ObjectIndexInObjectContainer {
                    let _guard = self.lock.lock();
                    let mut object_index_in_object_container = None;

                    #container_add_calls

                    object_index_in_object_container
                        .expect("object structs with named fields always have at least one field")
                        .into()
                }

                pub fn remove(
                    &self,
                    index: nalfecs::ObjectIndexInObjectContainer,
                ) -> Option<#item_name> {
                    let _guard = self.lock.lock();

                    #container_remove_calls

                    Some(#item_name {
                        #object_builder_fields
                    })
                }
            }

            impl nalfecs::ObjectContainerFor<#item_name> for #container_name {
                fn add_object(&self, object: #item_name) -> nalfecs::ObjectIndexInObjectContainer {
                    self.add(object)
                }

                fn remove_object(
                    &self,
                    index: nalfecs::ObjectIndexInObjectContainer,
                ) -> Option<#item_name> {
                    self.remove(index)
                }
            }

            impl nalfecs::Object for #item_name {
                type Container = #container_name;
            }
        });
    }
}
