use super::*;

pub trait CreateInputTypeBuilderExtension<'a>: InputTypeBuilderBase<'a> {
    /// Builds the create input type (<x>CreateInput / <x>CreateWithout<y>Input)
    #[rustfmt::skip]
    fn create_input_type(&self, model: ModelRef, parent_field: Option<RelationFieldRef>) -> InputObjectTypeRef {
        let name = match parent_field.as_ref().map(|pf| pf.related_field()) {
            Some(ref f) => format!("{}CreateWithout{}Input", model.name, capitalize(f.name.as_str())),
            _ => format!("{}CreateInput", model.name),
        };

        return_cached!(self.get_cache(), &name);

        let input_object = Arc::new(init_input_object_type(name.clone()));

        // Cache empty object for circuit breaking
        self.cache(name, Arc::clone(&input_object));

        // Compute input fields for scalar fields.
        let scalar_fields: Vec<ScalarFieldRef> = model
            .fields()
            .scalar()
            .into_iter()
            .filter(|f|  Self::field_should_be_kept_for_create_input_type(&f))
            .collect();

        let mut fields = self.scalar_input_fields(
            model.name.clone(),
            "Create",
            scalar_fields,
            |f: ScalarFieldRef| {
                if f.is_required && f.default_value().is_none() && (f.is_created_at() || f.is_updated_at()) { //todo shouldnt these also be Default Value expressions at some point?
                    self.map_optional_input_type(&f)
                } else if f.is_required && f.default_value().is_none(){
                    self.map_required_input_type(&f)
                } else {
                    self.map_optional_input_type(&f)
                }
            },
            true
        );

        // Compute input fields for relational fields.
        let mut relational_fields = self.relation_input_fields_create(Arc::clone(&model), parent_field.as_ref());
        fields.append(&mut relational_fields);

        input_object.set_fields(fields);
        Arc::downgrade(&input_object)
    }

    /// For create input types only. Compute input fields for relational fields.
    /// This recurses into create_input_type (via nested_create_input_field).
    fn relation_input_fields_create(
        &self,
        model: ModelRef,
        parent_field: Option<&RelationFieldRef>,
    ) -> Vec<InputField> {
        model
            .fields()
            .relation()
            .into_iter()
            .filter_map(|rf| {
                let related_model = rf.related_model();
                let related_field = rf.related_field();

                // Compute input object name
                let arity_part = if rf.is_list { "Many" } else { "One" };
                let without_part = format!("Without{}", capitalize(&related_field.name));
                let input_name = format!("{}Create{}{}Input", related_model.name, arity_part, without_part);
                let field_is_opposite_relation_field = parent_field
                    .as_ref()
                    .and_then(|pf| {
                        // TODO: The original version compared full case classes. Is this solution here enough?
                        if pf.related_field().name == rf.name {
                            Some(pf)
                        } else {
                            None
                        }
                    })
                    .is_some();

                if field_is_opposite_relation_field {
                    None
                } else {
                    let input_object = match self.get_cache().get(&input_name) {
                        Some(t) => t,
                        None => {
                            let input_object = Arc::new(init_input_object_type(input_name.clone()));
                            self.cache(input_name, Arc::clone(&input_object));

                            let mut fields = vec![self.nested_create_input_field(Arc::clone(&rf))];
                            let nested_connect = self.nested_connect_input_field(Arc::clone(&rf));
                            append_opt(&mut fields, nested_connect);

                            input_object.set_fields(fields);
                            Arc::downgrade(&input_object)
                        }
                    };

                    let input_type = InputType::object(input_object);
                    let input_field = if rf.is_required {
                        input_field(rf.name.clone(), input_type, None)
                    } else {
                        input_field(rf.name.clone(), InputType::opt(input_type), None)
                    };

                    Some(input_field)
                }
            })
            .collect()
    }

    fn nested_create_input_field(&self, field: RelationFieldRef) -> InputField {
        let input_object = self.create_input_type(field.related_model(), Some(Arc::clone(&field)));
        let input_object = Self::wrap_list_input_object_type(input_object, field.is_list);

        input_field("create", input_object, None)
    }

    fn field_should_be_kept_for_create_input_type(field: &ScalarFieldRef) -> bool {
        !field.is_auto_generated_int_id
    }
}
