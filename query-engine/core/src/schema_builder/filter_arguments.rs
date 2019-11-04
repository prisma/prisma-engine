use super::*;

pub struct FilterArgument {
    pub suffix: &'static str,
    pub is_list: bool,
}

/// Wrapper type for static initialization (circumventing lazy_static type issues).
struct StaticFilterArguments {
    pub base: Vec<FilterArgument>,
    pub inclusion: Vec<FilterArgument>,
    pub alphanumeric: Vec<FilterArgument>,
    pub string: Vec<FilterArgument>,
    pub multi_relation: Vec<FilterArgument>,
    pub one_relation: Vec<FilterArgument>,
}

lazy_static! {
    static ref FILTER_ARGUMENTS: StaticFilterArguments = StaticFilterArguments {
        base: vec![
            FilterArgument { suffix: "", is_list: false },
            FilterArgument { suffix: "_not",is_list: false }, // All values that are not equal to given value.
        ],

        inclusion: vec![
            FilterArgument { suffix: "_in",is_list: true }, // All values that are contained in given list.
            FilterArgument { suffix: "_not_in",is_list: true } // All values that are not contained in given list.
        ],

        alphanumeric: vec![
            FilterArgument { suffix: "_lt",is_list: false }, // All values less than the given value.
            FilterArgument { suffix: "_lte",is_list: false }, // All values less than or equal the given value.
            FilterArgument { suffix: "_gt",is_list: false }, // All values greater than the given value.
            FilterArgument { suffix: "_gte",is_list: false } // All values greater than or equal the given value.
        ],

        string: vec![
            FilterArgument { suffix: "_contains",is_list: false }, // All values containing the given string.
            FilterArgument { suffix: "_not_contains",is_list: false }, // All values not containing the given string.
            FilterArgument { suffix: "_starts_with",is_list: false }, // All values starting with the given string.
            FilterArgument { suffix: "_not_starts_with",is_list: false }, // All values not starting with the given string.
            FilterArgument { suffix: "_ends_with",is_list: false }, // All values ending with the given string.
            FilterArgument { suffix: "_not_ends_with",is_list: false } // All values not ending with the given string.
        ],

        multi_relation: vec![
            FilterArgument { suffix: "_every",is_list: false }, // All records where all records in the relation satisfy the given condition.
            FilterArgument { suffix: "_some",is_list: false }, // All records that have at least one record in the relation satisfying the given condition.
            FilterArgument { suffix: "_none",is_list: false } // All records that have no record in the relation satisfying the given condition.
        ],

        one_relation: vec![FilterArgument { suffix: "", is_list: false }],
    };
}

pub fn get_field_filters<'a>(field: &ModelField) -> Vec<&'a FilterArgument> {
    let args = &FILTER_ARGUMENTS;

    if field.is_list() {
        match field.type_identifier() {
            TypeIdentifier::Relation => args.multi_relation.iter().collect(),
            _ => vec![],
        }
    } else {
        let filters = match field.type_identifier() {
            TypeIdentifier::UUID => vec![&args.base, &args.inclusion, &args.alphanumeric, &args.string],
            TypeIdentifier::GraphQLID => vec![&args.base, &args.inclusion, &args.alphanumeric, &args.string],
            TypeIdentifier::String => vec![&args.base, &args.inclusion, &args.alphanumeric, &args.string],
            TypeIdentifier::Int => vec![&args.base, &args.inclusion, &args.alphanumeric],
            TypeIdentifier::Float => vec![&args.base, &args.inclusion, &args.alphanumeric],
            TypeIdentifier::Boolean => vec![&args.base],
            TypeIdentifier::Enum => vec![&args.base, &args.inclusion],
            TypeIdentifier::DateTime => vec![&args.base, &args.inclusion, &args.alphanumeric],
            TypeIdentifier::Json => vec![],
            TypeIdentifier::Relation => vec![&args.one_relation],
        };

        filters
            .into_iter()
            .map(|l| l.iter().collect::<Vec<&'a FilterArgument>>())
            .flatten()
            .collect()
    }
}
