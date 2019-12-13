use crate::SqlResult;
use chrono::*;
use datamodel::common::*;
use datamodel::*;
use itertools::Itertools;
use prisma_models::{DatamodelConverter, TempManifestationHolder, TempRelationHolder};
use sql_schema_describer as sql;
use sql_schema_describer::ColumnArity;

pub struct SqlSchemaCalculator<'a> {
    data_model: &'a Datamodel,
}

impl<'a> SqlSchemaCalculator<'a> {
    pub fn calculate(data_model: &Datamodel) -> SqlResult<sql::SqlSchema> {
        let calculator = SqlSchemaCalculator { data_model };
        calculator.calculate_internal()
    }

    fn calculate_internal(&self) -> SqlResult<sql::SqlSchema> {
        let mut tables = Vec::new();
        let model_tables_without_inline_relations = self.calculate_model_tables()?;
        let mut model_tables = self.add_inline_relations_to_model_tables(model_tables_without_inline_relations)?;
        let mut relation_tables = self.calculate_relation_tables()?;

        tables.append(&mut model_tables);
        tables.append(&mut relation_tables);

        // guarantee same sorting as in the sql-schema-describer
        for table in &mut tables {
            table.columns.sort_unstable_by_key(|col| col.name.clone());
        }

        let enums = Vec::new();
        let sequences = Vec::new();

        Ok(sql::SqlSchema {
            tables,
            enums,
            sequences,
        })
    }

    fn calculate_model_tables(&self) -> SqlResult<Vec<ModelTable>> {
        self.data_model
            .models()
            .map(|model| {
                let columns = model
                    .fields()
                    .flat_map(|f| match &f.field_type {
                        FieldType::Base(_) | FieldType::Enum(_) => Some(sql::Column {
                            name: f.db_name(),
                            tpe: column_type(f),
                            default: f.migration_value_new(&self.data_model),
                            auto_increment: {
                                match f.id_info {
                                    Some(IdInfo {
                                        strategy: IdStrategy::Auto,
                                        sequence: _,
                                    }) if column_type(f).family == sql::ColumnTypeFamily::Int => true,
                                    _ => false,
                                }
                            },
                        }),
                        _ => None,
                    })
                    .collect();

                let primary_key = sql::PrimaryKey {
                    columns: vec![model.id_field()?.db_name()],
                    sequence: None,
                };

                let single_field_indexes = model.fields().filter_map(|f| {
                    if f.is_unique {
                        Some(sql::Index {
                            name: format!("{}.{}", &model.db_name(), &f.db_name()),
                            columns: vec![f.db_name().clone()],
                            tpe: sql::IndexType::Unique,
                        })
                    } else {
                        None
                    }
                });

                let multiple_field_indexes = model.indexes.iter().map(|index_definition: &IndexDefinition| {
                    let referenced_fields: Vec<&Field> = index_definition
                        .fields
                        .iter()
                        .map(|field_name| model.find_field(field_name).expect("Unknown field in index directive."))
                        .collect();

                    sql::Index {
                        name: index_definition.name.clone().unwrap_or_else(|| {
                            format!(
                                "{}.{}",
                                &model.db_name(),
                                referenced_fields.iter().map(|field| field.db_name()).join("_")
                            )
                        }),
                        // The model index definition uses the model field names, but the SQL Index
                        // wants the column names.
                        columns: referenced_fields.iter().map(|field| field.db_name()).collect(),
                        tpe: if index_definition.tpe == IndexType::Unique {
                            sql::IndexType::Unique
                        } else {
                            sql::IndexType::Normal
                        },
                    }
                });

                let table = sql::Table {
                    name: model.db_name(),
                    columns,
                    indices: single_field_indexes.chain(multiple_field_indexes).collect(),
                    primary_key: Some(primary_key),
                    foreign_keys: Vec::new(),
                };

                Ok(ModelTable {
                    model: model.clone(),
                    table,
                })
            })
            .collect()
    }

