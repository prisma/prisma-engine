use super::{
    common::{render_nullability, Quoted, SQL_INDENTATION},
    IteratorJoin, SqlRenderer,
};
use crate::{
    flavour::{MysqlFlavour, MYSQL_IDENTIFIER_SIZE_LIMIT},
    pair::Pair,
    sql_migration::{AddColumn, AlterColumn, AlterEnum, AlterTable, DropColumn, RedefineTable, TableChange},
    sql_schema_differ::ColumnChanges,
};
use native_types::MySqlType;
use once_cell::sync::Lazy;
use prisma_value::PrismaValue;
use regex::Regex;
use sql_ddl::mysql as ddl;
use sql_schema_describer::{
    walkers::{ColumnWalker, EnumWalker, ForeignKeyWalker, IndexWalker, TableWalker, ViewWalker},
    ColumnTypeFamily, DefaultKind, DefaultValue, ForeignKeyAction, SqlSchema,
};
use std::borrow::Cow;

impl MysqlFlavour {
    fn render_column(&self, column: &ColumnWalker<'_>) -> String {
        let column_name = self.quote(column.name());
        let tpe_str = render_column_type(&column);
        let nullability_str = render_nullability(&column);
        let default_str = column
            .default()
            .filter(|default| {
                !matches!(default.kind(),  DefaultKind::SEQUENCE(_))
                    // We do not want to render JSON defaults because they are not supported by MySQL.
                    && !matches!(column.column_type_family(), ColumnTypeFamily::Json)
                    // We do not want to render binary defaults because they are not supported by MySQL.
                    && !matches!(column.column_type_family(), ColumnTypeFamily::Binary)
            })
            .map(|default| format!(" DEFAULT {}", render_default(column, default)))
            .unwrap_or_else(String::new);
        let foreign_key = column.table().foreign_key_for_column(column.name());
        let auto_increment_str = if column.is_autoincrement() {
            " AUTO_INCREMENT"
        } else {
            ""
        };

        match foreign_key {
            Some(_) => format!(
                "{}{} {}{}{}",
                SQL_INDENTATION, column_name, tpe_str, nullability_str, default_str
            ),
            None => format!(
                "{}{} {}{}{}{}",
                SQL_INDENTATION, column_name, tpe_str, nullability_str, default_str, auto_increment_str
            ),
        }
    }
}

