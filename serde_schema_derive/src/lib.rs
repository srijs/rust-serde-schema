extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate serde_derive_internals;
extern crate syn;

use std::borrow::Borrow;

use serde_derive_internals::{ast, Ctxt};
use syn::DeriveInput;

mod derive_enum;
mod derive_struct;

#[proc_macro_derive(SchemaSerialize)]
pub fn derive_schema_serialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();

    let cx = Ctxt::new();
    let container = ast::Container::from_ast(&cx, &input);

    let inner_impl = match container.data {
        ast::Data::Enum(variants) => derive_enum::derive_enum(variants, &container.attrs),
        ast::Data::Struct(style, fields) => {
            derive_struct::derive_struct(style, fields, &container.attrs)
        }
    };

    let ident = container.ident;
    let (impl_generics, ty_generics, where_clause) = container.generics.split_for_impl();

    let expanded = quote!{
        impl #impl_generics ::serde_schema::SchemaSerialize for #ident #ty_generics #where_clause {
            fn schema_register<S>(schema: &mut S) -> Result<S::TypeId, S::Error>
                where S: ::serde_schema::Schema
            {
                #inner_impl
            }
        }
    };

    cx.check().unwrap();

    expanded.into()
}

fn variant_field_type_variable(variant_idx: usize, field_idx: usize) -> syn::Ident {
    syn::Ident::from(format!("type_id_{}_{}", variant_idx, field_idx))
}

fn derive_register_field_types<'a, I>(variant_idx: usize, fields: I) -> quote::Tokens
where
    I: IntoIterator,
    I::Item: Borrow<ast::Field<'a>>,
{
    let mut expanded = quote!{};
    for (field_idx, field_item) in fields.into_iter().enumerate() {
        let field = field_item.borrow();
        let field_type = &field.ty;
        let type_id_ident = variant_field_type_variable(variant_idx, field_idx);
        expanded.append_all(quote!{
            let #type_id_ident =
                <#field_type as ::serde_schema::SchemaSerialize>::schema_register(schema)?;
        });
    }
    expanded
}

fn derive_field<'a>(variant_idx: usize, field_idx: usize, field: &ast::Field<'a>) -> quote::Tokens {
    let type_id_ident = variant_field_type_variable(variant_idx, field_idx);
    let field_name = field.attrs.name().serialize_name();
    quote!{
        .field(#field_name, #type_id_ident)
    }
}

fn derive_element<'a>(variant_idx: usize, element_idx: usize) -> quote::Tokens {
    let type_id_ident = variant_field_type_variable(variant_idx, element_idx);
    quote!{
        .element(#type_id_ident)
    }
}
