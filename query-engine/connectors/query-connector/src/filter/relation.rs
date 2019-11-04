use crate::compare::RelationCompare;
use crate::filter::Filter;
use prisma_models::RelationField;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct RelationFilter {
    pub field: Arc<RelationField>,
    pub nested_filter: Box<Filter>,
    pub condition: RelationCondition,
}

#[derive(Debug, Clone)]
pub struct OneRelationIsNullFilter {
    pub field: Arc<RelationField>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RelationCondition {
    EveryRelatedRecord,
    AtLeastOneRelatedRecord,
    NoRelatedRecord,
    ToOneRelatedRecord, // TODO: This is needed for Mongo and should be discussed with Matthias
}

impl RelationCondition {
    pub fn invert_of_subselect(self) -> bool {
        match self {
            RelationCondition::EveryRelatedRecord => true,
            _ => false,
        }
    }
}

impl RelationCompare for Arc<RelationField> {
    /// Every related record matches the filter.
    /// ```rust
    /// # use query_connector::{*, filter::*};
    /// # use prisma_models::*;
    /// # use quaint::ast::*;
    /// # use serde_json;
    /// # use std::{fs::File, sync::Arc};
    /// #
    /// # let tmp: InternalDataModelTemplate = serde_json::from_reader(File::open("../sql-query-connector/test_schema.json").unwrap()).unwrap();
    /// # let schema = tmp.build(String::from("test"));
    /// # let user = schema.find_model("User").unwrap();
    /// # let site = schema.find_model("Site").unwrap();
    /// #
    /// let rel_field = user.fields().find_from_relation_fields("sites").unwrap();
    /// let site_name = site.fields().find_from_scalar("name").unwrap();
    /// let filter = rel_field.every_related(site_name.equals("Blog"));
    ///
    /// match filter {
    ///     Filter::Relation(RelationFilter {
    ///         field: relation_field,
    ///         nested_filter: nested,
    ///         condition: condition,
    ///     }) => {
    ///         assert_eq!(String::from("sites"), relation_field.name);
    ///         assert_eq!(RelationCondition::EveryRelatedRecord, condition);
    ///
    ///         match *nested {
    ///             Filter::Scalar(ScalarFilter {
    ///                 field: scalar_field,
    ///                 condition: ScalarCondition::Equals(scalar_val),
    ///             }) => {
    ///                 assert_eq!(String::from("name"), scalar_field.name);
    ///                 assert_eq!(PrismaValue::from("Blog"), scalar_val);
    ///             }
    ///             _ => unreachable!()
    ///         }
    ///     }
    ///     _ => unreachable!()
    /// }
    /// ```
    fn every_related<T>(&self, filter: T) -> Filter
    where
        T: Into<Filter>,
    {
        Filter::from(RelationFilter {
            field: Arc::clone(self),
            nested_filter: Box::new(filter.into()),
            condition: RelationCondition::EveryRelatedRecord,
        })
    }

    /// At least one related record matches the filter.
    /// ```rust
    /// # use query_connector::{*, filter::*};
    /// # use prisma_models::*;
    /// # use quaint::ast::*;
    /// # use serde_json;
    /// # use std::{fs::File, sync::Arc};
    /// #
    /// # let tmp: InternalDataModelTemplate = serde_json::from_reader(File::open("../sql-query-connector/test_schema.json").unwrap()).unwrap();
    /// # let schema = tmp.build(String::from("test"));
    /// # let user = schema.find_model("User").unwrap();
    /// # let site = schema.find_model("Site").unwrap();
    /// #
    /// let rel_field = user.fields().find_from_relation_fields("sites").unwrap();
    /// let site_name = site.fields().find_from_scalar("name").unwrap();
    /// let filter = rel_field.at_least_one_related(site_name.equals("Blog"));
    ///
    /// match filter {
    ///     Filter::Relation(RelationFilter {
    ///         field: relation_field,
    ///         nested_filter: nested,
    ///         condition: condition,
    ///     }) => {
    ///         assert_eq!(String::from("sites"), relation_field.name);
    ///         assert_eq!(RelationCondition::AtLeastOneRelatedRecord, condition);
    ///
    ///         match *nested {
    ///             Filter::Scalar(ScalarFilter {
    ///                 field: scalar_field,
    ///                 condition: ScalarCondition::Equals(scalar_val),
    ///             }) => {
    ///                 assert_eq!(String::from("name"), scalar_field.name);
    ///                 assert_eq!(PrismaValue::from("Blog"), scalar_val);
    ///             }
    ///             _ => unreachable!()
    ///         }
    ///     }
    ///     _ => unreachable!()
    /// }
    /// ```
    fn at_least_one_related<T>(&self, filter: T) -> Filter
    where
        T: Into<Filter>,
    {
        Filter::from(RelationFilter {
            field: Arc::clone(self),
            nested_filter: Box::new(filter.into()),
            condition: RelationCondition::AtLeastOneRelatedRecord,
        })
    }