impl SqlRenderer for MysqlFlavour {
    fn quote<'a>(&self, name: &'a str) -> Quoted<&'a str> {
        Quoted::Backticks(name)
    }

    fn render_add_foreign_key(&self, foreign_key: &ForeignKeyWalker<'_>) -> String {
        ddl::AlterTable {
            table_name: foreign_key.table().name().into(),
            changes: vec![ddl::AlterTableClause::AddForeignKey(ddl::ForeignKey {
                constraint_name: foreign_key.constraint_name().map(From::from),
                constrained_columns: foreign_key
                    .constrained_column_names()
                    .iter()
                    .map(|c| Cow::Borrowed(c.as_str()))
                    .collect(),
                referenced_table: foreign_key.referenced_table().name().into(),
                referenced_columns: foreign_key
                    .referenced_column_names()
                    .iter()
                    .map(String::as_str)
                    .map(Cow::Borrowed)
                    .collect(),
                on_delete: Some(match foreign_key.on_delete_action() {
                    ForeignKeyAction::Cascade => ddl::ForeignKeyAction::Cascade,
                    ForeignKeyAction::NoAction => ddl::ForeignKeyAction::DoNothing,
                    ForeignKeyAction::Restrict => ddl::ForeignKeyAction::Restrict,
                    ForeignKeyAction::SetDefault => ddl::ForeignKeyAction::SetDefault,
                    ForeignKeyAction::SetNull => ddl::ForeignKeyAction::SetNull,
                }),
                on_update: Some(match foreign_key.on_update_action() {
                    ForeignKeyAction::Cascade => ddl::ForeignKeyAction::Cascade,
                    ForeignKeyAction::NoAction => ddl::ForeignKeyAction::DoNothing,
                    ForeignKeyAction::Restrict => ddl::ForeignKeyAction::Restrict,
                    ForeignKeyAction::SetDefault => ddl::ForeignKeyAction::SetDefault,
                    ForeignKeyAction::SetNull => ddl::ForeignKeyAction::SetNull,
                }),
            })],
        }
        .to_string()
    }

    fn render_alter_enum(&self, _alter_enum: &AlterEnum, _differ: &Pair<&SqlSchema>) -> Vec<String> {
        unreachable!("render_alter_enum on MySQL")
    }

    fn render_alter_index(&self, indexes: Pair<&IndexWalker<'_>>) -> Vec<String> {
        vec![ddl::AlterTable {
            table_name: indexes.previous().table().name().into(),
            changes: vec![sql_ddl::mysql::AlterTableClause::RenameIndex {
                previous_name: indexes.previous().name().into(),
                next_name: indexes.next().name().into(),
            }],
        }
        .to_string()]
    }

    fn render_alter_table(&self, alter_table: &AlterTable, schemas: &Pair<&SqlSchema>) -> Vec<String> {
        let AlterTable { table_index, changes } = alter_table;

        let tables = schemas.tables(table_index);

        let mut lines = Vec::new();

        for change in changes {
            match change {
                TableChange::DropPrimaryKey => lines.push(sql_ddl::mysql::AlterTableClause::DropPrimaryKey.to_string()),
                TableChange::AddPrimaryKey { columns } => lines.push(format!(
                    "ADD PRIMARY KEY ({})",
                    columns.iter().map(|colname| self.quote(colname)).join(", ")
                )),
                TableChange::AddColumn(AddColumn { column_index }) => {
                    let column = tables.next().column_at(*column_index);
                    let col_sql = self.render_column(&column);

                    lines.push(format!("ADD COLUMN {}", col_sql));
                }
                TableChange::DropColumn(DropColumn { index }) => lines.push(
                    sql_ddl::mysql::AlterTableClause::DropColumn {
                        column_name: tables.previous().column_at(*index).name().into(),
                    }
                    .to_string(),
                ),
                TableChange::AlterColumn(AlterColumn {
                    changes,
                    column_index,
                    type_change: _,
                }) => {
                    let columns = tables.columns(column_index);
                    let expanded = MysqlAlterColumn::new(&columns, &changes);

                    match expanded {
                        MysqlAlterColumn::DropDefault => lines.push(format!(
                            "ALTER COLUMN {column} DROP DEFAULT",
                            column = Quoted::mysql_ident(columns.previous().name())
                        )),
                        MysqlAlterColumn::Modify { new_default, changes } => {
                            lines.push(render_mysql_modify(&changes, new_default.as_ref(), columns.next()))
                        }
                    };
                }
                TableChange::DropAndRecreateColumn {
                    column_index,
                    changes: _,
                } => {
                    let columns = tables.columns(column_index);
                    lines.push(format!("DROP COLUMN `{}`", columns.previous().name()));
                    lines.push(format!("ADD COLUMN {}", self.render_column(columns.next())));
                }
            };
        }

        if lines.is_empty() {
            return Vec::new();
        }

        vec![format!(
            "ALTER TABLE {} {}",
            self.quote(tables.previous().name()),
            lines.join(",\n    ")
        )]
    }

    fn render_create_enum(&self, _create_enum: &EnumWalker<'_>) -> Vec<String> {
        Vec::new() // enums are defined on each column that uses them on MySQL
    }

    fn render_create_index(&self, index: &IndexWalker<'_>) -> String {
        let name = index.name();
        let name = if name.len() > MYSQL_IDENTIFIER_SIZE_LIMIT {
            &name[0..MYSQL_IDENTIFIER_SIZE_LIMIT]
        } else {
            &name
        };

        ddl::CreateIndex {
            unique: index.index_type().is_unique(),
            index_name: name.into(),
            on: (
                index.table().name().into(),
                index.columns().map(|c| c.name().into()).collect(),
            ),
        }
        .to_string()
    }

    fn render_create_table_as(&self, table: &TableWalker<'_>, table_name: &str) -> String {
        let columns: String = table.columns().map(|column| self.render_column(&column)).join(",\n");

        let primary_columns = table.primary_key_column_names();

        let primary_key = if let Some(primary_columns) = primary_columns.as_ref().filter(|cols| !cols.is_empty()) {
            let column_names = primary_columns.iter().map(|col| self.quote(&col)).join(",");
            format!(",\n\n{}PRIMARY KEY ({})", SQL_INDENTATION, column_names)
        } else {
            String::new()
        };

        let indexes = if table.indexes().next().is_some() {
            let indices: String = table
                .indexes()
                .map(|index| {
                    let tpe = if index.index_type().is_unique() { "UNIQUE " } else { "" };
                    let index_name = if index.name().len() > MYSQL_IDENTIFIER_SIZE_LIMIT {
                        &index.name()[0..MYSQL_IDENTIFIER_SIZE_LIMIT]
                    } else {
                        &index.name()
                    };

                    format!(
                        "{}INDEX {}({})",
                        tpe,
                        self.quote(&index_name),
                        index.columns().map(|col| self.quote(col.name())).join(", ")
                    )
                })
                .join(",\n");

            format!(",\n{}", indices)
        } else {
            String::new()
        };

        format!(
            "CREATE TABLE {} (\n{columns}{indexes}{primary_key}\n) DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci",
            table_name = self.quote(table_name),
            columns = columns,
            indexes = indexes,
            primary_key = primary_key,
        )
    }

    fn render_drop_and_recreate_index(&self, indexes: Pair<&IndexWalker<'_>>) -> Vec<String> {
        // Order matters: dropping the old index first wouldn't work when foreign key constraints are still relying on it.
        vec![
            self.render_create_index(indexes.next()),
            sql_ddl::mysql::DropIndex {
                index_name: indexes.previous().name().into(),
                table_name: indexes.previous().table().name().into(),
            }
            .to_string(),
        ]
    }

    fn render_drop_enum(&self, _: &EnumWalker<'_>) -> Vec<String> {
        Vec::new()
    }

    fn render_drop_foreign_key(&self, foreign_key: &ForeignKeyWalker<'_>) -> String {
        format!(
            "ALTER TABLE {table} DROP FOREIGN KEY {constraint_name}",
            table = self.quote(foreign_key.table().name()),
            constraint_name = Quoted::mysql_ident(foreign_key.constraint_name().unwrap()),
        )
    }

    fn render_drop_index(&self, index: &IndexWalker<'_>) -> String {
        sql_ddl::mysql::DropIndex {
            table_name: index.table().name().into(),
            index_name: index.name().into(),
        }
        .to_string()
    }

    fn render_drop_table(&self, table_name: &str) -> Vec<String> {
        vec![sql_ddl::mysql::DropTable {
            table_name: table_name.into(),
        }
        .to_string()]
    }

    fn render_redefine_tables(&self, _names: &[RedefineTable], _schemas: &Pair<&SqlSchema>) -> Vec<String> {
        unreachable!("render_redefine_table on MySQL")
    }

    fn render_rename_table(&self, name: &str, new_name: &str) -> String {
        sql_ddl::mysql::AlterTable {
            table_name: name.into(),
            changes: vec![sql_ddl::mysql::AlterTableClause::RenameTo {
                next_name: new_name.into(),
            }],
        }
        .to_string()
    }

    fn render_create_table(&self, table: &TableWalker<'_>) -> String {
        self.render_create_table_as(table, table.name())
    }

    fn render_drop_view(&self, view: &ViewWalker<'_>) -> String {
        format!("DROP VIEW {}", Quoted::mysql_ident(view.name()))
    }
}

