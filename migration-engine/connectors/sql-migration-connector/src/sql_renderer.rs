mod common;
mod mysql_renderer;
mod postgres_renderer;
mod sqlite_renderer;

pub(crate) use common::{IteratorJoin, Quoted, QuotedWithSchema};

use crate::{
    database_info::DatabaseInfo,
    sql_migration::{
        AddColumn, AddForeignKey, AlterColumn, AlterEnum, AlterIndex, AlterTable, CreateEnum, CreateIndex, DropColumn,
        DropEnum, DropForeignKey, DropIndex, TableChange,
    },
    sql_schema_differ::{ColumnDiffer, SqlSchemaDiffer},
};
use quaint::prelude::SqlFamily;
use sql_schema_describer::walkers::{ColumnWalker, TableWalker};
use sql_schema_describer::*;
use std::{borrow::Cow, fmt::Write as _};

pub(crate) trait SqlRenderer {
    fn quote<'a>(&self, name: &'a str) -> Quoted<&'a str>;

    fn quote_with_schema<'a, 'b>(&'a self, schema_name: &'a str, name: &'b str) -> QuotedWithSchema<'a, &'b str> {
        QuotedWithSchema {
            schema_name,
            name: self.quote(name),
        }
    }

    fn render_add_foreign_key(&self, add_foreign_key: &AddForeignKey, schema_name: &str) -> String {
        let AddForeignKey { foreign_key, table } = add_foreign_key;
        let mut add_constraint = String::with_capacity(120);

        write!(
            add_constraint,
            "ALTER TABLE {table} ADD ",
            table = self.quote_with_schema(&schema_name, table)
        )
        .unwrap();

        if let Some(constraint_name) = foreign_key.constraint_name.as_ref() {
            write!(add_constraint, "CONSTRAINT {} ", self.quote(constraint_name)).unwrap();
        }

        write!(
            add_constraint,
            "FOREIGN KEY ({})",
            foreign_key.columns.iter().map(|col| self.quote(col)).join(", ")
        )
        .unwrap();

        add_constraint.push_str(&self.render_references(&foreign_key));

        add_constraint
    }

    fn render_alter_enum(&self, alter_enum: &AlterEnum, differ: &SqlSchemaDiffer<'_>) -> anyhow::Result<Vec<String>>;

    fn render_column(&self, column: ColumnWalker<'_>) -> String;

    fn render_references(&self, foreign_key: &ForeignKey) -> String;

    fn render_default<'a>(&self, default: &'a DefaultValue, family: &ColumnTypeFamily) -> Cow<'a, str>;

    /// Attempt to render a database-specific ALTER COLUMN based on the
    /// passed-in differ. `None` means that we could not generate a good (set
    /// of) ALTER COLUMN(s), and we should fall back to dropping and recreating
    /// the column.
    fn render_alter_column(&self, differ: &ColumnDiffer<'_>) -> Option<RenderedAlterColumn>;

    /// Render an `AlterIndex` step.
    fn render_alter_index(
        &self,
        alter_index: &AlterIndex,
        database_info: &DatabaseInfo,
        current_schema: &SqlSchema,
    ) -> anyhow::Result<Vec<String>>;

    fn render_alter_table(
        &self,
        alter_table: &AlterTable,
        database_info: &DatabaseInfo,
        differ: &SqlSchemaDiffer<'_>,
    ) -> Vec<String> {
        let AlterTable { table, changes } = alter_table;
        let schema_name = database_info.connection_info().schema_name();

        let mut lines = Vec::new();
        let mut before_statements = Vec::new();
        let mut after_statements = Vec::new();

        for change in changes {
            match change {
                TableChange::DropPrimaryKey { constraint_name } => match database_info.sql_family() {
                    SqlFamily::Mysql => lines.push("DROP PRIMARY KEY".to_owned()),
                    SqlFamily::Postgres => lines.push(format!(
                        "DROP CONSTRAINT {}",
                        Quoted::postgres_ident(
                            constraint_name
                                .as_ref()
                                .expect("Missing constraint name for DROP CONSTRAINT on Postgres.")
                        )
                    )),
                    _ => (),
                },
                TableChange::AddPrimaryKey { columns } => lines.push(format!(
                    "ADD PRIMARY KEY ({})",
                    columns.iter().map(|colname| self.quote(colname)).join(", ")
                )),
                TableChange::AddColumn(AddColumn { column }) => {
                    let column = ColumnWalker {
                        table,
                        schema: differ.next,
                        column,
                    };
                    let col_sql = self.render_column(column);
                    lines.push(format!("ADD COLUMN {}", col_sql));
                }
                TableChange::DropColumn(DropColumn { name }) => {
                    let name = self.quote(&name);
                    lines.push(format!("DROP COLUMN {}", name));
                }
                TableChange::AlterColumn(AlterColumn { name, column: _ }) => {
                    let column = differ
                        .diff_table(&table.name)
                        .expect("AlterTable on unknown table.")
                        .diff_column(name)
                        .expect("AlterColumn on unknown column.");
                    match self.render_alter_column(&column) {
                        Some(RenderedAlterColumn {
                            alter_columns,
                            before,
                            after,
                        }) => {
                            for statement in alter_columns {
                                lines.push(statement);
                            }

                            if let Some(before) = before {
                                before_statements.push(before);
                            }

                            if let Some(after) = after {
                                after_statements.push(after);
                            }
                        }
                        None => {
                            let name = self.quote(&name);
                            lines.push(format!("DROP COLUMN {}", name));

                            let col_sql = self.render_column(column.next);
                            lines.push(format!("ADD COLUMN {}", col_sql));
                        }
                    }
                }
            };
        }

        if lines.is_empty() {
            return Vec::new();
        }

        let alter_table = format!(
            "ALTER TABLE {} {}",
            self.quote_with_schema(&schema_name, &table.name),
            lines.join(",\n")
        );

        let statements = before_statements
            .into_iter()
            .chain(std::iter::once(alter_table))
            .chain(after_statements.into_iter())
            .collect();

        statements
    }

    /// Render a `CreateEnum` step.
    fn render_create_enum(&self, create_enum: &CreateEnum) -> Vec<String>;

    /// Render a `CreateIndex` step.
    fn render_create_index(&self, create_index: &CreateIndex, database_info: &DatabaseInfo) -> String;

    /// Render a `CreateTable` step.
    fn render_create_table(&self, table: &TableWalker<'_>) -> anyhow::Result<String>;

    /// Render a `DropEnum` step.
    fn render_drop_enum(&self, drop_enum: &DropEnum) -> Vec<String>;

    /// Render a `DropForeignKey` step.
    fn render_drop_foreign_key(&self, drop_foreign_key: &DropForeignKey) -> String;

    /// Render a `DropIndex` step.
    fn render_drop_index(&self, drop_index: &DropIndex, database_info: &DatabaseInfo) -> String;

    /// Render a `DropTable` step.
    fn render_drop_table(&self, table_name: &str, schema_name: &str) -> Vec<String> {
        vec![format!(
            "DROP TABLE {}",
            self.quote_with_schema(&schema_name, &table_name)
        )]
    }

    /// Render a `RedefineTables` step.
    fn render_redefine_tables(
        &self,
        tables: &[String],
        differ: SqlSchemaDiffer<'_>,
        database_info: &DatabaseInfo,
    ) -> Vec<String>;

    fn render_rename_table(&self, name: &str, new_name: &str) -> String;
}

#[derive(Default)]
pub(crate) struct RenderedAlterColumn {
    /// The statements that will be included in the ALTER TABLE
    pub(crate) alter_columns: Vec<String>,
    /// The statements to be run before the ALTER TABLE.
    pub(crate) before: Option<String>,
    /// The statements to be run after the ALTER TABLE.
    pub(crate) after: Option<String>,
}
