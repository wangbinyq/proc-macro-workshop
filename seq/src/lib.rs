use proc_macro2::{Group, Literal, TokenStream, TokenTree};
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream, Result as ParseResult},
    parse_macro_input, parse_quote, Ident, LitInt, Token,
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
            let token: TokenStream = transform_token_stream(seq.content.clone(), ident, n);
            token
        })
        .collect();

    let expand = quote! {
        #(#tokens)*
    };

    proc_macro::TokenStream::from(expand)
}

fn transform_token_stream(ts: TokenStream, ident: &Ident, n: isize) -> TokenStream {
    let mut tokens = ts.into_iter().peekable();

    let mut transformed: Vec<TokenTree> = vec![];

    while let Some(tt) = tokens.next() {
        let mut replace_tile = false;
        let mut tt = match &tt {
            TokenTree::Punct(p) => {
                let mut tt = tt.clone();
                if p.as_char() == '~' {
                    if let Some(TokenTree::Ident(id)) = tokens.peek() {
                        if id == ident {
                            replace_tile = true;
                            let span = p.span();
                            tokens.next();
                            tt = TokenTree::Literal(Literal::isize_unsuffixed(n));
                            tt.set_span(span);
                        }
                    }
                }
                tt
            }
            TokenTree::Ident(id) => {
                if id == ident {
                    TokenTree::Literal(Literal::isize_unsuffixed(n))
                } else {
                    tt
                }
            }
            TokenTree::Group(group) => {
                let stream = group.stream();
                let stream = transform_token_stream(stream, ident, n);
                let span = group.span();
                let mut group = Group::new(group.delimiter(), stream);
                group.set_span(span);
                group.into()
            }
            _ => tt,
        };
        if replace_tile {
            if let Some(c) = transformed.pop() {
                let ident = Ident::new(&format!("{}{}", c, tt), c.span());
                tt = parse_quote! {
                    #ident
                };
            }
        }
        transformed.push(tt);
    }

    quote! {
        #(#transformed)*
    }
}
