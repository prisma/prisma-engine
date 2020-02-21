use crate::*;
use sql_renderer::{
    mysql_quoted, mysql_quoted_string, postgres_quoted, postgres_quoted_string, postgres_render_column_type,
    IteratorJoin, SqlRenderer,
};
use sql_schema_describer::*;
use sql_schema_helpers::{walk_columns, ColumnRef};
use std::fmt::Write as _;
use tracing_futures::Instrument;

pub struct SqlDatabaseStepApplier<'a> {
    pub connector: &'a crate::SqlMigrationConnector,
}

impl crate::component::Component for SqlDatabaseStepApplier<'_> {
    fn connector(&self) -> &crate::SqlMigrationConnector {
        self.connector
    }
}

#[async_trait::async_trait]
impl DatabaseMigrationStepApplier<SqlMigration> for SqlDatabaseStepApplier<'_> {
    async fn apply_step(&self, database_migration: &SqlMigration, index: usize) -> ConnectorResult<bool> {
        let renderer = self.renderer();
        let fut = self
            .apply_next_step(
                &database_migration.corrected_steps,
                index,
                renderer.as_ref(),
                &database_migration.before,
                &database_migration.after,
            )
            .instrument(tracing::debug_span!("ApplySqlStep", index));

        crate::catch(self.connection_info(), fut).await
    }

    async fn unapply_step(&self, database_migration: &SqlMigration, index: usize) -> ConnectorResult<bool> {
        let renderer = self.renderer();
        let fut = self
            .apply_next_step(
                &database_migration.rollback,
                index,
                renderer.as_ref(),
                &database_migration.after,
                &database_migration.before,
            )
            .instrument(tracing::debug_span!("UnapplySqlStep", index));

        crate::catch(self.connection_info(), fut).await
    }

    fn render_steps_pretty(&self, database_migration: &SqlMigration) -> ConnectorResult<Vec<serde_json::Value>> {
        render_steps_pretty(
            &database_migration,
            self.renderer().as_ref(),
            self.database_info(),
            &database_migration.before,
            &database_migration.after,
        )?
        .into_iter()
        .map(|pretty_step| {
            serde_json::to_value(&pretty_step)
                .map_err(|err| ConnectorError::from_kind(migration_connector::ErrorKind::Generic(err.into())))
        })
        .collect()
    }
}

impl SqlDatabaseStepApplier<'_> {
    async fn apply_next_step(
        &self,
        steps: &[SqlMigrationStep],
        index: usize,
        renderer: &(dyn SqlRenderer + Send + Sync),
        current_schema: &SqlSchema,
        next_schema: &SqlSchema,
    ) -> SqlResult<bool> {
        let has_this_one = steps.get(index).is_some();
        if !has_this_one {
            return Ok(false);
        }

        let step = &steps[index];
        tracing::debug!(?step);

        for sql_string in render_raw_sql(&step, renderer, self.database_info(), current_schema, next_schema)
            .map_err(|err: anyhow::Error| SqlError::Generic(err))?
        {
            tracing::debug!(index, %sql_string);

            let result = self.conn().query_raw(&sql_string, &[]).await;

            // TODO: this does not evaluate the results of SQLites PRAGMA foreign_key_check
            result?;
        }

        let has_more = steps.get(index + 1).is_some();
        Ok(has_more)
    }

    fn renderer<'a>(&'a self) -> Box<dyn SqlRenderer + Send + Sync + 'a> {
        SqlRenderer::for_family(&self.sql_family())
    }
}

fn render_steps_pretty(
    database_migration: &SqlMigration,
    renderer: &(dyn SqlRenderer + Send + Sync),
    database_info: &DatabaseInfo,
    current_schema: &SqlSchema,
    next_schema: &SqlSchema,
) -> ConnectorResult<Vec<PrettySqlMigrationStep>> {
    let mut steps = Vec::with_capacity(database_migration.corrected_steps.len());

    for step in &database_migration.corrected_steps {
        let sql = render_raw_sql(&step, renderer, database_info, current_schema, next_schema)
            .map_err(|err: anyhow::Error| {
                ConnectorError::from_kind(migration_connector::ErrorKind::Generic(err.into()))
            })?
            .join(";\n");

        if !sql.is_empty() {
            steps.push(PrettySqlMigrationStep {
                step: step.clone(),
                raw: sql,
            });
        }
    }

    Ok(steps)
}

