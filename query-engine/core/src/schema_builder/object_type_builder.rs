use super::*;
use prisma_models::OrderBy;

#[derive(Debug)]
pub struct ObjectTypeBuilder<'a> {
    internal_data_model: InternalDataModelRef,
    with_relations: bool,
    capabilities: &'a SupportedCapabilities,
    input_type_builder: Weak<InputTypeBuilder<'a>>,
    filter_object_type_builder: Weak<FilterObjectTypeBuilder<'a>>,
    object_type_cache: TypeRefCache<ObjectType>,
}

impl<'a> InputBuilderExtensions for ObjectTypeBuilder<'a> {}

impl<'a> CachedBuilder<ObjectType> for ObjectTypeBuilder<'a> {
    fn get_cache(&self) -> &TypeRefCache<ObjectType> {
        &self.object_type_cache
    }

    fn into_strong_refs(self) -> Vec<Arc<ObjectType>> {
        self.object_type_cache.into()
    }
}

impl<'a> ObjectTypeBuilder<'a> {
    /// Initializes a new ObjectTypeBuilder and constructs the
    pub fn new(
        internal_data_model: InternalDataModelRef,
        with_relations: bool,
        capabilities: &'a SupportedCapabilities,
        filter_object_type_builder: Weak<FilterObjectTypeBuilder<'a>>,
        input_type_builder: Weak<InputTypeBuilder<'a>>,
    ) -> Self {
        ObjectTypeBuilder {
            internal_data_model,
            with_relations,
            capabilities,
            filter_object_type_builder,
            input_type_builder,
            object_type_cache: TypeRefCache::new(),
        }
        .compute_model_object_types()
    }

    pub fn map_model_object_type(&self, model: &ModelRef) -> ObjectTypeRef {
        self.get_cache()
            .get(&model.name)
            .expect("Invariant violation: Initialized object type skeleton for each model.")
    }

    /// Initializes model object type cache on the query schema builder.
    fn compute_model_object_types(self) -> Self {
        // Compute initial cache.
        self.internal_data_model.models().iter().for_each(|m| {
            self.cache(
                m.name.clone(),
                Arc::new(ObjectType::new(m.name.clone(), Some(Arc::clone(&m)))),
            )
        });

        // Compute fields on all cached object types.
        self.internal_data_model.models().iter().for_each(|m| {
            let obj: ObjectTypeRef = self.map_model_object_type(m);
            let fields = self.compute_fields(m);

            obj.into_arc().set_fields(fields);
        });

        self
    }

    /// This assumes that the cache has already been initialized.
    fn compute_fields(&self, model: &ModelRef) -> Vec<Field> {
        model
            .fields()
            .all
            .iter()
            .filter(|f| match f {
                ModelField::Scalar(_) => true,
                ModelField::Relation(_) => self.with_relations,
            })
            .map(|f| self.map_field(f))
            .collect()
    }

    pub fn map_field(&self, model_field: &ModelField) -> Field {
        field(
            model_field.name(),
            self.many_records_field_arguments(&model_field),
            self.map_output_type(&model_field),
            None,
        )
    }

    fn map_output_type(&self, model_field: &ModelField) -> OutputType {
        let output_type = match model_field {
            ModelField::Relation(rf) => {
                let related_model_obj = OutputType::object(self.map_model_object_type(&rf.related_model()));

                if rf.is_list {
                    OutputType::list(related_model_obj)
                } else {
                    related_model_obj
                }
            }
            ModelField::Scalar(sf) => match sf.type_identifier {
                TypeIdentifier::String => OutputType::string(),
                TypeIdentifier::Float => OutputType::float(),
                TypeIdentifier::Boolean => OutputType::boolean(),
                TypeIdentifier::Enum(_) => Self::map_enum_field(sf).into(),
                TypeIdentifier::Json => OutputType::json(),
                TypeIdentifier::DateTime => OutputType::date_time(),
                TypeIdentifier::UUID => OutputType::uuid(),
                TypeIdentifier::Int => OutputType::int(),
            },
        };

        if model_field.is_scalar() && model_field.is_list() {
            OutputType::list(output_type)
        } else if !model_field.is_required() {
            OutputType::opt(output_type)
        } else {
            output_type
        }
    }

