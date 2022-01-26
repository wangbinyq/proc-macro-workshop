use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream, Result as ParseResult},
    Ident, LitInt, Token,
};

struct Seq {
    ident: Ident,
    start: LitInt,
    end: LitInt,
    content: TokenStream,
}

impl Parse for Seq {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let ident = input.parse()?;
        input.parse::<Token![in]>()?;
        let start = input.parse()?;
        input.parse::<Token![..]>()?;
        let end = input.parse()?;
        let content;
        braced!(content in input);
        let content = content.parse()?;
        Ok(Self {
            ident,
            start,
            end,
            content,
        })
    }
}

#[proc_macro]
pub fn seq(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let _ = input;

    let expand = quote! {};

    proc_macro::TokenStream::from(expand)
}