fn render_raw_sql(
    step: &SqlMigrationStep,
    renderer: &(dyn SqlRenderer + Send + Sync),
    database_info: &DatabaseInfo,
    current_schema: &SqlSchema,
    next_schema: &SqlSchema,
) -> Result<Vec<String>, anyhow::Error> {
    let sql_family = renderer.sql_family();
    let schema_name = database_info.connection_info().schema_name().to_string();

    match step {
        SqlMigrationStep::CreateEnum(create_enum) => render_create_enum(renderer, create_enum),
        SqlMigrationStep::DropEnum(drop_enum) => render_drop_enum(renderer, drop_enum),
        SqlMigrationStep::AlterEnum(alter_enum) => match renderer.sql_family() {
            SqlFamily::Postgres => postgres_alter_enum(alter_enum, next_schema, &schema_name),
            SqlFamily::Mysql => mysql_alter_enum(alter_enum, next_schema, &schema_name),
            _ => Ok(Vec::new()),
        },
        SqlMigrationStep::CreateTable(CreateTable { table }) => {
            let mut create_table = String::with_capacity(100);

            write!(create_table, "CREATE TABLE ")?;
            renderer.write_quoted_with_schema(&mut create_table, &schema_name, &table.name)?;
            writeln!(create_table, " (")?;

            let mut columns = table.columns.iter().peekable();
            while let Some(column) = columns.next() {
                let column = ColumnRef {
                    schema: next_schema,
                    column,
                    table,
                };
                let col_sql = renderer.render_column(&schema_name, column, false);

                write!(
                    create_table,
                    "    {}{}",
                    col_sql,
                    if columns.peek().is_some() { ",\n" } else { "" }
                )?;
            }

            let primary_key_is_already_set = create_table.contains("PRIMARY KEY");
            let primary_columns = table.primary_key_columns();

            if primary_columns.len() > 0 && !primary_key_is_already_set {
                let column_names: Vec<String> = primary_columns
                    .clone()
                    .into_iter()
                    .map(|col| renderer.quote(&col))
                    .collect();
                write!(create_table, ",\n    PRIMARY KEY ({})", column_names.join(","))?;
            }

            if sql_family == SqlFamily::Sqlite && !table.foreign_keys.is_empty() {
                write!(create_table, ",")?;

                let mut fks = table.foreign_keys.iter().peekable();

                while let Some(fk) = fks.next() {
                    write!(
                        create_table,
                        "FOREIGN KEY ({constrained_columns}) {references}{comma}",
                        constrained_columns = fk.columns.iter().map(|col| format!(r#""{}""#, col)).join(","),
                        references = renderer.render_references(&schema_name, fk),
                        comma = if fks.peek().is_some() { ",\n" } else { "" },
                    )?;
                }
            }

            write!(create_table, "\n) {}", create_table_suffix(sql_family))?;

            Ok(vec![create_table])
        }
        SqlMigrationStep::DropTable(DropTable { name }) => Ok(vec![format!(
            "DROP TABLE {};",
            renderer.quote_with_schema(&schema_name, &name)
        )]),
        SqlMigrationStep::DropTables(DropTables { names }) => {
            let fully_qualified_names: Vec<String> = names
                .iter()
                .map(|name| renderer.quote_with_schema(&schema_name, &name))
                .collect();
            Ok(vec![format!("DROP TABLE {};", fully_qualified_names.join(","))])
        }
        SqlMigrationStep::RenameTable { name, new_name } => {
            let new_name = match sql_family {
                SqlFamily::Sqlite => renderer.quote(new_name),
                _ => renderer.quote_with_schema(&schema_name, &new_name),
            };
            Ok(vec![format!(
                "ALTER TABLE {} RENAME TO {};",
                renderer.quote_with_schema(&schema_name, &name),
                new_name
            )])
        }
        SqlMigrationStep::AddForeignKey(AddForeignKey { table, foreign_key }) => match sql_family {
            SqlFamily::Sqlite => Ok(Vec::new()),
            _ => {
                let mut add_constraint = String::with_capacity(120);

                write!(
                    add_constraint,
                    "ALTER TABLE {table} ADD ",
                    table = renderer.quote_with_schema(&schema_name, table)
                )?;

                if let Some(constraint_name) = foreign_key.constraint_name.as_ref() {
                    write!(add_constraint, "CONSTRAINT {} ", renderer.quote(constraint_name))?;
                }

                write!(
                    add_constraint,
                    "FOREIGN KEY ({})",
                    foreign_key.columns.iter().map(|col| renderer.quote(col)).join(", ")
                )?;

                add_constraint.push_str(&renderer.render_references(&schema_name, &foreign_key));

                Ok(vec![add_constraint])
            }
        },
        SqlMigrationStep::AlterTable(AlterTable { table, changes }) => {
            let mut lines = Vec::new();
            for change in changes {
                match change {
                    TableChange::AddColumn(AddColumn { column }) => {
                        let column = ColumnRef {
                            table,
                            schema: next_schema,
                            column,
                        };
                        let col_sql = renderer.render_column(&schema_name, column, true);
                        lines.push(format!("ADD COLUMN {}", col_sql));
                    }
                    TableChange::DropColumn(DropColumn { name }) => {
                        let name = renderer.quote(&name);
                        lines.push(format!("DROP COLUMN {}", name));
                    }
                    TableChange::AlterColumn(AlterColumn { name, column }) => {
                        match safe_alter_column(
                            renderer,
                            current_schema.get_table(&table.name).unwrap().column(&name).unwrap(),
                            &column,
                        ) {
                            Some(safe_sql) => {
                                for line in safe_sql {
                                    lines.push(line)
                                }
                            }
                            None => {
                                let name = renderer.quote(&name);
                                lines.push(format!("DROP COLUMN {}", name));
                                let column = ColumnRef {
                                    schema: next_schema,
                                    table,
                                    column,
                                };
                                let col_sql = renderer.render_column(&schema_name, column, true);
                                lines.push(format!("ADD COLUMN {}", col_sql));
                            }
                        }
                    }
                    TableChange::DropForeignKey(DropForeignKey { constraint_name }) => match sql_family {
                        SqlFamily::Mysql => {
                            let constraint_name = renderer.quote(&constraint_name);
                            lines.push(format!("DROP FOREIGN KEY {}", constraint_name));
                        }
                        SqlFamily::Postgres => {
                            let constraint_name = renderer.quote(&constraint_name);
                            lines.push(format!("DROP CONSTRAINT IF EXiSTS {}", constraint_name));
                        }
                        _ => (),
                    },
                };
            }
            Ok(vec![format!(
                "ALTER TABLE {} {};",
                renderer.quote_with_schema(&schema_name, &table.name),
                lines.join(",\n")
            )])
        }
        SqlMigrationStep::CreateIndex(CreateIndex { table, index }) => {
            Ok(vec![render_create_index(renderer, database_info, table, index)])
        }
        SqlMigrationStep::DropIndex(DropIndex { table, name }) => match sql_family {
            SqlFamily::Mysql => Ok(vec![format!(
                "DROP INDEX {} ON {}",
                renderer.quote(&name),
                renderer.quote_with_schema(&schema_name, &table),
            )]),
            SqlFamily::Postgres | SqlFamily::Sqlite => Ok(vec![format!(
                "DROP INDEX {}",
                renderer.quote_with_schema(&schema_name, &name)
            )]),
        },
        SqlMigrationStep::AlterIndex(AlterIndex {
            table,
            index_name,
            index_new_name,
        }) => match sql_family {
            SqlFamily::Mysql => {
                // MariaDB does not support `ALTER TABLE ... RENAME INDEX`.
                if database_info.is_mariadb() {
                    let old_index = current_schema
                        .table(table)
                        .map_err(|_| {
                            anyhow::anyhow!(
                                "Invariant violation: could not find table `{}` in current schema.",
                                table
                            )
                        })?
                        .indices
                        .iter()
                        .find(|idx| idx.name.as_str() == index_name)
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "Invariant violation: could not find index `{}` on table `{}` in current schema.",
                                index_name,
                                table
                            )
                        })?;
                    let mut new_index = old_index.clone();
                    new_index.name = index_new_name.clone();

                    // Order matters: dropping the old index first wouldn't work when foreign key constraints are still relying on it.
                    Ok(vec![
                        render_create_index(renderer, database_info, table, &new_index),
                        mysql_drop_index(renderer, &schema_name, table, index_name)?,
                    ])
                } else {
                    Ok(vec![format!(
                        "ALTER TABLE {table_name} RENAME INDEX {index_name} TO {index_new_name}",
                        table_name = renderer.quote_with_schema(&schema_name, &table),
                        index_name = renderer.quote(index_name),
                        index_new_name = renderer.quote(index_new_name)
                    )])
                }
            }
            SqlFamily::Postgres => Ok(vec![format!(
                "ALTER INDEX {} RENAME TO {}",
                renderer.quote_with_schema(&schema_name, index_name),
                renderer.quote(index_new_name)
            )]),
            SqlFamily::Sqlite => unimplemented!("Index renaming on SQLite."),
        },
        SqlMigrationStep::RawSql { raw } => Ok(vec![raw.to_owned()]),
    }
}

