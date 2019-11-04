use crate::commands::command::*;
use crate::migration_engine::MigrationEngine;
use datamodel::ast::SchemaAst;
use log::*;
use migration_connector::*;
use serde::{Deserialize, Serialize};

pub struct CalculateDatamodelCommand<'a> {
    input: &'a CalculateDatamodelInput,
}

impl<'a> MigrationCommand<'a> for CalculateDatamodelCommand<'a> {
    type Input = CalculateDatamodelInput;
    type Output = CalculateDatamodelOutput;

    fn new(input: &'a Self::Input) -> Box<Self> {
        Box::new(CalculateDatamodelCommand { input })
    }

    fn execute<C, D>(&self, engine: &MigrationEngine<C, D>) -> CommandResult<Self::Output>
    where
        C: MigrationConnector<DatabaseMigration = D>,
        D: DatabaseMigrationMarker + 'static,
    {
        debug!("{:?}", self.input);

        let base_datamodel = SchemaAst::empty();
        let datamodel = engine.datamodel_calculator().infer(&base_datamodel, &self.input.steps);

        Ok(CalculateDatamodelOutput {
            datamodel: datamodel::render_schema_ast_to_string(&datamodel).unwrap(),
        })
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CalculateDatamodelInput {
    pub steps: Vec<MigrationStep>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalculateDatamodelOutput {
    pub datamodel: String,
}
