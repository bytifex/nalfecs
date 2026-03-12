use crate::object::arg::{Arg, ArgKind};

pub struct Args {
    container_name: syn::Ident,
    debug: bool,
}

impl Args {
    pub fn container_name(&self) -> &syn::Ident {
        &self.container_name
    }

    pub fn debug(&self) -> bool {
        self.debug
    }
}

impl syn::parse::Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let span = input.span();
        let args = syn::punctuated::Punctuated::<Arg, syn::Token![,]>::parse_terminated(input)?;

        // checking for mutual exclusions
        for later_idx in 1..args.len() {
            for earlier_idx in 0..later_idx {
                let arg_earlier = args.get(earlier_idx).unwrap();
                let arg_later = args.get(later_idx).unwrap();
                if arg_later.kind.is_mutually_exclusive_with(&arg_earlier.kind) {
                    return Err(syn::Error::new(
                        arg_later.span,
                        "argument is excluded by another argument",
                    ));
                }
            }
        }

        // processing arguments
        let mut debug = false;
        let mut container_name = None;
        for arg in args {
            match arg.kind {
                ArgKind::Debug => debug = true,
                ArgKind::ContainerName(name) => container_name = Some(name),
            }
        }

        Ok(Self {
            debug,
            container_name: container_name
                .ok_or_else(|| syn::Error::new(span, "missing argument = `container_name`"))?,
        })
    }
}