fn render_mysql_modify(
    changes: &ColumnChanges,
    new_default: Option<&sql_schema_describer::DefaultValue>,
    next_column: &ColumnWalker<'_>,
) -> String {
    let column_type: Option<String> = if changes.type_changed() {
        Some(next_column.column_type().full_data_type.clone()).filter(|r| !r.is_empty() || r.contains("datetime"))
    // @default(now()) does not work with datetimes of certain sizes
    } else {
        Some(next_column.column_type().full_data_type.clone()).filter(|r| !r.is_empty())
    };

    let column_type = column_type
        .map(Cow::Owned)
        .unwrap_or_else(|| render_column_type(&next_column));

    let default = new_default
        .map(|default| render_default(&next_column, &default))
        .filter(|expr| !expr.is_empty())
        .map(|expression| format!(" DEFAULT {}", expression))
        .unwrap_or_else(String::new);

    format!(
        "MODIFY {column_name} {column_type}{nullability}{default}{sequence}",
        column_name = Quoted::mysql_ident(&next_column.name()),
        column_type = column_type,
        nullability = if next_column.arity().is_required() {
            " NOT NULL"
        } else {
            ""
        },
        default = default,
        sequence = if next_column.is_autoincrement() {
            " AUTO_INCREMENT"
        } else {
            ""
        },
    )
}

