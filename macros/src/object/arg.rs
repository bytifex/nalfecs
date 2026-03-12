use proc_macro2::Span;

pub struct Arg {
    pub span: Span,
    pub kind: ArgKind,
}

pub enum ArgKind {
    Debug,
    ContainerName(syn::Ident),
}

impl syn::parse::Parse for Arg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let span = input.span();
        let ident: Option<syn::Ident> = input.parse()?;

        if let Some(ident) = ident {
            if ident == "container_name" {
                input.parse::<syn::Token![=]>()?;
                let name = input.parse::<syn::LitStr>()?;
                let name_span = name.span();
                let name = name.value();
                Ok(Arg {
                    span,
                    kind: ArgKind::ContainerName(syn::Ident::new(&name, name_span)),
                })
            } else if ident == "debug" {
                Ok(Arg {
                    span,
                    kind: ArgKind::Debug,
                })
            } else {
                Err(syn::Error::new(
                    ident.span(),
                    format!("unknown argument: {}", ident),
                ))
            }
        } else {
            Err(syn::Error::new(span, "expected ident"))
        }
    }
}

impl ArgKind {
    pub fn is_mutually_exclusive_with(&self, other: &ArgKind) -> bool {
        match (self, other) {
            (ArgKind::ContainerName(_), ArgKind::ContainerName(_)) => true,
            (ArgKind::Debug, ArgKind::Debug) => true,

            (ArgKind::Debug, ArgKind::ContainerName(_)) => false,
            (ArgKind::ContainerName(_), ArgKind::Debug) => false,
        }
    }
}
