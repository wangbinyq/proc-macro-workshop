use proc_macro2::{Group, Literal, TokenStream, TokenTree};
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream, Result as ParseResult},
    parse_macro_input, Ident, LitInt, Token,
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
        let start = input.parse::<LitInt>()?;
        input.parse::<Token![..]>()?;
        let end = input.parse::<LitInt>()?;
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
    let seq = parse_macro_input!(input as Seq);
    let tokens: Vec<TokenStream> = (seq.start.base10_parse().unwrap()
        ..seq.end.base10_parse().unwrap())
        .map(|n| {
            let ident = &seq.ident;
            let token: TokenStream = seq
                .content
                .clone()
                .into_iter()
                .map(|tt| replace_token_tree_ident(tt, ident, n))
                .collect();
            token
        })
        .collect();

    let expand = quote! {
        #(#tokens)*
    };

    proc_macro::TokenStream::from(expand)
}

fn replace_token_tree_ident(tt: TokenTree, ident: &Ident, n: isize) -> TokenTree {
    match &tt {
        TokenTree::Ident(id) => {
            if ident == id {
                return TokenTree::Literal(Literal::isize_unsuffixed(n));
            }
            tt
        }
        TokenTree::Group(group) => {
            let stream = group
                .stream()
                .into_iter()
                .map(|tt2| replace_token_tree_ident(tt2, ident, n))
                .collect();
            let span = group.span();
            let mut group = Group::new(group.delimiter(), stream);
            group.set_span(span);
            group.into()
        }
        _ => tt,
    }
}
