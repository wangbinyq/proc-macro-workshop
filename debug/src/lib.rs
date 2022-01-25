use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Field, Fields, FieldsNamed};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let name_str = format!("{}", name);
    let fields: Vec<Field> = fields(&input.data)
        .expect("struct must with named fields")
        .named
        .into_iter()
        .map(|f| f)
        .collect();
    let debug_fields: Vec<_> = fields.iter().map(debug_field).collect();

    let expand = quote! {

        impl ::core::fmt::Debug for #name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct(#name_str)
                    #(#debug_fields)*
                    .finish()
            }
        }
    };

    proc_macro::TokenStream::from(expand)
}

fn fields(data: &Data) -> Option<FieldsNamed> {
    if let Data::Struct(ds) = data {
        if let Fields::Named(fields) = &ds.fields {
            Some(fields.clone())
        } else {
            None
        }
    } else {
        None
    }
}

fn debug_field(field: &Field) -> TokenStream {
    let ident = field.ident.as_ref().unwrap();
    let name = ident.to_string();
    quote! {
        .field(#name, &self.#ident)
    }
}
