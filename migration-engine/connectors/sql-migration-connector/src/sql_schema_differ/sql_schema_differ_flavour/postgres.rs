use super::SqlSchemaDifferFlavour;
use crate::flavour::SqlFlavour;
use crate::{
    flavour::PostgresFlavour,
    pair::Pair,
    sql_migration::AlterEnum,
    sql_schema_differ::column::{ColumnDiffer, ColumnTypeChange},
    sql_schema_differ::SqlSchemaDiffer,
};
use migration_connector::MigrationFeature;
use native_types::PostgresType;
use once_cell::sync::Lazy;
use regex::RegexSet;
use sql_schema_describer::{walkers::IndexWalker, ColumnTypeFamily};

/// The maximum length of postgres identifiers, in bytes.
///
/// Reference: https://www.postgresql.org/docs/12/limits.html
const POSTGRES_IDENTIFIER_SIZE_LIMIT: usize = 63;

impl SqlSchemaDifferFlavour for PostgresFlavour {
    fn alter_enums(&self, differ: &SqlSchemaDiffer<'_>) -> Vec<AlterEnum> {
        differ
            .enum_pairs()
            .filter_map(|differ| {
                let step = AlterEnum {
                    index: differ.enums.as_ref().map(|e| e.enum_index()),
                    created_variants: differ.created_values().map(String::from).collect(),
                    dropped_variants: differ.dropped_values().map(String::from).collect(),
                };

                if step.is_empty() {
                    None
                } else {
                    Some(step)
                }
            })
            .collect()
    }

    fn index_should_be_renamed(&self, pair: &Pair<IndexWalker<'_>>) -> bool {
        // Implements correct comparison for truncated index names.
        let (previous_name, next_name) = pair.as_ref().map(|idx| idx.name()).into_tuple();

        if previous_name.len() == POSTGRES_IDENTIFIER_SIZE_LIMIT && next_name.len() > POSTGRES_IDENTIFIER_SIZE_LIMIT {
            previous_name[0..POSTGRES_IDENTIFIER_SIZE_LIMIT] != next_name[0..POSTGRES_IDENTIFIER_SIZE_LIMIT]
        } else {
            previous_name != next_name
        }
    }

    fn table_should_be_ignored(&self, table_name: &str) -> bool {
        static POSTGRES_IGNORED_TABLES: Lazy<RegexSet> = Lazy::new(|| {
            RegexSet::new(&[
                // PostGIS. Reference: https://postgis.net/docs/manual-1.4/ch04.html#id418599
                "(?i)^spatial_ref_sys$",
                "(?i)^geometry_columns$",
            ])
            .unwrap()
        });

        POSTGRES_IGNORED_TABLES.is_match(table_name)
    }

    fn column_type_change(&self, differ: &ColumnDiffer<'_>) -> Option<ColumnTypeChange> {
        let native_types_enabled = self.features().contains(MigrationFeature::NativeTypes);
        let previous_family = differ.previous.column_type_family();
        let next_family = differ.next.column_type_family();
        let previous_type: Option<PostgresType> = differ.previous.column_native_type();
        let next_type: Option<PostgresType> = differ.next.column_native_type();
        let from_list_to_scalar = differ.previous.arity().is_list() && !differ.next.arity().is_list();
        let from_scalar_to_list = !differ.previous.arity().is_list() && differ.next.arity().is_list();

        if !native_types_enabled {
            match (previous_family, next_family) {
                (_, ColumnTypeFamily::String) if from_list_to_scalar => Some(ColumnTypeChange::SafeCast),
                (_, _) if from_list_to_scalar => Some(ColumnTypeChange::NotCastable),
                (ColumnTypeFamily::Decimal, ColumnTypeFamily::Decimal)
                | (ColumnTypeFamily::Float, ColumnTypeFamily::Float)
                | (ColumnTypeFamily::Decimal, ColumnTypeFamily::Float)
                | (ColumnTypeFamily::Float, ColumnTypeFamily::Decimal)
                | (ColumnTypeFamily::Binary, ColumnTypeFamily::Binary)
                    if from_scalar_to_list =>
                {
                    Some(ColumnTypeChange::NotCastable)
                }
                (previous, next) => family_change_riskyness(previous, next),
            }
        } else {
            native_type_change_riskyness(previous_type.unwrap(), next_type.unwrap())
        }
    }
}

