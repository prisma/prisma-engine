use super::*;

use crate::constants::outputs::fields;
use prisma_models::ScalarFieldRef;

/// Initializes model output object type cache on the context.
/// This is a critical first step to ensure that all model output object types are present
/// and that subsequent schema computation has a base to rely on.
/// Called only once at the very beginning of schema building.
pub(crate) fn initialize_model_object_type_cache(ctx: &mut BuilderContext) {
    // Compute initial cache. No fields are computed because we first
    // need all models to be present, then we can compute fields in a second pass.
    ctx.internal_data_model
        .models()
        .to_owned()
        .into_iter()
        .for_each(|model| {
            let ident = Identifier::new(model.name.clone(), MODEL_NAMESPACE);
            ctx.cache_output_type(ident.clone(), Arc::new(ObjectType::new(ident, Some(model))))
        });

    // Compute fields on all cached object types.
    ctx.internal_data_model
        .models()
        .to_owned()
        .into_iter()
        .for_each(|model| {
            let obj: ObjectTypeWeakRef = output_objects::map_model_object_type(ctx, &model);
            let fields = compute_model_object_type_fields(ctx, &model);

            obj.into_arc().set_fields(fields);
        });
}

/// Computes model output type fields.
/// Important: This requires that the cache has already been initialized.
fn compute_model_object_type_fields(ctx: &mut BuilderContext, model: &ModelRef) -> Vec<OutputField> {
    model
        .fields()
        .all
        .iter()
        .map(|f| output_objects::map_field(ctx, f))
        .collect()
}

/// Returns an output object type for the given model.
/// Relies on the output type cache being initalized.
pub(crate) fn map_model_object_type(ctx: &mut BuilderContext, model: &ModelRef) -> ObjectTypeWeakRef {
    let ident = Identifier::new(model.name.clone(), MODEL_NAMESPACE);
    ctx.get_output_type(&ident)
        .expect("Invariant violation: Initialized output object type for each model.")
}

pub(crate) fn map_field(ctx: &mut BuilderContext, model_field: &ModelField) -> OutputField {
    field(
        model_field.name(),
        arguments::many_records_field_arguments(ctx, &model_field),
        map_output_type(ctx, &model_field),
        None,
    )
    .nullable_if(!model_field.is_required())
}

pub(crate) fn map_output_type(ctx: &mut BuilderContext, model_field: &ModelField) -> OutputType {
    match model_field {
        ModelField::Scalar(sf) => map_scalar_output_type_for_field(ctx, sf),
        ModelField::Relation(rf) => map_relation_output_type(ctx, rf),
    }
}

pub(crate) fn map_scalar_output_type_for_field(ctx: &mut BuilderContext, field: &ScalarFieldRef) -> OutputType {
    map_scalar_output_type(ctx, &field.type_identifier, field.is_list)
}

pub(crate) fn map_scalar_output_type(ctx: &mut BuilderContext, typ: &TypeIdentifier, list: bool) -> OutputType {
    let output_type = match typ {
        TypeIdentifier::String => OutputType::string(),
        TypeIdentifier::Float => OutputType::float(),
        TypeIdentifier::Decimal => OutputType::decimal(),
        TypeIdentifier::Boolean => OutputType::boolean(),
        TypeIdentifier::Enum(e) => map_enum_type(ctx, &e).into(),
        TypeIdentifier::Json => OutputType::json(),
        TypeIdentifier::DateTime => OutputType::date_time(),
        TypeIdentifier::UUID => OutputType::uuid(),
        TypeIdentifier::Int => OutputType::int(),
        TypeIdentifier::Xml => OutputType::xml(),
        TypeIdentifier::Bytes => OutputType::bytes(),
        TypeIdentifier::BigInt => OutputType::bigint(),
        TypeIdentifier::Unsupported => unreachable!("No unsupported field should reach that path"),
    };

    if list {
        OutputType::list(output_type)
    } else {
        output_type
    }
}

pub(crate) fn map_relation_output_type(ctx: &mut BuilderContext, field: &RelationFieldRef) -> OutputType {
    let related_model_obj = OutputType::object(map_model_object_type(ctx, &field.related_model()));

    if field.is_list {
        OutputType::list(related_model_obj)
    } else {
        related_model_obj
    }
}

fn map_enum_type(ctx: &mut BuilderContext, enum_name: &str) -> EnumType {
    let e = ctx
        .internal_data_model
        .find_enum(enum_name)
        .expect("Enum references must always be valid.");

    e.into()
}

pub(crate) fn affected_records_object_type(ctx: &mut BuilderContext) -> ObjectTypeWeakRef {
    let ident = Identifier::new("AffectedRowsOutput".to_owned(), PRISMA_NAMESPACE);
    return_cached_output!(ctx, &ident);

    let object_type = Arc::new(object_type(
        ident.clone(),
        vec![field(fields::COUNT, vec![], OutputType::int(), None)],
        None,
    ));

    ctx.cache_output_type(ident, object_type.clone());
    Arc::downgrade(&object_type)
}
