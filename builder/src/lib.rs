use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Data, DeriveInput, Field, Fields,
    FieldsNamed, GenericArgument, PathArguments, Type,
};

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let builder_name = Ident::new(&format!("{}Builder", name), Span::call_site());

    let fields: Vec<Field> = fields(&input.data)
        .expect("struct must with named fields")
        .named
        .into_iter()
        .map(|f| f)
        .collect();

    let builder_fields: Vec<_> = fields.iter().map(wrap_option).collect();
    let builder_init_fields: Vec<_> = fields.iter().map(init_builder_fields).collect();
    let builder_setter: Vec<_> = fields.iter().map(builder_setter).collect();
    let build_fields: Vec<_> = fields.iter().map(build_fields).collect();

    let expanded = quote! {
        pub struct #builder_name {
            #(#builder_fields),*
        }

        impl #builder_name {
            #(#builder_setter)*

            pub fn build(&mut self) -> Result<Command, Box<dyn std::error::Error>> {
                Ok(#name {
                    #(#build_fields),*
                })
            }
        }

        impl #name {
            pub fn builder() -> #builder_name {
                #builder_name {
                    #(#builder_init_fields),*
                }
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
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

fn wrap_option(field: &Field) -> Field {
    let ty = &field.ty;
    let mut field = field.clone();

    if !is_optional_field(&field) {
        let ty = parse_quote!(
            Option<#ty>
        );
        field.ty = ty;
    };

    field
}

fn init_builder_fields(field: &Field) -> TokenStream {
    let name = field
        .ident
        .as_ref()
        .expect(&format!("field must named: {:?}", field.span()));

    quote! {
        #name: None
    }
}

fn builder_setter(field: &Field) -> TokenStream {
    let name = field
        .ident
        .as_ref()
        .expect(&format!("field must named: {:?}", field.span()));
    let ty = inner_optional_field(field).unwrap_or(field.ty.clone());

    quote! {
        pub fn #name(&mut self, #name: #ty) -> &mut Self {
            self.#name = Some(#name);
            self
        }
    }
}

fn build_fields(field: &Field) -> TokenStream {
    let name = field
        .ident
        .as_ref()
        .expect(&format!("field must named: {:?}", field.span()));
    let error = format!("field {} is not set", name);
    if is_optional_field(field) {
        quote! {#name: self.#name.clone() }
    } else {
        quote! {
            #name: self.#name.clone().ok_or(#error)?
        }
    }
}

fn is_optional_field(field: &Field) -> bool {
    if let Type::Path(ty) = &field.ty {
        ty.path.segments[0].ident == "Option"
    } else {
        false
    }
}

fn inner_optional_field(field: &Field) -> Option<Type> {
    if let Type::Path(ty) = &field.ty {
        if ty.path.segments[0].ident == "Option" {
            if let PathArguments::AngleBracketed(args) = &ty.path.segments[0].arguments {
                if let Some(GenericArgument::Type(ty)) = args.args.first() {
                    return Some(ty.clone());
                }
            }
        }
    }
    None
}