    fn add_inline_relations_to_model_tables(&self, model_tables: Vec<ModelTable>) -> SqlResult<Vec<sql::Table>> {
        let mut result = Vec::new();
        let relations = self.calculate_relations();
        for mut model_table in model_tables {
            for relation in relations.iter() {
                match &relation.manifestation {
                    TempManifestationHolder::Inline {
                        in_table_of_model,
                        column: column_name,
                    } if in_table_of_model == &model_table.model.name => {
                        let (model, related_model) = if model_table.model == relation.model_a {
                            (&relation.model_a, &relation.model_b)
                        } else {
                            (&relation.model_b, &relation.model_a)
                        };

                        let field = model.fields().find(|f| &f.db_name() == column_name).unwrap();

                        let column = sql::Column {
                            name: column_name.to_string(),
                            tpe: column_type_for_scalar_type(
                                scalar_type_for_field(related_model.id_field()?),
                                column_arity(&field),
                            ),
                            default: None,
                            auto_increment: false,
                        };
                        let foreign_key = sql::ForeignKey {
                            constraint_name: None,
                            columns: vec![column_name.to_string()],
                            referenced_table: related_model.db_name(),
                            referenced_columns: vec![related_model.id_field()?.db_name()],
                            on_delete_action: if column.is_required() {
                                sql::ForeignKeyAction::Restrict
                            } else {
                                sql::ForeignKeyAction::SetNull
                            },
                        };
                        model_table.table.columns.push(column);
                        model_table.table.foreign_keys.push(foreign_key);

                        if relation.is_one_to_one() {
                            add_one_to_one_relation_unique_index(&mut model_table.table, column_name)
                        }
                    }
                    _ => {}
                }
            }
            result.push(model_table.table);
        }
        Ok(result)
    }

    fn calculate_relation_tables(&self) -> SqlResult<Vec<sql::Table>> {
        let mut result = Vec::new();
        for relation in self.calculate_relations().iter() {
            match &relation.manifestation {
                TempManifestationHolder::Table => {
                    let foreign_keys = vec![
                        sql::ForeignKey {
                            constraint_name: None,
                            columns: vec![relation.model_a_column()],
                            referenced_table: relation.model_a.db_name(),
                            referenced_columns: vec![relation.model_a.id_field()?.db_name()],
                            on_delete_action: sql::ForeignKeyAction::Cascade,
                        },
                        sql::ForeignKey {
                            constraint_name: None,
                            columns: vec![relation.model_b_column()],
                            referenced_table: relation.model_b.db_name(),
                            referenced_columns: vec![relation.model_b.id_field()?.db_name()],
                            on_delete_action: sql::ForeignKeyAction::Cascade,
                        },
                    ];
                    let table = sql::Table {
                        name: relation.table_name(),
                        columns: vec![
                            sql::Column {
                                name: relation.model_a_column(),
                                tpe: column_type(relation.model_a.id_field()?),
                                default: None,
                                auto_increment: false,
                            },
                            sql::Column {
                                name: relation.model_b_column(),
                                tpe: column_type(relation.model_b.id_field()?),
                                default: None,
                                auto_increment: false,
                            },
                        ],
                        indices: vec![sql::Index {
                            name: format!("{}_AB_unique", relation.table_name()),
                            columns: vec![relation.model_a_column(), relation.model_b_column()],
                            tpe: sql::IndexType::Unique,
                        }],
                        primary_key: None,
                        foreign_keys,
                    };
                    result.push(table);
                }
                _ => {}
            }
        }
        Ok(result)
    }

    fn calculate_relations(&self) -> Vec<TempRelationHolder> {
        DatamodelConverter::calculate_relations(&self.data_model)
    }
}

#[derive(PartialEq, Debug)]
struct ModelTable {
    table: sql::Table,
    model: Model,
}

pub trait ModelExtensions {
    fn id_field(&self) -> Result<&Field, String>;

    fn db_name(&self) -> String;
}

impl ModelExtensions for Model {
    fn id_field(&self) -> Result<&Field, String> {
        match self.fields().find(|f| f.is_id()) {
            Some(f) => Ok(f),
            None => Err(format!("Model {} does not have an id field", self.name)),
        }
    }

    fn db_name(&self) -> String {
        self.database_name.clone().unwrap_or_else(|| self.name.clone())
    }
}

pub trait FieldExtensions {
    fn is_id(&self) -> bool;

    fn is_list(&self) -> bool;

    fn is_required(&self) -> bool;

    fn db_name(&self) -> String;

    fn migration_value(&self, datamodel: &Datamodel) -> ScalarValue;

    fn migration_value_new(&self, datamodel: &Datamodel) -> Option<String>;
}

impl FieldExtensions for Field {
    fn is_id(&self) -> bool {
        self.id_info.is_some()
    }

    fn is_list(&self) -> bool {
        self.arity == FieldArity::List
    }

    fn is_required(&self) -> bool {
        self.arity == FieldArity::Required
    }

    fn db_name(&self) -> String {
        self.database_name.clone().unwrap_or_else(|| self.name.clone())
    }