fn render_create_index(
    renderer: &dyn SqlRenderer,
    database_info: &DatabaseInfo,
    table_name: &str,
    index: &Index,
) -> String {
    let Index { name, columns, tpe } = index;
    let index_type = match tpe {
        IndexType::Unique => "UNIQUE",
        IndexType::Normal => "",
    };
    let sql_family = database_info.sql_family();
    let index_name = match sql_family {
        SqlFamily::Sqlite => renderer.quote_with_schema(database_info.connection_info().schema_name(), &name),
        _ => renderer.quote(&name),
    };
    let table_reference = match sql_family {
        SqlFamily::Sqlite => renderer.quote(table_name),
        _ => renderer.quote_with_schema(database_info.connection_info().schema_name(), table_name),
    };
    let columns: Vec<String> = columns.iter().map(|c| renderer.quote(c)).collect();

    format!(
        "CREATE {} INDEX {} ON {}({})",
        index_type,
        index_name,
        table_reference,
        columns.join(",")
    )
}

fn mysql_drop_index(
    renderer: &dyn SqlRenderer,
    schema_name: &str,
    table_name: &str,
    index_name: &str,
) -> Result<String, std::fmt::Error> {
    let mut drop_index = String::with_capacity(24 + table_name.len() + index_name.len());
    write!(drop_index, "DROP INDEX ")?;
    renderer.write_quoted(&mut drop_index, index_name)?;
    write!(drop_index, " ON ")?;
    renderer.write_quoted_with_schema(&mut drop_index, &schema_name, table_name)?;

    Ok(drop_index)
}

