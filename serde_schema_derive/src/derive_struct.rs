use quote;
use serde_derive_internals::{ast, attr};

use super::{derive_element, derive_field, derive_register_field_types, variant_field_type_variable};

pub fn derive_struct<'a>(
    style: ast::Style,
    fields: Vec<ast::Field<'a>>,
    attr_container: &attr::Container,
) -> quote::Tokens {
    match style {
        ast::Style::Struct => derive_struct_named_fields(fields, attr_container),
        ast::Style::Newtype => derive_struct_newtype(fields, attr_container),
        ast::Style::Tuple => derive_struct_tuple(fields, attr_container),
        ast::Style::Unit => derive_struct_unit(attr_container),
    }
}

fn derive_struct_newtype<'a>(
    fields: Vec<ast::Field<'a>>,
    attr_container: &attr::Container,
) -> quote::Tokens {
    let name = attr_container.name().serialize_name();
    let expanded_type_ids = derive_register_field_types(0, fields.iter());
    let type_id_ident = variant_field_type_variable(0, 0);
    quote!{
        #expanded_type_ids
        ::serde_schema::Schema::register_type(schema,
            ::serde_schema::types::Type::build()
                .newtype_struct_type(#name, #type_id_ident))
    }
}

fn derive_struct_unit(attr_container: &attr::Container) -> quote::Tokens {
    let name = attr_container.name().serialize_name();
    quote!{
        ::serde_schema::Schema::register_type(schema,
            ::serde_schema::types::Type::build().unit_struct_type(#name))
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
        ::serde_schema::types::Type::build()
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

fn derive_struct_tuple<'a>(
    fields: Vec<ast::Field<'a>>,
    attr_container: &attr::Container,
) -> quote::Tokens {
    let len = fields.len();
    let name = attr_container.name().serialize_name();

    let expanded_type_ids = derive_register_field_types(0, fields.iter());

    let mut expanded_build_type = quote!{
        ::serde_schema::types::Type::build()
            .tuple_struct_type(#name, #len)
    };
    for (element_idx, _) in fields.iter().enumerate() {
        expanded_build_type.append_all(derive_element(0, element_idx));
    }
    expanded_build_type.append_all(quote!{
        .end()
    });

    quote!{
        #expanded_type_ids
        ::serde_schema::Schema::register_type(schema, #expanded_build_type)
    }
}