    fn migration_value(&self, datamodel: &Datamodel) -> ScalarValue {
        self.default_value
            .clone()
            .unwrap_or_else(|| default_migration_value(&self.field_type, datamodel))
    }

    fn migration_value_new(&self, datamodel: &Datamodel) -> Option<String> {
        let value = match &self.default_value {
            Some(x) => match x {
                ScalarValue::Expression(_, _, _) => default_migration_value(&self.field_type, datamodel),
                x => x.clone(),
            },
            None => default_migration_value(&self.field_type, datamodel),
        };
        let result = match value {
            ScalarValue::Boolean(x) => {
                if x {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            ScalarValue::Int(x) => format!("{}", x),
            ScalarValue::Float(x) => format!("{}", x),
            ScalarValue::Decimal(x) => format!("{}", x),
            ScalarValue::String(x) => format!("{}", x),

            ScalarValue::DateTime(x) => {
                let mut raw = format!("{}", x); // this will produce a String 1970-01-01 00:00:00 UTC
                raw.truncate(raw.len() - 4); // strip the UTC suffix
                format!("{}", raw)
            }
            ScalarValue::ConstantLiteral(x) => format!("{}", x), // this represents enum values
            ScalarValue::Expression(_, _, _) => {
                unreachable!("expressions must have been filtered out in the preceding pattern match")
            }
        };
        if self.is_id() {
            None
        } else {
            Some(result)
        }
    }
}

fn default_migration_value(field_type: &FieldType, datamodel: &Datamodel) -> ScalarValue {
    match field_type {
        FieldType::Base(ScalarType::Boolean) => ScalarValue::Boolean(false),
        FieldType::Base(ScalarType::Int) => ScalarValue::Int(0),
        FieldType::Base(ScalarType::Float) => ScalarValue::Float(0.0),
        FieldType::Base(ScalarType::String) => ScalarValue::String("".to_string()),
        FieldType::Base(ScalarType::Decimal) => ScalarValue::Decimal(0.0),
        FieldType::Base(ScalarType::DateTime) => {
            let naive = NaiveDateTime::from_timestamp(0, 0);
            let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
            ScalarValue::DateTime(datetime)
        }
        FieldType::Enum(ref enum_name) => {
            let inum = datamodel
                .find_enum(&enum_name)
                .expect(&format!("Enum {} was not present in the Datamodel.", enum_name));
            let first_value = inum
                .values
                .first()
                .expect(&format!("Enum {} did not contain any values.", enum_name));
            ScalarValue::String(first_value.to_string())
        }
        _ => unimplemented!("this functions must only be called for scalar fields"),
    }
}

fn column_type(field: &Field) -> sql::ColumnType {
    column_type_for_scalar_type(scalar_type_for_field(field), column_arity(field))
}

fn scalar_type_for_field(field: &Field) -> &ScalarType {
    match &field.field_type {
        FieldType::Base(ref scalar) => &scalar,
        FieldType::Enum(_) => &ScalarType::String,
        x => panic!(format!(
            "This field type is not suported here. Field type is {:?} on field {}",
            x, field.name
        )),
    }
}

fn column_arity(field: &Field) -> sql::ColumnArity {
    match &field.arity {
        FieldArity::Required => sql::ColumnArity::Required,
        FieldArity::List => sql::ColumnArity::List,
        FieldArity::Optional => sql::ColumnArity::Nullable,
    }
}

fn column_type_for_scalar_type(scalar_type: &ScalarType, column_arity: ColumnArity) -> sql::ColumnType {
    match scalar_type {
        ScalarType::Int => sql::ColumnType::pure(sql::ColumnTypeFamily::Int, column_arity),
        ScalarType::Float => sql::ColumnType::pure(sql::ColumnTypeFamily::Float, column_arity),
        ScalarType::Boolean => sql::ColumnType::pure(sql::ColumnTypeFamily::Boolean, column_arity),
        ScalarType::String => sql::ColumnType::pure(sql::ColumnTypeFamily::String, column_arity),
        ScalarType::DateTime => sql::ColumnType::pure(sql::ColumnTypeFamily::DateTime, column_arity),
        ScalarType::Decimal => unimplemented!(),
    }
}

fn add_one_to_one_relation_unique_index(table: &mut sql::Table, column_name: &str) {
    let index = sql::Index {
        name: format!("{}_{}", table.name, column_name),
        columns: vec![column_name.to_string()],
        tpe: sql::IndexType::Unique,
    };

    table.indices.push(index);
}
