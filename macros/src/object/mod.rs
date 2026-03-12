mod arg;
mod args;
mod container_add_calls;
mod container_field_definitions;
mod item;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

use crate::object::{args::Args, item::Item};

pub fn object(arg: TokenStream, input: TokenStream) -> TokenStream {
    let args: Args = syn::parse_macro_input!(arg);

    let item: Item = syn::parse_macro_input!(input);

    let mut tokens = TokenStream2::new();
    item.to_tokens(&mut tokens, args.container_name());

    if args.debug() {
        eprintln!("{}", tokens);
    }

    tokens.into()
}
