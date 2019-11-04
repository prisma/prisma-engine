use super::*;
use once_cell::sync::OnceCell;
use prisma_models::{EnumType, EnumValue, ModelRef, PrismaValue};
use std::sync::Arc;

/// Object type initializer for cases where only the name is known, and fields are computed later.
pub fn init_object_type<T>(name: T, model: Option<ModelRef>) -> ObjectType
where
    T: Into<String>,
{
    ObjectType {
        name: name.into(),
        fields: OnceCell::new(),
        model,
    }
}

/// Object type convenience wrapper function.
pub fn object_type<T>(name: T, fields: Vec<Field>, model: Option<ModelRef>) -> ObjectType
where
    T: Into<String>,
{
    let object_type = init_object_type(name.into(), model);

    object_type.set_fields(fields);
    object_type
}

/// Input object type convenience wrapper function.
pub fn input_object_type<T>(name: T, fields: Vec<InputField>) -> InputObjectType
where
    T: Into<String>,
{
    let object_type = init_input_object_type(name.into());

    object_type.set_fields(fields);
    object_type
}

/// Input object type initializer for cases where only the name is known, and fields are computed later.
pub fn init_input_object_type<T>(name: T) -> InputObjectType
where
    T: Into<String>,
{
    InputObjectType {
        name: name.into(),
        fields: OnceCell::new(),
    }
}

/// Enum type convenience wrapper function.
pub fn enum_type<T>(name: T, values: Vec<EnumValue>) -> EnumType
where
    T: Into<String>,
{
    EnumType {
        name: name.into(),
        values,
    }
}

/// Argument convenience wrapper function.
pub fn argument<T>(name: T, arg_type: InputType, default_value: Option<PrismaValue>) -> Argument
where
    T: Into<String>,
{
    Argument {
        name: name.into(),
        argument_type: arg_type,
        default_value,
    }
}

/// Field convenience wrapper function.
pub fn field<T>(
    name: T,
    arguments: Vec<Argument>,
    field_type: OutputType,
    query_builder: Option<SchemaQueryBuilder>,
) -> Field
where
    T: Into<String>,
{
    Field {
        name: name.into(),
        arguments,
        field_type: Arc::new(field_type),
        query_builder,
    }
}

/// Field convenience wrapper function.
pub fn input_field<T>(name: T, field_type: InputType, default_value: Option<PrismaValue>) -> InputField
where
    T: Into<String>,
{
    InputField {
        name: name.into(),
        field_type,
        default_value,
    }
}

/// Pluralizes given (English) input string. Falls back to appending "s".
pub fn pluralize<T>(s: T) -> String
where
    T: AsRef<str>,
{
    prisma_inflector::default().pluralize(s.as_ref())
}

/// Lowercases first letter, essentially.
/// Assumes 1-byte characters, panics otherwise.
pub fn camel_case<T>(s: T) -> String
where
    T: Into<String>,
{
    let s = s.into();

    // This is safe to unwrap, as the validation regex for model / field
    // names used in the data model essentially guarantees ASCII.
    let first_char = s.chars().next().unwrap();

    format!("{}{}", first_char.to_lowercase(), s[1..].to_owned())
}

/// Capitalizes first character.
/// Assumes 1-byte characters.
pub fn capitalize<T>(s: T) -> String
where
    T: Into<String>,
{
    let s = s.into();

    // This is safe to unwrap, as the validation regex for model / field
    // names used in the data model essentially guarantees ASCII.
    let first_char = s.chars().next().unwrap();

    format!("{}{}", first_char.to_uppercase(), s[1..].to_owned())
}

/// Appends an option of type T to a vector over T if the option is Some.
pub fn append_opt<T>(vec: &mut Vec<T>, opt: Option<T>) {
    opt.into_iter().for_each(|t| vec.push(t));
}