fn render_column_type(column: &ColumnWalker<'_>) -> Cow<'static, str> {
    if let ColumnTypeFamily::Enum(enum_name) = column.column_type_family() {
        let r#enum = column
            .schema()
            .get_enum(&enum_name)
            .unwrap_or_else(|| panic!("Could not render the variants of enum `{}`", enum_name));

        let variants: String = r#enum.values.iter().map(Quoted::mysql_string).join(", ");

        return format!("ENUM({})", variants).into();
    }

    if let ColumnTypeFamily::Unsupported(description) = &column.column_type().family {
        return description.to_string().into();
    }

    let native_type = column
        .column_native_type()
        .expect("Column native type missing in mysql_renderer::render_column_type()");

    fn render(input: Option<u32>) -> String {
        match input {
            None => "".to_string(),
            Some(arg) => format!("({})", arg),
        }
    }

    fn render_decimal(input: Option<(u32, u32)>) -> String {
        match input {
            None => "".to_string(),
            Some((precision, scale)) => format!("({}, {})", precision, scale),
        }
    }

    match native_type {
        MySqlType::Int => "INTEGER".into(),
        MySqlType::SmallInt => "SMALLINT".into(),
        MySqlType::TinyInt if column.column_type_family().is_boolean() => "BOOLEAN".into(),
        MySqlType::TinyInt => "TINYINT".into(),
        MySqlType::MediumInt => "MEDIUMINT".into(),
        MySqlType::BigInt => "BIGINT".into(),
        MySqlType::Decimal(precision) => format!("DECIMAL{}", render_decimal(precision)).into(),
        MySqlType::Float => "FLOAT".into(),
        MySqlType::Double => "DOUBLE".into(),
        MySqlType::Bit(size) => format!("BIT({size})", size = size).into(),
        MySqlType::Char(size) => format!("CHAR({size})", size = size).into(),
        MySqlType::VarChar(size) => format!("VARCHAR({size})", size = size).into(),
        MySqlType::Binary(size) => format!("BINARY({size})", size = size).into(),
        MySqlType::VarBinary(size) => format!("VARBINARY({size})", size = size).into(),
        MySqlType::TinyBlob => "TINYBLOB".into(),
        MySqlType::Blob => "BLOB".into(),
        MySqlType::MediumBlob => "MEDIUMBLOB".into(),
        MySqlType::LongBlob => "LONGBLOB".into(),
        MySqlType::TinyText => "TINYTEXT".into(),
        MySqlType::Text => "TEXT".into(),
        MySqlType::MediumText => "MEDIUMTEXT".into(),
        MySqlType::LongText => "LONGTEXT".into(),
        MySqlType::Date => "DATE".into(),
        MySqlType::Time(precision) => format!("TIME{}", render(precision)).into(),
        MySqlType::DateTime(precision) => format!("DATETIME{}", render(precision)).into(),
        MySqlType::Timestamp(precision) => format!("TIMESTAMP{}", render(precision)).into(),
        MySqlType::Year => "YEAR".into(),
        MySqlType::Json => "JSON".into(),
        MySqlType::UnsignedInt => "INTEGER UNSIGNED".into(),
        MySqlType::UnsignedSmallInt => "SMALLINT UNSIGNED".into(),
        MySqlType::UnsignedTinyInt => "TINYINT UNSIGNED".into(),
        MySqlType::UnsignedMediumInt => "MEDIUMINT UNSIGNED".into(),
        MySqlType::UnsignedBigInt => "BIGINT UNSIGNED".into(),
    }
}

fn escape_string_literal(s: &str) -> Cow<'_, str> {
    static STRING_LITERAL_CHARACTER_TO_ESCAPE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"'"#).unwrap());

    STRING_LITERAL_CHARACTER_TO_ESCAPE_RE.replace_all(s, "'$0")
}

