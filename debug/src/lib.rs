use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, Data, DeriveInput, Error, Field, Fields, FieldsNamed,
    GenericParam, Generics, Lit, Meta,
};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let name_str = format!("{}", name);
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let fields: Vec<Field> = fields(input.data)
        .expect("struct must with named fields")
        .named
        .into_iter()
        .map(|f| f)
        .collect();
    let debug_fields: Vec<_> = fields.iter().map(debug_field).collect();

    let expand = quote! {

        impl #impl_generics ::core::fmt::Debug for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct(#name_str)
                    #(#debug_fields)*
                    .finish()
            }
        }
    };

    proc_macro::TokenStream::from(expand)
}

fn fields(data: Data) -> Option<FieldsNamed> {
    if let Data::Struct(ds) = data {
        if let Fields::Named(fields) = ds.fields {
            Some(fields)
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
    let metas = field
        .attrs
        .iter()
        .map(|attr| attr.parse_meta())
        .collect::<Result<Vec<_>, Error>>()
        .unwrap();

    let format = metas.first().and_then(|meta| {
        if let Meta::NameValue(nv) = meta {
            if let Lit::Str(str) = &nv.lit {
                return Some(str);
            }
        }
        None
    });

    let value = if let Some(format) = format {
        quote! {
            &::core::format_args!(#format, &self.#ident)
        }
    } else {
        quote! {
            self.#ident
        }
    };

    quote! {
        .field(#name, &#value)
    }
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(::core::fmt::Debug));
        }
    }
    generics
}
