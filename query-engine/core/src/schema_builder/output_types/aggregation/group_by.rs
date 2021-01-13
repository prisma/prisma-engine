use super::*;
use std::convert::identity;

/// Builds group by aggregation object type for given model (e.g. GroupByUserOutputType).
pub(crate) fn group_by_output_object_type(ctx: &mut BuilderContext, model: &ModelRef) -> ObjectTypeWeakRef {
    let ident = Identifier::new(
        format!("{}GroupByOutputType", capitalize(&model.name)),
        PRISMA_NAMESPACE,
    );
    return_cached_output!(ctx, &ident);

    let object = Arc::new(ObjectType::new(ident.clone(), Some(ModelRef::clone(model))));

    // Model fields that can be grouped by value.
    let mut object_fields = scalar_fields(ctx, model);

    // Fields used in aggregations
    let non_list_fields = collect_non_list_fields(model);
    let numeric_fields = collect_numeric_fields(model);

    // Count is available on all fields.
    append_opt(
        &mut object_fields,
        aggregation_field(
            ctx,
            "count",
            &model,
            model.fields().scalar(),
            |_, _| OutputType::int(),
            |mut obj| {
                obj.add_field(field("_all", vec![], OutputType::int(), None));
                obj
            },
        ),
    );

    append_opt(
        &mut object_fields,
        aggregation_field(
            ctx,
            "avg",
            &model,
            numeric_fields.clone(),
            field_avg_output_type,
            identity,
        ),
    );

    append_opt(
        &mut object_fields,
        aggregation_field(
            ctx,
            "sum",
            &model,
            numeric_fields.clone(),
            map_scalar_output_type_for_field,
            identity,
        ),
    );

    append_opt(
        &mut object_fields,
        aggregation_field(
            ctx,
            "min",
            &model,
            non_list_fields.clone(),
            map_scalar_output_type_for_field,
            identity,
        ),
    );

    append_opt(
        &mut object_fields,
        aggregation_field(
            ctx,
            "max",
            &model,
            non_list_fields,
            map_scalar_output_type_for_field,
            identity,
        ),
    );

    object.set_fields(object_fields);
    ctx.cache_output_type(ident, ObjectTypeStrongRef::clone(&object));

    ObjectTypeStrongRef::downgrade(&object)
}

fn scalar_fields(ctx: &mut BuilderContext, model: &ModelRef) -> Vec<OutputField> {
    let fields = model.fields().scalar();

    fields
        .into_iter()
        .map(|f| {
            field(f.name.clone(), vec![], map_scalar_output_type_for_field(ctx, &f), None).optional_if(!f.is_required)
        })
        .collect()
}
