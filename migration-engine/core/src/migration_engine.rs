use crate::{
    migration::{datamodel_calculator::*, datamodel_migration_steps_inferrer::*},
    CoreResult,
};
use datamodel::ast::SchemaAst;
use migration_connector::*;
use std::sync::Arc;

pub struct MigrationEngine<C, D>
where
    C: MigrationConnector<DatabaseMigration = D>,
    D: DatabaseMigrationMarker + 'static,
{
    datamodel_migration_steps_inferrer: Arc<dyn DataModelMigrationStepsInferrer>,
    datamodel_calculator: Arc<dyn DataModelCalculator>,
    connector: C,
}

impl<C, D> MigrationEngine<C, D>
where
    C: MigrationConnector<DatabaseMigration = D>,
    D: DatabaseMigrationMarker + Send + Sync + 'static,
{
    pub async fn new(connector: C) -> CoreResult<Self> {
        let engine = MigrationEngine {
            datamodel_migration_steps_inferrer: Arc::new(DataModelMigrationStepsInferrerImplWrapper {}),
            datamodel_calculator: Arc::new(DataModelCalculatorImpl),
            connector,
        };

        Ok(engine)
    }

    pub fn connector(&self) -> &C {
        &self.connector
    }

    pub fn datamodel_migration_steps_inferrer(&self) -> &Arc<dyn DataModelMigrationStepsInferrer> {
        &self.datamodel_migration_steps_inferrer
    }

    pub fn datamodel_calculator(&self) -> &Arc<dyn DataModelCalculator> {
        &self.datamodel_calculator
    }

    pub fn render_schema_ast(&self, schema_ast: &SchemaAst) -> String {
        datamodel::render_schema_ast_to_string(&schema_ast)
    }
}