    /// To one related record. FIXME
    /// ```rust
    /// # use query_connector::{*, filter::*};
    /// # use prisma_models::*;
    /// # use quaint::ast::*;
    /// # use serde_json;
    /// # use std::{fs::File, sync::Arc};
    /// #
    /// # let tmp: InternalDataModelTemplate = serde_json::from_reader(File::open("../sql-query-connector/test_schema.json").unwrap()).unwrap();
    /// # let schema = tmp.build(String::from("test"));
    /// # let user = schema.find_model("User").unwrap();
    /// # let site = schema.find_model("Site").unwrap();
    /// #
    /// let rel_field = user.fields().find_from_relation_fields("sites").unwrap();
    /// let site_name = site.fields().find_from_scalar("name").unwrap();
    /// let filter = rel_field.to_one_related(site_name.equals("Blog"));
    ///
    /// match filter {
    ///     Filter::Relation(RelationFilter {
    ///         field: relation_field,
    ///         nested_filter: nested,
    ///         condition: condition,
    ///     }) => {
    ///         assert_eq!(String::from("sites"), relation_field.name);
    ///         assert_eq!(RelationCondition::ToOneRelatedRecord, condition);
    ///
    ///         match *nested {
    ///             Filter::Scalar(ScalarFilter {
    ///                 field: scalar_field,
    ///                 condition: ScalarCondition::Equals(scalar_val),
    ///             }) => {
    ///                 assert_eq!(String::from("name"), scalar_field.name);
    ///                 assert_eq!(PrismaValue::from("Blog"), scalar_val);
    ///             }
    ///             _ => unreachable!()
    ///         }
    ///     }
    ///     _ => unreachable!()
    /// }
    /// ```
    fn to_one_related<T>(&self, filter: T) -> Filter
    where
        T: Into<Filter>,
    {
        Filter::from(RelationFilter {
            field: Arc::clone(self),
            nested_filter: Box::new(filter.into()),
            condition: RelationCondition::ToOneRelatedRecord,
        })
    }

    /// None of the related records matches the filter.
    /// ```rust
    /// # use query_connector::{*, filter::*};
    /// # use prisma_models::*;
    /// # use quaint::ast::*;
    /// # use serde_json;
    /// # use std::{fs::File, sync::Arc};
    /// #
    /// # let tmp: InternalDataModelTemplate = serde_json::from_reader(File::open("../sql-query-connector/test_schema.json").unwrap()).unwrap();
    /// # let schema = tmp.build(String::from("test"));
    /// # let user = schema.find_model("User").unwrap();
    /// # let site = schema.find_model("Site").unwrap();
    /// #
    /// let rel_field = user.fields().find_from_relation_fields("sites").unwrap();
    /// let site_name = site.fields().find_from_scalar("name").unwrap();
    /// let filter = rel_field.no_related(site_name.equals("Blog"));
    ///
    /// match filter {
    ///     Filter::Relation(RelationFilter {
    ///         field: relation_field,
    ///         nested_filter: nested,
    ///         condition: condition,
    ///     }) => {
    ///         assert_eq!(String::from("sites"), relation_field.name);
    ///         assert_eq!(RelationCondition::NoRelatedRecord, condition);
    ///
    ///         match *nested {
    ///             Filter::Scalar(ScalarFilter {
    ///                 field: scalar_field,
    ///                 condition: ScalarCondition::Equals(scalar_val),
    ///             }) => {
    ///                 assert_eq!(String::from("name"), scalar_field.name);
    ///                 assert_eq!(PrismaValue::from("Blog"), scalar_val);
    ///             }
    ///             _ => unreachable!()
    ///         }
    ///     }
    ///     _ => unreachable!()
    /// }
    /// ```
    fn no_related<T>(&self, filter: T) -> Filter
    where
        T: Into<Filter>,
    {
        Filter::from(RelationFilter {
            field: Arc::clone(self),
            nested_filter: Box::new(filter.into()),
            condition: RelationCondition::NoRelatedRecord,
        })
    }

    /// One of the relations is `Null`.
    /// ```rust
    /// # use query_connector::{*, filter::*};
    /// # use prisma_models::*;
    /// # use quaint::ast::*;
    /// # use serde_json;
    /// # use std::{fs::File, sync::Arc};
    /// #
    /// # let tmp: InternalDataModelTemplate = serde_json::from_reader(File::open("../sql-query-connector/test_schema.json").unwrap()).unwrap();
    /// # let schema = tmp.build(String::from("test"));
    /// # let user = schema.find_model("User").unwrap();
    /// #
    /// let rel_field = user.fields().find_from_relation_fields("sites").unwrap();
    /// let filter = rel_field.one_relation_is_null();
    ///
    /// match filter {
    ///     Filter::OneRelationIsNull(OneRelationIsNullFilter { field }) =>
    ///         assert_eq!(String::from("sites"), field.name),
    ///     _ => unreachable!()
    /// };
    /// ```
    fn one_relation_is_null(&self) -> Filter {
        Filter::from(OneRelationIsNullFilter {
            field: Arc::clone(self),
        })
    }
}
