extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate serde_derive_internals;
extern crate syn;

use std::borrow::Borrow;

use serde_derive_internals::{ast, attr, Ctxt};
use syn::DeriveInput;

#[proc_macro_derive(SchemaSerialize)]
pub fn derive_schema_serialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();

    let cx = Ctxt::new();
    let container = ast::Container::from_ast(&cx, &input);

    let inner_impl = match container.data {
        ast::Data::Enum(variants) => derive_enum_impl(variants, &container.attrs),
        ast::Data::Struct(style, fields) => match style {
            ast::Style::Struct => derive_struct_named_fields(fields, &container.attrs),
            ast::Style::Newtype => panic!("newtype structs are not supported yet"),
            ast::Style::Tuple => panic!("tuple structs are not supported yet"),
            ast::Style::Unit => derive_struct_unit(&container.attrs),
        },
    };

    let ident = container.ident;
    let generics = container.generics;

    let expanded = quote!{
        impl #generics ::serde_schema::SchemaSerialize for #ident #generics {
            fn schema_register<S>(schema: &mut S) -> Result<S::TypeId, S::Error>
                where S: serde_schema::Schema
            {
                #inner_impl
            }
        }
    };

    cx.check().unwrap();

    expanded.into()
}

fn derive_enum_impl<'a>(
    variants: Vec<ast::Variant<'a>>,
    attr_container: &attr::Container,
) -> quote::Tokens {
    let name = attr_container.name().serialize_name();
    let len = variants.len();

    let mut expanded_type_ids = quote!{};
    for (variant_idx, variant) in variants.iter().enumerate() {
        expanded_type_ids.append_all(derive_register_field_types(
            variant_idx,
            variant.fields.iter(),
        ));
    }

    let mut expanded_build_type = quote!{
        serde_schema::types::Type::build()
            .enum_type(#name, #len)
    };

    for (variant_idx, variant) in variants.iter().enumerate() {
        let variant_name = variant.attrs.name().serialize_name();
        let fields_len = variant.fields.len();
        match variant.style {
            ast::Style::Struct => {
                let mut expanded_inner = quote!{
                    .struct_variant(#variant_name, #fields_len)
                };
                for (field_idx, field) in variant.fields.iter().enumerate() {
                    expanded_inner.append_all(derive_field(variant_idx, field_idx, field));
                }
                expanded_inner.append_all(quote!{
                    .end()
                });
                expanded_build_type.append_all(expanded_inner);
            }
            ast::Style::Newtype => {
                let field_type = variant_field_type_variable(variant_idx, 0);
                expanded_build_type.append_all(quote!{
                    .newtype_variant(#variant_name, #field_type)
                });
            }
            ast::Style::Tuple => panic!("tuple variants are not supported yet"),
            ast::Style::Unit => panic!("unit variants are not supported yet"),
        }
    }

    expanded_build_type.append_all(quote!{
        .end()
    });

    quote!{
        #expanded_type_ids
        ::serde_schema::Schema::register_type(schema, #expanded_build_type)
    }
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

fn derive_struct_unit(attr_container: &attr::Container) -> quote::Tokens {
    let name = attr_container.name().serialize_name();
    quote!{
        ::serde_schema::Schema::register_type(schema,
            serde_schema::types::Type::build().unit_struct_type(#name))
    }
}

fn derive_struct_named_fields<'a>(
    fields: Vec<ast::Field<'a>>,
    attr_container: &attr::Container,
) -> quote::Tokens {
    let len = fields.len();
    let name = attr_container.name().serialize_name();

    let expanded_type_ids = derive_register_field_types(0, fields.iter());

    let mut expanded_build_type = quote!{
        serde_schema::types::Type::build()
            .struct_type(#name, #len)
    };
    for (field_idx, field) in fields.iter().enumerate() {
        expanded_build_type.append_all(derive_field(0, field_idx, field));
    }
    expanded_build_type.append_all(quote!{
        .end()
    });

    quote!{
        #expanded_type_ids
        ::serde_schema::Schema::register_type(schema, #expanded_build_type)
    }
}
