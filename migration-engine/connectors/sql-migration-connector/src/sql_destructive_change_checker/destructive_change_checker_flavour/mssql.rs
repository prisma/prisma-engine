use super::DestructiveChangeCheckerFlavour;
use crate::{
    flavour::{MssqlFlavour, SqlFlavour},
    pair::Pair,
    sql_destructive_change_checker::{
        destructive_check_plan::DestructiveCheckPlan, unexecutable_step_check::UnexecutableStepCheck,
        warning_check::SqlMigrationWarningCheck,
    },
    sql_migration::{AlterColumn, ColumnTypeChange},
    sql_schema_differ::ColumnChanges,
};
use datamodel_connector::Connector;
use migration_connector::MigrationFeature;
use sql_datamodel_connector::SqlDatamodelConnectors;
use sql_schema_describer::{walkers::ColumnWalker, DefaultKind, DefaultValue};

impl DestructiveChangeCheckerFlavour for MssqlFlavour {
    fn check_alter_column(
        &self,
        alter_column: &AlterColumn,
        columns: &Pair<ColumnWalker<'_>>,
        plan: &mut DestructiveCheckPlan,
        step_index: usize,
    ) {
        let AlterColumn {
            column_index: _,
            changes,
            type_change,
        } = alter_column;

        if changes.only_default_changed() {
            return;
        }

        if changes.arity_changed() && columns.next().arity().is_required() {
            plan.push_unexecutable(
                UnexecutableStepCheck::MadeOptionalFieldRequired {
                    column: columns.previous().name().to_owned(),
                    table: columns.previous().table().name().to_owned(),
                },
                step_index,
            );

            return;
        }

        match type_change {
            Some(ColumnTypeChange::SafeCast) | None => (),
            Some(ColumnTypeChange::RiskyCast) => {
                let native_types_enabled = self.features().contains(MigrationFeature::NativeTypes);
                let datamodel_connector = SqlDatamodelConnectors::mssql();
                let previous_type = match &columns.previous().column_type().native_type {
                    Some(tpe) if native_types_enabled => datamodel_connector.render_native_type(tpe.clone()),
                    _ => format!("{:?}", columns.previous().column_type_family()),
                };

                let next_type = match &columns.next().column_type().native_type {
                    Some(tpe) if native_types_enabled => datamodel_connector.render_native_type(tpe.clone()),
                    _ => format!("{:?}", columns.next().column_type_family()),
                };

                plan.push_warning(
                    SqlMigrationWarningCheck::RiskyCast {
                        table: columns.previous().table().name().to_owned(),
                        column: columns.previous().name().to_owned(),
                        previous_type,
                        next_type,
                    },
                    step_index,
                );
            }
            Some(ColumnTypeChange::NotCastable) => {
                plan.push_warning(
                    SqlMigrationWarningCheck::NotCastable {
                        table: columns.previous().table().name().to_owned(),
                        column: columns.previous().name().to_owned(),
                        previous_type: format!("{:?}", columns.previous().column_type_family()),
                        next_type: format!("{:?}", columns.next().column_type_family()),
                    },
                    step_index,
                );
            }
        };
    }

    fn check_drop_and_recreate_column(
        &self,
        columns: &Pair<ColumnWalker<'_>>,
        changes: &ColumnChanges,
        plan: &mut DestructiveCheckPlan,
        step_index: usize,
    ) {
        // Unexecutable drop and recreate.
        if changes.arity_changed()
            && columns.previous().arity().is_nullable()
            && columns.next().arity().is_required()
            && !default_can_be_rendered(columns.next().default())
        {
            plan.push_unexecutable(
                UnexecutableStepCheck::AddedRequiredFieldToTable {
                    column: columns.previous().name().to_owned(),
                    table: columns.previous().table().name().to_owned(),
                },
                step_index,
            )
        } else if columns.next().arity().is_required() && columns.next().default().is_none() {
            plan.push_unexecutable(
                UnexecutableStepCheck::DropAndRecreateRequiredColumn {
                    column: columns.previous().name().to_owned(),
                    table: columns.previous().table().name().to_owned(),
                },
                step_index,
            )
        } else {
            plan.push_warning(
                SqlMigrationWarningCheck::DropAndRecreateColumn {
                    column: columns.previous().name().to_owned(),
                    table: columns.previous().table().name().to_owned(),
                },
                step_index,
            )
        }
    }
}

fn default_can_be_rendered(default: Option<&DefaultValue>) -> bool {
    match default.as_ref().map(|d| d.kind()) {
        None => false,
        Some(DefaultKind::VALUE(_)) => true,
        Some(DefaultKind::DBGENERATED(expr)) => !expr.is_empty(),
        Some(DefaultKind::NOW) => true,
        Some(DefaultKind::SEQUENCE(_)) => false,
    }
}