fn create_table_suffix(sql_family: SqlFamily) -> &'static str {
    match sql_family {
        SqlFamily::Sqlite => "",
        SqlFamily::Postgres => "",
        SqlFamily::Mysql => "\nDEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci",
    }
}

fn safe_alter_column(
    renderer: &dyn SqlRenderer,
    previous_column: &Column,
    next_column: &Column,
) -> Option<Vec<String>> {
    use crate::sql_migration::expanded_alter_column::*;

    let expanded = crate::sql_migration::expanded_alter_column::expand_alter_column(
        previous_column,
        next_column,
        &renderer.sql_family(),
    )?;

    let alter_column_prefix = format!("ALTER COLUMN {}", renderer.quote(&previous_column.name));

    let steps = match expanded {
        ExpandedAlterColumn::Postgres(steps) => steps
            .into_iter()
            .map(|step| match step {
                PostgresAlterColumn::DropDefault => format!("{} DROP DEFAULT", &alter_column_prefix),
                PostgresAlterColumn::SetDefault(new_default) => {
                    format!("{} SET DEFAULT '{}'", &alter_column_prefix, new_default)
                }
                PostgresAlterColumn::DropNotNull => format!("{} DROP NOT NULL", &alter_column_prefix),
                PostgresAlterColumn::SetType(ty) => format!(
                    "{} SET DATA TYPE {}",
                    &alter_column_prefix,
                    postgres_render_column_type(&ty)
                ),
            })
            .collect(),
        ExpandedAlterColumn::Mysql(steps) => steps
            .into_iter()
            .map(|step| match step {
                MysqlAlterColumn::DropDefault => format!("{} DROP DEFAULT", &alter_column_prefix),
                MysqlAlterColumn::SetDefault(new_default) => {
                    format!("{} SET DEFAULT '{}'", &alter_column_prefix, new_default)
                }
            })
            .collect(),
        ExpandedAlterColumn::Sqlite(_steps) => vec![],
    };

    Some(steps)
}

fn render_create_enum(
    renderer: &(dyn SqlRenderer + Send + Sync),
    create_enum: &CreateEnum,
) -> Result<Vec<String>, anyhow::Error> {
    match renderer.sql_family() {
        SqlFamily::Postgres => {
            let sql = format!(
                r#"CREATE TYPE {enum_name} AS ENUM ({variants});"#,
                enum_name = sql_renderer::postgres_quoted(&create_enum.name),
                variants = create_enum
                    .variants
                    .iter()
                    .map(sql_renderer::postgres_quoted_string)
                    .join(", "),
            );
            Ok(vec![sql])
        }
        _ => Ok(Vec::new()),
    }
}

fn render_drop_enum(
    renderer: &(dyn SqlRenderer + Send + Sync),
    drop_enum: &DropEnum,
) -> Result<Vec<String>, anyhow::Error> {
    match renderer.sql_family() {
        SqlFamily::Postgres => {
            let sql = format!(
                "DROP TYPE {enum_name}",
                enum_name = sql_renderer::postgres_quoted(&drop_enum.name),
            );

            Ok(vec![sql])
        }
        _ => Ok(Vec::new()),
    }
}