    /// Builds "many records where" arguments based on the given model and field.
    pub fn many_records_field_arguments(&self, field: &ModelField) -> Vec<Argument> {
        match field {
            ModelField::Scalar(_) => vec![],
            ModelField::Relation(rf) if rf.is_list && !rf.related_model().is_embedded => {
                self.many_records_arguments(&rf.related_model())
            }
            ModelField::Relation(rf) if rf.is_list && rf.related_model().is_embedded => vec![],
            ModelField::Relation(rf) if !rf.is_list => vec![],
            _ => unreachable!(),
        }
    }

    /// Builds "many records where" arguments solely based on the given model.
    pub fn many_records_arguments(&self, model: &ModelRef) -> Vec<Argument> {
        let unique_input_type = InputType::opt(InputType::object(
            self.input_type_builder.into_arc().where_unique_object_type(model),
        ));

        vec![
            self.where_argument(&model),
            self.order_by_argument(&model),
            argument("skip", InputType::opt(InputType::int()), None),
            argument("after", unique_input_type.clone(), None),
            argument("before", unique_input_type, None),
            argument("first", InputType::opt(InputType::int()), None),
            argument("last", InputType::opt(InputType::int()), None),
        ]
    }

    /// Builds "where" argument.
    pub fn where_argument(&self, model: &ModelRef) -> Argument {
        let where_object = self
            .filter_object_type_builder
            .into_arc()
            .filter_object_type(Arc::clone(model));

        argument("where", InputType::opt(InputType::object(where_object)), None)
    }

    // Builds "orderBy" argument.
    pub fn order_by_argument(&self, model: &ModelRef) -> Argument {
        let enum_values: Vec<_> = model
            .fields()
            .all
            .iter()
            .filter(|field| match field {
                ModelField::Scalar(sf) => !sf.is_list,
                ModelField::Relation(rf) => {
                    !rf.relation().is_many_to_many()
                        && rf.is_inlined_on_enclosing_model()
                        && rf.data_source_fields().len() == 1
                }
            })
            .map(|field| {
                vec![
                    (
                        format!("{}_{}", field.name(), SortOrder::Ascending.abbreviated()),
                        OrderBy {
                            field: field.clone(),
                            sort_order: SortOrder::Ascending,
                        },
                    ),
                    (
                        format!("{}_{}", field.name(), SortOrder::Descending.abbreviated()),
                        OrderBy {
                            field: field.clone(),
                            sort_order: SortOrder::Descending,
                        },
                    ),
                ]
            })
            .flatten()
            .collect();

        let enum_name = format!("{}OrderByInput", model.name);
        let enum_type = order_by_enum_type(enum_name, enum_values);

        argument("orderBy", InputType::opt(enum_type.into()), None)
    }

    pub fn map_enum_field(scalar_field: &Arc<ScalarField>) -> EnumType {
        match scalar_field.type_identifier {
            TypeIdentifier::Enum(_) => {
                let internal_enum = scalar_field.internal_enum.as_ref().expect(
                    "Invariant violation: Enum fields are expected to have an internal_enum associated with them.",
                );

                internal_enum.clone().into()
            }
            _ => panic!("Invariant violation: map_enum_field can only be called on scalar enum fields."),
        }
    }

    pub fn batch_payload_object_type(&self) -> ObjectTypeRef {
        return_cached!(self.get_cache(), "BatchPayload");

        let object_type = Arc::new(object_type(
            "BatchPayload",
            vec![field("count", vec![], OutputType::int(), None)],
            None,
        ));

        self.cache("BatchPayload".into(), Arc::clone(&object_type));
        Arc::downgrade(&object_type)
    }

    /// Builds aggregation object type for given model (e.g. AggregateUser).
    pub fn aggregation_object_type(&self, model: &ModelRef) -> ObjectTypeRef {
        let name = format!("Aggregate{}", capitalize(&model.name));
        return_cached!(self.get_cache(), &name);

        let object = ObjectTypeStrongRef::new(ObjectType::new(&name, Some(ModelRef::clone(model))));
        let fields = vec![field("count", vec![], OutputType::int(), None)];

        object.set_fields(fields);
        self.cache(name, ObjectTypeStrongRef::clone(&object));

        ObjectTypeStrongRef::downgrade(&object)
    }
}
