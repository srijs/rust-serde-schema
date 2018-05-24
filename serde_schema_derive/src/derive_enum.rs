use quote;
use serde_derive_internals::{ast, attr};

use super::{derive_field, derive_register_field_types, variant_field_type_variable};

pub fn derive_enum<'a>(
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
        let expanded_build_variant = match variant.style {
            ast::Style::Struct => {
                derive_struct_variant(&variant_name, variant_idx, &variant.fields)
            }
            ast::Style::Newtype => derive_newtype_variant(&variant_name, variant_idx),
            ast::Style::Tuple => panic!("tuple variants are not supported yet"),
            ast::Style::Unit => derive_unit_variant(&variant_name),
        };
        expanded_build_type.append_all(expanded_build_variant);
    }

    expanded_build_type.append_all(quote!{
        .end()
    });

    quote!{
        #expanded_type_ids
        ::serde_schema::Schema::register_type(schema, #expanded_build_type)
    }
}

fn derive_unit_variant<'a>(variant_name: &str) -> quote::Tokens {
    quote!{
        .unit_variant(#variant_name)
    }
}

fn derive_newtype_variant<'a>(variant_name: &str, variant_idx: usize) -> quote::Tokens {
    let field_type = variant_field_type_variable(variant_idx, 0);
    quote!{
        .newtype_variant(#variant_name, #field_type)
    }
}

fn derive_struct_variant<'a>(
    variant_name: &str,
    variant_idx: usize,
    fields: &Vec<ast::Field<'a>>,
) -> quote::Tokens {
    let fields_len = fields.len();
    let mut expanded = quote!{
        .struct_variant(#variant_name, #fields_len)
    };
    for (field_idx, field) in fields.iter().enumerate() {
        expanded.append_all(derive_field(variant_idx, field_idx, field));
    }
    expanded.append_all(quote!{
        .end()
    });
    expanded
}
