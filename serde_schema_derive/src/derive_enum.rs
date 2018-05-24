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
