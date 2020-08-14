mod check;
mod database_inspection_results;
mod destructive_change_checker_flavour;
mod destructive_check_plan;
mod unexecutable_step_check;
mod warning_check;

pub(crate) use destructive_change_checker_flavour::DestructiveChangeCheckerFlavour;

use crate::{
    sql_schema_differ::{ColumnDiffer, TableDiffer},
    Component, DropTable, SqlMigration, SqlMigrationStep, SqlResult, TableChange,
};
use destructive_check_plan::DestructiveCheckPlan;
use migration_connector::{ConnectorResult, DestructiveChangeChecker, DestructiveChangeDiagnostics};
use sql_schema_describer::{
    walkers::{find_column, ColumnWalker, SqlSchemaExt},
    SqlSchema,
};
use unexecutable_step_check::UnexecutableStepCheck;
use warning_check::SqlMigrationWarningCheck;

/// The SqlDestructiveChangeChecker is responsible for informing users about potentially
/// destructive or impossible changes that their attempted migrations contain.
///
/// It proceeds in three steps:
///
/// - Examine the SqlMigrationSteps in the migration, to generate a `DestructiveCheckPlan`
///   containing destructive change checks (implementors of the `Check` trait). At this stage, there
///   is no interaction with the database.
/// - Execute that plan (`DestructiveCheckPlan::execute`), running queries against the database to
///   inspect its current state, depending on what information the checks require.
/// - Render the final user-facing messages based on the plan and the gathered information.
pub struct SqlDestructiveChangeChecker<'a> {
    pub connector: &'a crate::SqlMigrationConnector,
}

impl Component for SqlDestructiveChangeChecker<'_> {
    fn connector(&self) -> &crate::SqlMigrationConnector {
        self.connector
    }
}

impl SqlDestructiveChangeChecker<'_> {
    fn check_table_drop(&self, table_name: &str, plan: &mut DestructiveCheckPlan) {
        plan.push_warning(SqlMigrationWarningCheck::NonEmptyTableDrop {
            table: table_name.to_owned(),
        });
    }

    /// Emit a warning when we drop a column that contains non-null values.
    fn check_column_drop(&self, column: &ColumnWalker<'_>, plan: &mut DestructiveCheckPlan) {
        plan.push_warning(SqlMigrationWarningCheck::NonEmptyColumnDrop {
            table: column.table().name().to_owned(),
            column: column.name().to_owned(),
        });
    }

    /// Columns cannot be added when all of the following holds:
    ///
    /// - There are existing rows
    /// - The new column is required
    /// - There is no default value for the new column
    fn check_add_column(&self, column: &ColumnWalker<'_>, plan: &mut DestructiveCheckPlan) {
        let column_is_required_without_default = column.is_required() && column.default().is_none();

        // Optional columns and columns with a default can safely be added.
        if !column_is_required_without_default {
            return;
        }

        let typed_unexecutable = UnexecutableStepCheck::AddedRequiredFieldToTable {
            column: column.name().to_owned(),
            table: column.table().name().to_owned(),
        };

        plan.push_unexecutable(typed_unexecutable);
    }

    fn plan(&self, steps: &[SqlMigrationStep], before: &SqlSchema, after: &SqlSchema) -> DestructiveCheckPlan {
        let mut plan = DestructiveCheckPlan::new();

        for step in steps {
            match step {
                SqlMigrationStep::AlterTable(alter_table) => {
                    // The table in alter_table is the updated table, but we want to
                    // check against the current state of the table.
                    let before_table = before.table_walker(&alter_table.table.name);
                    let after_table = after.table_walker(&alter_table.table.name);

                    if let (Some(before_table), Some(after_table)) = (before_table, after_table) {
                        for change in &alter_table.changes {
                            match *change {
                                TableChange::DropColumn(ref drop_column) => {
                                    let column = find_column(before, &alter_table.table.name, &drop_column.name)
                                        .expect("Dropping of unknown column.");

                                    self.check_column_drop(&column, &mut plan);
                                }
                                TableChange::AlterColumn(ref alter_column) => {
                                    let previous_column = before_table
                                        .column(&alter_column.name)
                                        .expect("unsupported column renaming");
                                    let next_column = after_table
                                        .column(&alter_column.name)
                                        .expect("unsupported column renaming");

                                    let differ = ColumnDiffer {
                                        database_info: self.database_info(),
                                        previous: previous_column,
                                        next: next_column,
                                        flavour: self.flavour(),
                                    };

                                    self.flavour().check_alter_column(&differ, &mut plan)
                                }
                                TableChange::AddColumn(ref add_column) => {
                                    let column = find_column(after, after_table.name(), &add_column.column.name)
                                        .expect("Could not find column in AddColumn");

                                    self.check_add_column(&column, &mut plan)
                                }
                                TableChange::DropPrimaryKey { .. } => {
                                    plan.push_warning(SqlMigrationWarningCheck::PrimaryKeyChange {
                                        table: alter_table.table.name.clone(),
                                    })
                                }
                                _ => (),
                            }
                        }
                    }
                }
                SqlMigrationStep::RedefineTables { names } => {
                    for name in names {
                        let previous = before.table_walker(&name).expect("Redefining unknown table.");
                        let next = after.table_walker(&name).expect("Redefining unknown table.");
                        let differ = TableDiffer {
                            database_info: self.database_info(),
                            flavour: self.flavour(),
                            previous,
                            next,
                        };

                        if let Some(_) = differ.dropped_primary_key() {
                            plan.push_warning(SqlMigrationWarningCheck::PrimaryKeyChange { table: name.clone() })
                        }

                        for added_column in differ.added_columns() {
                            self.check_add_column(&added_column, &mut plan);
                        }

                        for dropped_column in differ.dropped_columns() {
                            self.check_column_drop(&dropped_column, &mut plan);
                        }

                        for columns in differ.column_pairs() {
                            self.flavour().check_alter_column(&columns, &mut plan);
                        }
                    }
                }
                SqlMigrationStep::DropTable(DropTable { name }) => {
                    self.check_table_drop(name, &mut plan);
                }
                // SqlMigrationStep::CreateIndex(CreateIndex { table, index }) if index.is_unique() => todo!(),
                // do nothing
                _ => (),
            }
        }

        plan
    }

    #[tracing::instrument(skip(self, steps, before), target = "SqlDestructiveChangeChecker::check")]
    async fn check_impl(
        &self,
        steps: &[SqlMigrationStep],
        before: &SqlSchema,
        after: &SqlSchema,
    ) -> SqlResult<DestructiveChangeDiagnostics> {
        let plan = self.plan(steps, before, after);

        plan.execute(self.schema_name(), self.conn()).await
    }
}

#[async_trait::async_trait]
impl DestructiveChangeChecker<SqlMigration> for SqlDestructiveChangeChecker<'_> {
    async fn check(&self, database_migration: &SqlMigration) -> ConnectorResult<DestructiveChangeDiagnostics> {
        self.check_impl(
            &database_migration.steps,
            &database_migration.before,
            &database_migration.after,
        )
        .await
        .map_err(|sql_error| sql_error.into_connector_error(&self.connection_info()))
    }

    fn pure_check(&self, database_migration: &SqlMigration) -> ConnectorResult<DestructiveChangeDiagnostics> {
        let plan = self.plan(
            &database_migration.steps,
            &database_migration.before,
            &database_migration.after,
        );

        Ok(plan.pure_check())
    }
}
