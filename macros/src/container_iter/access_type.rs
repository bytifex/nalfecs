pub enum AccessType {
    Immutable(syn::Type),
    Mutable(syn::Type),
}

impl syn::parse::Parse for AccessType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mutable: Option<syn::Token![mut]> = input.parse()?;
        let ty = input.parse()?;
        if mutable.is_some() {
            Ok(Self::Mutable(ty))
        } else {
            Ok(Self::Immutable(ty))
        }
    }
}
