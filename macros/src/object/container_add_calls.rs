use quote::quote;

pub struct ContainerAddCalls<'a>(pub &'a [(syn::Ident, &'a syn::Ident, &'a syn::Type)]);

impl quote::ToTokens for ContainerAddCalls<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for (container_field_ident, field_ident, _field_type) in self.0.iter() {
            tokens.extend(quote! {
                let index = self.#container_field_ident.write().add(object.#field_ident);

                debug_assert_eq!(
                    object_index_in_object_container,
                    nalfecs::ObjectIndexInObjectContainer::from(index),
                    "component indices diverged while adding object"
                );
            });
        }
    }
}
