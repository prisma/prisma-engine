pub(crate) mod rendered_step;

mod common;
mod mysql_renderer;
mod postgres_renderer;
mod sqlite_renderer;

pub(crate) use common::{IteratorJoin, Quoted, QuotedWithSchema};

use crate::{sql_schema_differ::ColumnDiffer, sql_schema_helpers::ColumnRef, CreateEnum, DropEnum};
use quaint::prelude::SqlFamily;
use sql_schema_describer::*;
use std::{borrow::Cow, fmt::Write};

pub(crate) trait SqlRenderer {
    fn quote<'a>(&self, name: &'a str) -> Quoted<&'a str>;

    fn quote_with_schema<'a, 'b>(&'a self, schema_name: &'a str, name: &'b str) -> QuotedWithSchema<'a, &'b str> {
        QuotedWithSchema {
            schema_name,
            name: self.quote(name),
        }
    }

    fn render_column(&self, schema_name: &str, column: ColumnRef<'_>, add_fk_prefix: bool) -> String;

    fn render_references(&self, schema_name: &str, foreign_key: &ForeignKey) -> String;

    fn render_default<'a>(&self, default: &'a DefaultValue, family: &ColumnTypeFamily) -> Cow<'a, str>;

    /// Attempt to render a database-specific ALTER COLUMN based on the
    /// passed-in differ. `None` means that we could not generate a good (set
    /// of) ALTER COLUMN(s), and we should fall back to dropping and recreating
    /// the column.
    fn render_alter_column(&self, differ: &ColumnDiffer<'_>) -> Option<RenderedAlterColumn>;

    /// Render a `CreateEnum` step.
    fn render_create_enum(&self, create_enum: &CreateEnum) -> Vec<String>;

    /// Render a `CreateTable` step.
    fn render_create_table(
        &self,
        table: &Table,
        schema_name: &str,
        next_schema: &SqlSchema,
        sql_family: SqlFamily,
    ) -> anyhow::Result<Vec<String>> {
        let columns: String = table
            .columns
            .iter()
            .map(|column| {
                // FIXME Temporary hack: we should get this from a `TableRef`, but
                // this is not possible because we sometimes create tables as
                // part of the table redifinition process on sqlite.
                let column = ColumnRef {
                    schema: next_schema,
                    column,
                    table,
                };
                self.render_column(&schema_name, column, false)
            })
            .join(",\n");

        let mut create_table = format!(
            "CREATE TABLE {} (\n{}",
            self.quote_with_schema(&schema_name, &table.name),
            columns,
        );

        let primary_key_is_already_set = create_table.contains("PRIMARY KEY");
        let primary_columns = table.primary_key_columns();

        if !primary_columns.is_empty() && !primary_key_is_already_set {
            let column_names = primary_columns.iter().map(|col| self.quote(&col)).join(",");
            write!(create_table, ",\nPRIMARY KEY ({})", column_names)?;
        }

        if sql_family == SqlFamily::Mysql && !table.indices.is_empty() {
            let indices: String = table
                .indices
                .iter()
                .map(|index| {
                    let tpe = if index.is_unique() { "UNIQUE " } else { "" };
                    format!(
                        "{}Index {}({})",
                        tpe,
                        self.quote(&index.name),
                        index.columns.iter().map(|col| self.quote(&col)).join(",\n")
                    )
                })
                .join(",\n");

            write!(create_table, ",\n{}", indices)?;
        }

        if sql_family == SqlFamily::Sqlite && !table.foreign_keys.is_empty() {
            writeln!(create_table, ",")?;

            let mut fks = table.foreign_keys.iter().peekable();

            while let Some(fk) = fks.next() {
                writeln!(
                    create_table,
                    "FOREIGN KEY ({constrained_columns}) {references}{comma}",
                    constrained_columns = fk.columns.iter().map(|col| format!(r#""{}""#, col)).join(","),
                    references = self.render_references(&schema_name, fk),
                    comma = if fks.peek().is_some() { ",\n" } else { "" },
                )?;
            }
        }

        create_table.push_str(create_table_suffix(sql_family));

        Ok(vec![create_table])
    }

    /// Render a `DropEnum` step.
    fn render_drop_enum(&self, drop_enum: &DropEnum) -> Vec<String>;
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

fn create_table_suffix(sql_family: SqlFamily) -> &'static str {
    match sql_family {
        SqlFamily::Sqlite => ")",
        SqlFamily::Postgres => ")",
        SqlFamily::Mysql => "\n) DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci",
        SqlFamily::Mssql => todo!("Greetings from Redmond"),
    }
}
