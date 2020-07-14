use crate::commenting_out_guardrails::commenting_out_guardrails;
use crate::introspection::introspect;
use crate::misc_helpers::*;
use crate::prisma_1_defaults::*;
use crate::sanitize_datamodel_names::sanitize_datamodel_names;
use crate::version_checker::VersionChecker;
use crate::SqlIntrospectionResult;
use datamodel::Datamodel;
use introspection_connector::{IntrospectionResult, Warning};
use quaint::connector::SqlFamily;
use sql_schema_describer::*;
use tracing::debug;

/// Calculate a data model from a database schema.
pub fn calculate_datamodel(schema: &SqlSchema, family: &SqlFamily) -> SqlIntrospectionResult<IntrospectionResult> {
    debug!("Calculating data model.");

    let mut version_check = VersionChecker::new(family.clone(), schema);
    let mut data_model = Datamodel::new();

    // 1to1 translation of the sql schema
    introspect(schema, &mut version_check, &mut data_model)?;

    // our opinionation about valid names
    sanitize_datamodel_names(&mut data_model);

    // commenting out models, fields, enums, enum values
    let mut warnings: Vec<Warning> = commenting_out_guardrails(&mut data_model);

    // deduplicating relation field names
    deduplicate_relation_field_names(&mut data_model);

    // try to identify whether the schema was created by a previous Prisma version
    let version = version_check.version(&warnings, &data_model);

    // if based on a previous Prisma version add id default opinionations
    add_prisma_1_id_defaults(family, &version, &mut data_model, schema, &mut warnings);

    // renderer -> parser -> validator, is_commented_out gets lost between renderer and parser
    debug!("Done calculating data model {:?}", data_model);
    Ok(IntrospectionResult {
        data_model,
        version,
        warnings,
    })
}