/// https://dev.mysql.com/doc/refman/8.0/en/alter-table.html
///
/// We don't use SET DEFAULT because it can't be used to set the default to an expression on most
/// MySQL versions. We use MODIFY for default changes instead.
#[derive(Debug)]
enum MysqlAlterColumn {
    DropDefault,
    Modify {
        new_default: Option<DefaultValue>,
        changes: ColumnChanges,
    },
}

impl MysqlAlterColumn {
    fn new(columns: &Pair<ColumnWalker<'_>>, changes: &ColumnChanges) -> Self {
        if changes.only_default_changed() && columns.next().default().is_none() {
            return MysqlAlterColumn::DropDefault;
        }

        if changes.column_was_renamed() {
            unreachable!("MySQL column renaming.")
        }

        let defaults = (
            columns.previous().default().as_ref().map(|d| d.kind()),
            columns.next().default().as_ref().map(|d| d.kind()),
        );

        // @default(dbgenerated()) does not give us the information in the prisma schema, so we have to
        // transfer it from the introspected current state of the database.
        let new_default = match defaults {
            (Some(DefaultKind::DBGENERATED(previous)), Some(DefaultKind::DBGENERATED(next)))
                if next.is_empty() && !previous.is_empty() =>
            {
                Some(DefaultValue::db_generated(previous.clone()))
            }
            _ => columns.next().default().cloned(),
        };

        MysqlAlterColumn::Modify {
            changes: *changes,
            new_default,
        }
    }
}

fn render_default<'a>(column: &ColumnWalker<'a>, default: &'a DefaultValue) -> Cow<'a, str> {
    match default.kind() {
        DefaultKind::DBGENERATED(val) => val.as_str().into(),
        DefaultKind::VALUE(PrismaValue::String(val)) | DefaultKind::VALUE(PrismaValue::Enum(val)) => {
            Quoted::mysql_string(escape_string_literal(&val)).to_string().into()
        }
        DefaultKind::NOW => {
            let precision = column
                .column_native_type()
                .as_ref()
                .and_then(MySqlType::timestamp_precision)
                .unwrap_or(3);

            format!("CURRENT_TIMESTAMP({})", precision).into()
        }
        DefaultKind::VALUE(val) if column.column_type_family().is_datetime() => {
            Quoted::mysql_string(val).to_string().into()
        }
        DefaultKind::VALUE(val) => val.to_string().into(),
        DefaultKind::SEQUENCE(_) => Default::default(),
    }
}
