use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Data, DeriveInput, Error, Field, Fields,
    FieldsNamed, GenericArgument, Lit, Meta, NestedMeta, PathArguments, Type,
};

#[proc_macro_derive(Builder, attributes(builder))]
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

            pub fn build(&mut self) -> ::core::result::Result<Command, ::std::boxed::Box<dyn ::std::error::Error>> {
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
            ::core::option::Option<#ty>
        );
        field.ty = ty;
    };
    field.attrs = ::std::vec![];

    field
}

fn init_builder_fields(field: &Field) -> TokenStream {
    let name = field
        .ident
        .as_ref()
        .expect(&format!("field must named: {:?}", field.span()));

    quote! {
        #name: ::core::option::Option::None
    }
}

fn builder_setter(field: &Field) -> TokenStream {
    let name = field
        .ident
        .as_ref()
        .expect(&format!("field must named: {:?}", field.span()));
    let ty = inner_optional_field(field).unwrap_or(field.ty.clone());

    match get_each_attr(field) {
        Err(err) => Error::into_compile_error(err),
        Ok(each_field) => {
            let mut setter = quote! {
                pub fn #name(&mut self, #name: #ty) -> &mut Self {
                    self.#name = ::core::option::Option::Some(#name);
                    self
                }
            };

            if let Some(each_field) = each_field {
                let inner_ty = inner_vec_field(field).unwrap();
                let each_setter = quote! {
                    pub fn #each_field(&mut self, #name: #inner_ty) -> &mut Self {
                        if let ::core::option::Option::Some(field) = &mut self.#name {
                            field.push(#name);
                        } else {
                            self.#name = ::core::option::Option::Some(vec![#name]);
                        }
                        self
                    }
                };
                if &each_field == name {
                    setter = each_setter;
                } else {
                    setter = quote! {
                        #setter

                        #each_setter
                    };
                }
            }
            setter
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
    } else if is_vec_field(field) {
        quote! {
            #name: self.#name.clone().unwrap_or_default()
        }
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

fn is_vec_field(field: &Field) -> bool {
    if let Type::Path(ty) = &field.ty {
        ty.path.segments[0].ident == "Vec"
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

fn inner_vec_field(field: &Field) -> Option<Type> {
    if let Type::Path(ty) = &field.ty {
        if ty.path.segments[0].ident == "Vec" {
            if let PathArguments::AngleBracketed(args) = &ty.path.segments[0].arguments {
                if let Some(GenericArgument::Type(ty)) = args.args.first() {
                    return Some(ty.clone());
                }
            }
        }
    }
    None
}

fn get_each_attr(field: &Field) -> Result<Option<Ident>, Error> {
    let metas: Vec<_> = field
        .attrs
        .iter()
        .map(|attr| attr.parse_meta())
        .collect::<Result<_, _>>()?;

    if metas.len() > 0 {
        if let Meta::List(ml) = &metas[0] {
            if let Some(NestedMeta::Meta(meta)) = &ml.nested.iter().next() {
                if let Meta::NameValue(nv) = meta {
                    if let Lit::Str(str) = &nv.lit {
                        if nv.path.segments[0].ident.to_string() == "each" {
                            let str: Ident = str.parse()?;
                            return Ok(Some(str));
                        }
                    }
                }
            }
        }
        Err(Error::new(
            field.span(),
            r#"expected `builder(each = "...")`"#,
        ))
    } else {
        Ok(None)
    }
}
