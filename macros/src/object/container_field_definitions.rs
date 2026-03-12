use quote::quote;

pub struct ContainerFieldDefinitions<'a>(pub &'a [(syn::Ident, &'a syn::Ident, &'a syn::Type)]);

impl quote::ToTokens for ContainerFieldDefinitions<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for (container_field_ident, _field_ident, field_type) in self.0.iter() {
            tokens.extend(quote! {
                #container_field_ident: nalfecs::parking_lot::RwLock<
                    nalfecs::ComponentContainer<#field_type>
                >,
            });
        }
    }
}