fn family_change_riskyness(previous: &ColumnTypeFamily, next: &ColumnTypeFamily) -> Option<ColumnTypeChange> {
    match (previous, next) {
        (previous, next) if previous == next => None,
        (_, ColumnTypeFamily::String) => Some(ColumnTypeChange::SafeCast),
        (ColumnTypeFamily::String, ColumnTypeFamily::Int)
        | (ColumnTypeFamily::DateTime, ColumnTypeFamily::Float)
        | (ColumnTypeFamily::String, ColumnTypeFamily::Float) => Some(ColumnTypeChange::NotCastable),
        (_, _) => Some(ColumnTypeChange::RiskyCast),
    }
}

fn native_type_change_riskyness(previous: PostgresType, next: PostgresType) -> Option<ColumnTypeChange> {
    use ColumnTypeChange::*;

    let cast = || match previous {
        PostgresType::SmallInt => match next {
            PostgresType::SmallInt => SafeCast, //duplicate -.-
            PostgresType::Integer => SafeCast,
            PostgresType::BigInt => SafeCast,
            PostgresType::Decimal(_) => SafeCast,
            PostgresType::Numeric(_) => SafeCast,
            PostgresType::Real => SafeCast,
            PostgresType::DoublePrecision => SafeCast,
            PostgresType::VarChar(_) => SafeCast,
            PostgresType::Char(_) => SafeCast,
            PostgresType::Text => SafeCast,
            PostgresType::ByteA => NotCastable,
            PostgresType::Timestamp(_) => NotCastable,
            PostgresType::Timestamptz(_) => NotCastable,
            PostgresType::Date => NotCastable,
            PostgresType::Time(_) => NotCastable,
            PostgresType::Timetz(_) => NotCastable,
            PostgresType::Boolean => NotCastable,
            PostgresType::Bit(_) => NotCastable,
            PostgresType::VarBit(_) => NotCastable,
            PostgresType::UUID => NotCastable,
            PostgresType::Xml => NotCastable,
            PostgresType::JSON => NotCastable,
            PostgresType::JSONB => NotCastable,
        },
        PostgresType::Integer => match next {
            PostgresType::SmallInt => RiskyCast,
            PostgresType::Integer => SafeCast,
            PostgresType::BigInt => SafeCast,
            PostgresType::Decimal(_) => SafeCast,
            PostgresType::Numeric(_) => SafeCast,
            PostgresType::Real => SafeCast,
            PostgresType::DoublePrecision => SafeCast,
            PostgresType::VarChar(_) => SafeCast,
            PostgresType::Char(_) => SafeCast,
            PostgresType::Text => SafeCast,
            PostgresType::ByteA => NotCastable,
            PostgresType::Timestamp(_) => NotCastable,
            PostgresType::Timestamptz(_) => NotCastable,
            PostgresType::Date => NotCastable,
            PostgresType::Time(_) => NotCastable,
            PostgresType::Timetz(_) => NotCastable,
            PostgresType::Boolean => NotCastable,
            PostgresType::Bit(_) => NotCastable,
            PostgresType::VarBit(_) => NotCastable,
            PostgresType::UUID => NotCastable,
            PostgresType::Xml => NotCastable,
            PostgresType::JSON => NotCastable,
            PostgresType::JSONB => NotCastable,
        },
        PostgresType::BigInt => match next {
            PostgresType::SmallInt => RiskyCast,
            PostgresType::Integer => RiskyCast,
            PostgresType::BigInt => SafeCast,
            PostgresType::Decimal(_) => SafeCast,
            PostgresType::Numeric(_) => SafeCast,
            PostgresType::Real => SafeCast,
            PostgresType::DoublePrecision => SafeCast,
            PostgresType::VarChar(_) => SafeCast,
            PostgresType::Char(_) => SafeCast,
            PostgresType::Text => SafeCast,
            PostgresType::ByteA => NotCastable,
            PostgresType::Timestamp(_) => NotCastable,
            PostgresType::Timestamptz(_) => NotCastable,
            PostgresType::Date => NotCastable,
            PostgresType::Time(_) => NotCastable,
            PostgresType::Timetz(_) => NotCastable,
            PostgresType::Boolean => NotCastable,
            PostgresType::Bit(_) => NotCastable,
            PostgresType::VarBit(_) => NotCastable,
            PostgresType::UUID => NotCastable,
            PostgresType::Xml => NotCastable,
            PostgresType::JSON => NotCastable,
            PostgresType::JSONB => NotCastable,
        },
        PostgresType::Decimal(_) => match next {
            PostgresType::SmallInt => SafeCast,
            PostgresType::Integer => SafeCast,
            PostgresType::BigInt => SafeCast,
            PostgresType::Decimal(_) => SafeCast,
            PostgresType::Numeric(_) => SafeCast,
            PostgresType::Real => SafeCast,
            PostgresType::DoublePrecision => SafeCast,
            PostgresType::VarChar(_) => SafeCast,
            PostgresType::Char(_) => SafeCast,
            PostgresType::Text => SafeCast,
            PostgresType::ByteA => SafeCast,
            PostgresType::Timestamp(_) => SafeCast,
            PostgresType::Timestamptz(_) => SafeCast,
            PostgresType::Date => SafeCast,
            PostgresType::Time(_) => SafeCast,
            PostgresType::Timetz(_) => SafeCast,
            PostgresType::Boolean => SafeCast,
            PostgresType::Bit(_) => SafeCast,
            PostgresType::VarBit(_) => SafeCast,
            PostgresType::UUID => SafeCast,
            PostgresType::Xml => SafeCast,
            PostgresType::JSON => SafeCast,
            PostgresType::JSONB => SafeCast,
        },
        PostgresType::Numeric(_) => match next {
            PostgresType::SmallInt => SafeCast,
            PostgresType::Integer => SafeCast,
            PostgresType::BigInt => SafeCast,
            PostgresType::Decimal(_) => SafeCast,
            PostgresType::Numeric(_) => SafeCast,
            PostgresType::Real => SafeCast,
            PostgresType::DoublePrecision => SafeCast,
            PostgresType::VarChar(_) => SafeCast,
            PostgresType::Char(_) => SafeCast,
            PostgresType::Text => SafeCast,
            PostgresType::ByteA => SafeCast,
            PostgresType::Timestamp(_) => SafeCast,
            PostgresType::Timestamptz(_) => SafeCast,
            PostgresType::Date => SafeCast,
            PostgresType::Time(_) => SafeCast,
            PostgresType::Timetz(_) => SafeCast,
            PostgresType::Boolean => SafeCast,
            PostgresType::Bit(_) => SafeCast,
            PostgresType::VarBit(_) => SafeCast,
            PostgresType::UUID => SafeCast,
            PostgresType::Xml => SafeCast,
            PostgresType::JSON => SafeCast,
            PostgresType::JSONB => SafeCast,
        },
        PostgresType::Real => match next {
            PostgresType::SmallInt => SafeCast,
            PostgresType::Integer => SafeCast,
            PostgresType::BigInt => SafeCast,
            PostgresType::Decimal(_) => SafeCast,
            PostgresType::Numeric(_) => SafeCast,
            PostgresType::Real => SafeCast,
            PostgresType::DoublePrecision => SafeCast,
            PostgresType::VarChar(_) => SafeCast,
            PostgresType::Char(_) => SafeCast,
            PostgresType::Text => SafeCast,
            PostgresType::ByteA => SafeCast,
            PostgresType::Timestamp(_) => SafeCast,
            PostgresType::Timestamptz(_) => SafeCast,
            PostgresType::Date => SafeCast,
            PostgresType::Time(_) => SafeCast,
            PostgresType::Timetz(_) => SafeCast,
            PostgresType::Boolean => SafeCast,
            PostgresType::Bit(_) => SafeCast,
            PostgresType::VarBit(_) => SafeCast,
            PostgresType::UUID => SafeCast,
            PostgresType::Xml => SafeCast,
            PostgresType::JSON => SafeCast,
            PostgresType::JSONB => SafeCast,
        },
        PostgresType::DoublePrecision => match next {
            PostgresType::SmallInt => SafeCast,
            PostgresType::Integer => SafeCast,
            PostgresType::BigInt => SafeCast,
            PostgresType::Decimal(_) => SafeCast,
            PostgresType::Numeric(_) => SafeCast,
            PostgresType::Real => SafeCast,
            PostgresType::DoublePrecision => SafeCast,
            PostgresType::VarChar(_) => SafeCast,
            PostgresType::Char(_) => SafeCast,
            PostgresType::Text => SafeCast,
            PostgresType::ByteA => SafeCast,
            PostgresType::Timestamp(_) => SafeCast,
            PostgresType::Timestamptz(_) => SafeCast,
            PostgresType::Date => SafeCast,
            PostgresType::Time(_) => SafeCast,
            PostgresType::Timetz(_) => SafeCast,
            PostgresType::Boolean => SafeCast,
            PostgresType::Bit(_) => SafeCast,
            PostgresType::VarBit(_) => SafeCast,
            PostgresType::UUID => SafeCast,
            PostgresType::Xml => SafeCast,
            PostgresType::JSON => SafeCast,
            PostgresType::JSONB => SafeCast,
        },
        PostgresType::VarChar(_) => match next {
            PostgresType::SmallInt => SafeCast,
            PostgresType::Integer => SafeCast,
            PostgresType::BigInt => SafeCast,
            PostgresType::Decimal(_) => SafeCast,
            PostgresType::Numeric(_) => SafeCast,
            PostgresType::Real => SafeCast,
            PostgresType::DoublePrecision => SafeCast,
            PostgresType::VarChar(_) => SafeCast,
            PostgresType::Char(_) => SafeCast,
            PostgresType::Text => SafeCast,
            PostgresType::ByteA => SafeCast,
            PostgresType::Timestamp(_) => SafeCast,
            PostgresType::Timestamptz(_) => SafeCast,
            PostgresType::Date => SafeCast,
            PostgresType::Time(_) => SafeCast,
            PostgresType::Timetz(_) => SafeCast,
            PostgresType::Boolean => SafeCast,
            PostgresType::Bit(_) => SafeCast,
            PostgresType::VarBit(_) => SafeCast,
            PostgresType::UUID => SafeCast,
            PostgresType::Xml => SafeCast,
            PostgresType::JSON => SafeCast,
            PostgresType::JSONB => SafeCast,
        },
        PostgresType::Char(_) => match next {
            PostgresType::SmallInt => SafeCast,
            PostgresType::Integer => SafeCast,
            PostgresType::BigInt => SafeCast,
            PostgresType::Decimal(_) => SafeCast,
            PostgresType::Numeric(_) => SafeCast,
            PostgresType::Real => SafeCast,
            PostgresType::DoublePrecision => SafeCast,
            PostgresType::VarChar(_) => SafeCast,
            PostgresType::Char(_) => SafeCast,
            PostgresType::Text => SafeCast,
            PostgresType::ByteA => SafeCast,
            PostgresType::Timestamp(_) => SafeCast,
            PostgresType::Timestamptz(_) => SafeCast,
            PostgresType::Date => SafeCast,
            PostgresType::Time(_) => SafeCast,
            PostgresType::Timetz(_) => SafeCast,
            PostgresType::Boolean => SafeCast,
            PostgresType::Bit(_) => SafeCast,
            PostgresType::VarBit(_) => SafeCast,
            PostgresType::UUID => SafeCast,
            PostgresType::Xml => SafeCast,
            PostgresType::JSON => SafeCast,
            PostgresType::JSONB => SafeCast,
        },
        PostgresType::Text => match next {
            PostgresType::SmallInt => SafeCast,
            PostgresType::Integer => SafeCast,
            PostgresType::BigInt => SafeCast,
            PostgresType::Decimal(_) => SafeCast,
            PostgresType::Numeric(_) => SafeCast,
            PostgresType::Real => SafeCast,
            PostgresType::DoublePrecision => SafeCast,
            PostgresType::VarChar(_) => SafeCast,
            PostgresType::Char(_) => SafeCast,
            PostgresType::Text => SafeCast,
            PostgresType::ByteA => SafeCast,
            PostgresType::Timestamp(_) => SafeCast,
            PostgresType::Timestamptz(_) => SafeCast,
            PostgresType::Date => SafeCast,
            PostgresType::Time(_) => SafeCast,
            PostgresType::Timetz(_) => SafeCast,
            PostgresType::Boolean => SafeCast,
            PostgresType::Bit(_) => SafeCast,
            PostgresType::VarBit(_) => SafeCast,
            PostgresType::UUID => SafeCast,
            PostgresType::Xml => SafeCast,
            PostgresType::JSON => SafeCast,
            PostgresType::JSONB => SafeCast,
        },
        PostgresType::ByteA => match next {
            PostgresType::SmallInt => SafeCast,
            PostgresType::Integer => SafeCast,
            PostgresType::BigInt => SafeCast,
            PostgresType::Decimal(_) => SafeCast,
            PostgresType::Numeric(_) => SafeCast,
            PostgresType::Real => SafeCast,
            PostgresType::DoublePrecision => SafeCast,
            PostgresType::VarChar(_) => SafeCast,
            PostgresType::Char(_) => SafeCast,
            PostgresType::Text => SafeCast,
            PostgresType::ByteA => SafeCast,
            PostgresType::Timestamp(_) => SafeCast,
            PostgresType::Timestamptz(_) => SafeCast,
            PostgresType::Date => SafeCast,
            PostgresType::Time(_) => SafeCast,
            PostgresType::Timetz(_) => SafeCast,
            PostgresType::Boolean => SafeCast,
            PostgresType::Bit(_) => SafeCast,
            PostgresType::VarBit(_) => SafeCast,
            PostgresType::UUID => SafeCast,
            PostgresType::Xml => SafeCast,
            PostgresType::JSON => SafeCast,
            PostgresType::JSONB => SafeCast,
        },
        PostgresType::Timestamp(_) => match next {
            PostgresType::SmallInt => SafeCast,
            PostgresType::Integer => SafeCast,
            PostgresType::BigInt => SafeCast,
            PostgresType::Decimal(_) => SafeCast,
            PostgresType::Numeric(_) => SafeCast,
            PostgresType::Real => SafeCast,
            PostgresType::DoublePrecision => SafeCast,
            PostgresType::VarChar(_) => SafeCast,
            PostgresType::Char(_) => SafeCast,
            PostgresType::Text => SafeCast,
            PostgresType::ByteA => SafeCast,
            PostgresType::Timestamp(_) => SafeCast,
            PostgresType::Timestamptz(_) => SafeCast,
            PostgresType::Date => SafeCast,
            PostgresType::Time(_) => SafeCast,
            PostgresType::Timetz(_) => SafeCast,
            PostgresType::Boolean => SafeCast,
            PostgresType::Bit(_) => SafeCast,
            PostgresType::VarBit(_) => SafeCast,
            PostgresType::UUID => SafeCast,
            PostgresType::Xml => SafeCast,
            PostgresType::JSON => SafeCast,
            PostgresType::JSONB => SafeCast,
        },
        PostgresType::Timestamptz(_) => match next {
            PostgresType::SmallInt => SafeCast,
            PostgresType::Integer => SafeCast,
            PostgresType::BigInt => SafeCast,
            PostgresType::Decimal(_) => SafeCast,
            PostgresType::Numeric(_) => SafeCast,
            PostgresType::Real => SafeCast,
            PostgresType::DoublePrecision => SafeCast,
            PostgresType::VarChar(_) => SafeCast,
            PostgresType::Char(_) => SafeCast,
            PostgresType::Text => SafeCast,
            PostgresType::ByteA => SafeCast,
            PostgresType::Timestamp(_) => SafeCast,
            PostgresType::Timestamptz(_) => SafeCast,
            PostgresType::Date => SafeCast,
            PostgresType::Time(_) => SafeCast,
            PostgresType::Timetz(_) => SafeCast,
            PostgresType::Boolean => SafeCast,
            PostgresType::Bit(_) => SafeCast,
            PostgresType::VarBit(_) => SafeCast,
            PostgresType::UUID => SafeCast,
            PostgresType::Xml => SafeCast,
            PostgresType::JSON => SafeCast,
            PostgresType::JSONB => SafeCast,
        },
        PostgresType::Date => match next {
            PostgresType::SmallInt => SafeCast,
            PostgresType::Integer => SafeCast,
            PostgresType::BigInt => SafeCast,
            PostgresType::Decimal(_) => SafeCast,
            PostgresType::Numeric(_) => SafeCast,
            PostgresType::Real => SafeCast,
            PostgresType::DoublePrecision => SafeCast,
            PostgresType::VarChar(_) => SafeCast,
            PostgresType::Char(_) => SafeCast,
            PostgresType::Text => SafeCast,
            PostgresType::ByteA => SafeCast,
            PostgresType::Timestamp(_) => SafeCast,
            PostgresType::Timestamptz(_) => SafeCast,
            PostgresType::Date => SafeCast,
            PostgresType::Time(_) => SafeCast,
            PostgresType::Timetz(_) => SafeCast,
            PostgresType::Boolean => SafeCast,
            PostgresType::Bit(_) => SafeCast,
            PostgresType::VarBit(_) => SafeCast,
            PostgresType::UUID => SafeCast,
            PostgresType::Xml => SafeCast,
            PostgresType::JSON => SafeCast,
            PostgresType::JSONB => SafeCast,
        },
        PostgresType::Time(_) => match next {
            PostgresType::SmallInt => SafeCast,
            PostgresType::Integer => SafeCast,
            PostgresType::BigInt => SafeCast,
            PostgresType::Decimal(_) => SafeCast,
            PostgresType::Numeric(_) => SafeCast,
            PostgresType::Real => SafeCast,
            PostgresType::DoublePrecision => SafeCast,
            PostgresType::VarChar(_) => SafeCast,
            PostgresType::Char(_) => SafeCast,
            PostgresType::Text => SafeCast,
            PostgresType::ByteA => SafeCast,
            PostgresType::Timestamp(_) => SafeCast,
            PostgresType::Timestamptz(_) => SafeCast,
            PostgresType::Date => SafeCast,
            PostgresType::Time(_) => SafeCast,
            PostgresType::Timetz(_) => SafeCast,
            PostgresType::Boolean => SafeCast,
            PostgresType::Bit(_) => SafeCast,
            PostgresType::VarBit(_) => SafeCast,
            PostgresType::UUID => SafeCast,
            PostgresType::Xml => SafeCast,
            PostgresType::JSON => SafeCast,
            PostgresType::JSONB => SafeCast,
        },
        PostgresType::Timetz(_) => match next {
            PostgresType::SmallInt => SafeCast,
            PostgresType::Integer => SafeCast,
            PostgresType::BigInt => SafeCast,
            PostgresType::Decimal(_) => SafeCast,
            PostgresType::Numeric(_) => SafeCast,
            PostgresType::Real => SafeCast,
            PostgresType::DoublePrecision => SafeCast,
            PostgresType::VarChar(_) => SafeCast,
            PostgresType::Char(_) => SafeCast,
            PostgresType::Text => SafeCast,
            PostgresType::ByteA => SafeCast,
            PostgresType::Timestamp(_) => SafeCast,
            PostgresType::Timestamptz(_) => SafeCast,
            PostgresType::Date => SafeCast,
            PostgresType::Time(_) => SafeCast,
            PostgresType::Timetz(_) => SafeCast,
            PostgresType::Boolean => SafeCast,
            PostgresType::Bit(_) => SafeCast,
            PostgresType::VarBit(_) => SafeCast,
            PostgresType::UUID => SafeCast,
            PostgresType::Xml => SafeCast,
            PostgresType::JSON => SafeCast,
            PostgresType::JSONB => SafeCast,
        },
        PostgresType::Boolean => match next {
            PostgresType::SmallInt => SafeCast,
            PostgresType::Integer => SafeCast,
            PostgresType::BigInt => SafeCast,
            PostgresType::Decimal(_) => SafeCast,
            PostgresType::Numeric(_) => SafeCast,
            PostgresType::Real => SafeCast,
            PostgresType::DoublePrecision => SafeCast,
            PostgresType::VarChar(_) => SafeCast,
            PostgresType::Char(_) => SafeCast,
            PostgresType::Text => SafeCast,
            PostgresType::ByteA => SafeCast,
            PostgresType::Timestamp(_) => SafeCast,
            PostgresType::Timestamptz(_) => SafeCast,
            PostgresType::Date => SafeCast,
            PostgresType::Time(_) => SafeCast,
            PostgresType::Timetz(_) => SafeCast,
            PostgresType::Boolean => SafeCast,
            PostgresType::Bit(_) => SafeCast,
            PostgresType::VarBit(_) => SafeCast,
            PostgresType::UUID => SafeCast,
            PostgresType::Xml => SafeCast,
            PostgresType::JSON => SafeCast,
            PostgresType::JSONB => SafeCast,
        },
        PostgresType::Bit(_) => match next {
            PostgresType::SmallInt => SafeCast,
            PostgresType::Integer => SafeCast,
            PostgresType::BigInt => SafeCast,
            PostgresType::Decimal(_) => SafeCast,
            PostgresType::Numeric(_) => SafeCast,
            PostgresType::Real => SafeCast,
            PostgresType::DoublePrecision => SafeCast,
            PostgresType::VarChar(_) => SafeCast,
            PostgresType::Char(_) => SafeCast,
            PostgresType::Text => SafeCast,
            PostgresType::ByteA => SafeCast,
            PostgresType::Timestamp(_) => SafeCast,
            PostgresType::Timestamptz(_) => SafeCast,
            PostgresType::Date => SafeCast,
            PostgresType::Time(_) => SafeCast,
            PostgresType::Timetz(_) => SafeCast,
            PostgresType::Boolean => SafeCast,
            PostgresType::Bit(_) => SafeCast,
            PostgresType::VarBit(_) => SafeCast,
            PostgresType::UUID => SafeCast,
            PostgresType::Xml => SafeCast,
            PostgresType::JSON => SafeCast,
            PostgresType::JSONB => SafeCast,
        },
        PostgresType::VarBit(_) => match next {
            PostgresType::SmallInt => SafeCast,
            PostgresType::Integer => SafeCast,
            PostgresType::BigInt => SafeCast,
            PostgresType::Decimal(_) => SafeCast,
            PostgresType::Numeric(_) => SafeCast,
            PostgresType::Real => SafeCast,
            PostgresType::DoublePrecision => SafeCast,
            PostgresType::VarChar(_) => SafeCast,
            PostgresType::Char(_) => SafeCast,
            PostgresType::Text => SafeCast,
            PostgresType::ByteA => SafeCast,
            PostgresType::Timestamp(_) => SafeCast,
            PostgresType::Timestamptz(_) => SafeCast,
            PostgresType::Date => SafeCast,
            PostgresType::Time(_) => SafeCast,
            PostgresType::Timetz(_) => SafeCast,
            PostgresType::Boolean => SafeCast,
            PostgresType::Bit(_) => SafeCast,
            PostgresType::VarBit(_) => SafeCast,
            PostgresType::UUID => SafeCast,
            PostgresType::Xml => SafeCast,
            PostgresType::JSON => SafeCast,
            PostgresType::JSONB => SafeCast,
        },
        PostgresType::UUID => match next {
            PostgresType::SmallInt => SafeCast,
            PostgresType::Integer => SafeCast,
            PostgresType::BigInt => SafeCast,
            PostgresType::Decimal(_) => SafeCast,
            PostgresType::Numeric(_) => SafeCast,
            PostgresType::Real => SafeCast,
            PostgresType::DoublePrecision => SafeCast,
            PostgresType::VarChar(_) => SafeCast,
            PostgresType::Char(_) => SafeCast,
            PostgresType::Text => SafeCast,
            PostgresType::ByteA => SafeCast,
            PostgresType::Timestamp(_) => SafeCast,
            PostgresType::Timestamptz(_) => SafeCast,
            PostgresType::Date => SafeCast,
            PostgresType::Time(_) => SafeCast,
            PostgresType::Timetz(_) => SafeCast,
            PostgresType::Boolean => SafeCast,
            PostgresType::Bit(_) => SafeCast,
            PostgresType::VarBit(_) => SafeCast,
            PostgresType::UUID => SafeCast,
            PostgresType::Xml => SafeCast,
            PostgresType::JSON => SafeCast,
            PostgresType::JSONB => SafeCast,
        },
        PostgresType::Xml => match next {
            PostgresType::SmallInt => SafeCast,
            PostgresType::Integer => SafeCast,
            PostgresType::BigInt => SafeCast,
            PostgresType::Decimal(_) => SafeCast,
            PostgresType::Numeric(_) => SafeCast,
            PostgresType::Real => SafeCast,
            PostgresType::DoublePrecision => SafeCast,
            PostgresType::VarChar(_) => SafeCast,
            PostgresType::Char(_) => SafeCast,
            PostgresType::Text => SafeCast,
            PostgresType::ByteA => SafeCast,
            PostgresType::Timestamp(_) => SafeCast,
            PostgresType::Timestamptz(_) => SafeCast,
            PostgresType::Date => SafeCast,
            PostgresType::Time(_) => SafeCast,
            PostgresType::Timetz(_) => SafeCast,
            PostgresType::Boolean => SafeCast,
            PostgresType::Bit(_) => SafeCast,
            PostgresType::VarBit(_) => SafeCast,
            PostgresType::UUID => SafeCast,
            PostgresType::Xml => SafeCast,
            PostgresType::JSON => SafeCast,
            PostgresType::JSONB => SafeCast,
        },
        PostgresType::JSON => match next {
            PostgresType::SmallInt => SafeCast,
            PostgresType::Integer => SafeCast,
            PostgresType::BigInt => SafeCast,
            PostgresType::Decimal(_) => SafeCast,
            PostgresType::Numeric(_) => SafeCast,
            PostgresType::Real => SafeCast,
            PostgresType::DoublePrecision => SafeCast,
            PostgresType::VarChar(_) => SafeCast,
            PostgresType::Char(_) => SafeCast,
            PostgresType::Text => SafeCast,
            PostgresType::ByteA => SafeCast,
            PostgresType::Timestamp(_) => SafeCast,
            PostgresType::Timestamptz(_) => SafeCast,
            PostgresType::Date => SafeCast,
            PostgresType::Time(_) => SafeCast,
            PostgresType::Timetz(_) => SafeCast,
            PostgresType::Boolean => SafeCast,
            PostgresType::Bit(_) => SafeCast,
            PostgresType::VarBit(_) => SafeCast,
            PostgresType::UUID => SafeCast,
            PostgresType::Xml => SafeCast,
            PostgresType::JSON => SafeCast,
            PostgresType::JSONB => SafeCast,
        },
        PostgresType::JSONB => match next {
            PostgresType::SmallInt => SafeCast,
            PostgresType::Integer => SafeCast,
            PostgresType::BigInt => SafeCast,
            PostgresType::Decimal(_) => SafeCast,
            PostgresType::Numeric(_) => SafeCast,
            PostgresType::Real => SafeCast,
            PostgresType::DoublePrecision => SafeCast,
            PostgresType::VarChar(_) => SafeCast,
            PostgresType::Char(_) => SafeCast,
            PostgresType::Text => SafeCast,
            PostgresType::ByteA => SafeCast,
            PostgresType::Timestamp(_) => SafeCast,
            PostgresType::Timestamptz(_) => SafeCast,
            PostgresType::Date => SafeCast,
            PostgresType::Time(_) => SafeCast,
            PostgresType::Timetz(_) => SafeCast,
            PostgresType::Boolean => SafeCast,
            PostgresType::Bit(_) => SafeCast,
            PostgresType::VarBit(_) => SafeCast,
            PostgresType::UUID => SafeCast,
            PostgresType::Xml => SafeCast,
            PostgresType::JSON => SafeCast,
            PostgresType::JSONB => SafeCast,
        },
    };

    if previous == next {
        None
    } else {
        Some(cast())
    }
}