fn postgres_alter_enum(
    alter_enum: &AlterEnum,
    next_schema: &SqlSchema,
    schema_name: &str,
) -> anyhow::Result<Vec<String>> {
    if alter_enum.dropped_variants.is_empty() {
        let stmts: Vec<String> = alter_enum
            .created_variants
            .iter()
            .map(|created_value| {
                format!(
                    "ALTER TYPE {enum_name} ADD VALUE {value}",
                    enum_name = postgres_quoted(&alter_enum.name),
                    value = postgres_quoted_string(created_value)
                )
            })
            .collect();

        Ok(stmts)
    } else {
        let new_enum = next_schema
            .get_enum(&alter_enum.name)
            .ok_or_else(|| anyhow::anyhow!("Enum `{}` not found in target schema.", alter_enum.name))?;

        let mut stmts = Vec::with_capacity(8);

        let tmp_name = format!("{}_new", &new_enum.name);
        let tmp_old_name = format!("{}_old", &alter_enum.name);

        // create the new enum with tmp name
        {
            let create_new_enum = format!(
                "CREATE TYPE {enum_name} AS ENUM ({variants})",
                enum_name = postgres_quoted(&tmp_name),
                variants = new_enum.values.iter().map(postgres_quoted_string).join(", ")
            );

            stmts.push(create_new_enum);
        }

        // alter type of the current columns to new, with a cast
        {
            let affected_columns = walk_columns(next_schema).filter(|column| match &column.column_type().family {
                ColumnTypeFamily::Enum(name) if name.as_str() == alter_enum.name.as_str() => true,
                _ => false,
            });

            for column in affected_columns {
                let sql = format!(
                    "ALTER TABLE {schema_name}.{table_name} \
                        ALTER COLUMN {column_name} DROP DEFAULT,
                        ALTER COLUMN {column_name} TYPE {tmp_name} \
                            USING ({column_name}::text::{tmp_name}),
                        ALTER COLUMN {column_name} SET DEFAULT {new_enum_default}",
                    schema_name = postgres_quoted(schema_name),
                    table_name = postgres_quoted(column.table().name()),
                    column_name = postgres_quoted(column.name()),
                    tmp_name = postgres_quoted(&tmp_name),
                    new_enum_default = postgres_quoted_string(new_enum.values.first().unwrap()),
                );

                stmts.push(sql);
            }
        }

        // rename old enum
        {
            let sql = format!(
                "ALTER TYPE {enum_name} RENAME TO {tmp_old_name}",
                enum_name = postgres_quoted(&alter_enum.name),
                tmp_old_name = postgres_quoted(&tmp_old_name)
            );

            stmts.push(sql);
        }

        // rename new enum
        {
            let sql = format!(
                "ALTER TYPE {tmp_name} RENAME TO {enum_name}",
                tmp_name = postgres_quoted(&tmp_name),
                enum_name = postgres_quoted(&new_enum.name)
            );

            stmts.push(sql)
        }

        // drop old enum
        {
            let sql = format!(
                "DROP TYPE {tmp_old_name}",
                tmp_old_name = postgres_quoted(&tmp_old_name),
            );

            stmts.push(sql)
        }

        Ok(stmts)
    }
}

fn mysql_alter_enum(alter_enum: &AlterEnum, next_schema: &SqlSchema, schema_name: &str) -> anyhow::Result<Vec<String>> {
    let column = sql_schema_helpers::walk_columns(next_schema)
        .find(|col| match &col.column_type().family {
            ColumnTypeFamily::Enum(enum_name) if enum_name.as_str() == alter_enum.name.as_str() => true,
            _ => false,
        })
        .ok_or_else(|| anyhow::anyhow!("Could not find column to alter for {:?}", alter_enum))?;
    let enum_variants = next_schema
        .get_enum(&alter_enum.name)
        .ok_or_else(|| anyhow::anyhow!("Couldn't find enum {:?}", alter_enum.name))?
        .values
        .iter()
        .map(mysql_quoted_string)
        .join(", ");

    let change_column = format!(
        "ALTER TABLE {schema_name}.{table_name} CHANGE {column_name} {column_name} ENUM({enum_variants})",
        schema_name = mysql_quoted(schema_name),
        table_name = mysql_quoted(column.table().name()),
        column_name = column.name(),
        enum_variants = enum_variants,
    );

    Ok(vec![change_column])
}
