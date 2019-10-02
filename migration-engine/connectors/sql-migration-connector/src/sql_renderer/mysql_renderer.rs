use super::common::*;
use itertools::Itertools;
use sql_schema_describer::*;

const MYSQL_TEXT_FIELD_INDEX_PREFIX: &str = "(100)";

pub struct MySqlRenderer {}
impl super::SqlRenderer for MySqlRenderer {
    fn quote(&self, name: &str) -> String {
        format!("`{}`", name)
    }

    fn render_column(&self, schema_name: &str, table: &Table, column: &Column, add_fk_prefix: bool) -> String {
        let column_name = self.quote(&column.name);
        let tpe_str = self.render_column_type(&column.tpe);
        let nullability_str = render_nullability(&table, &column);
        let default_str = self.render_default(&column).unwrap_or_else(String::new);

        let foreign_key = table.foreign_key_for_column(&column.name);
        let references_str = self.render_references(&schema_name, column, foreign_key);
        let auto_increment_str = if column.auto_increment { "AUTO_INCREMENT" } else { "" };

        match foreign_key {
            Some(_) => {
                let index_prefix = if column.tpe.family == ColumnTypeFamily::String {
                    MYSQL_TEXT_FIELD_INDEX_PREFIX
                } else {
                    ""
                };

                let add = if add_fk_prefix { "ADD" } else { "" };
                let fk_line = {
                    let column_name = &format!("{}{}", self.quote(&column.name), index_prefix);
                    format!("{} FOREIGN KEY ({}) {}", add, column_name, references_str)
                };
                format!(
                    "{} {} {} {},\n{}",
                    column_name, tpe_str, nullability_str, default_str, fk_line
                )
            }
            None => format!(
                "{} {} {} {} {}",
                column_name, tpe_str, nullability_str, default_str, auto_increment_str
            ),
        }
    }

    fn render_column_type(&self, t: &ColumnType) -> String {
        match &t.family {
            ColumnTypeFamily::Boolean => format!("boolean"),
            ColumnTypeFamily::DateTime => format!("datetime(3)"),
            ColumnTypeFamily::Float => format!("Decimal(65,30)"),
            ColumnTypeFamily::Int => format!("int"),
            ColumnTypeFamily::String => format!("mediumtext"),
            x => unimplemented!("{:?} not handled yet", x),
        }
    }

    fn render_references(&self, schema_name: &str, foreign_key: Option<&ForeignKey>) -> String {
        match foreign_key {
            Some(fk) => format!(
                "REFERENCES `{}`.`{}`(`{}`) {}",
                schema_name,
                fk.referenced_table,
                fk.referenced_columns.first().unwrap(),
                render_on_delete(&fk.on_delete_action)
            ),
            None => "".to_string(),
        }
    }

    // For String columns, we can't index the whole column, so we have to add the prefix (e.g. `ON name(191)`).
    fn render_index_columns(&self, table: &Table, columns: &[String]) -> String {
        columns
            .iter()
            .map(|name| {
                (
                    name,
                    &table
                        .columns
                        .iter()
                        .find(|col| &col.name == name)
                        .expect("Index column is in the table.")
                        .tpe
                        .family,
                )
            })
            .map(|(name, tpe)| {
                if tpe == &ColumnTypeFamily::String {
                    format!("{}{}", self.quote(&name), MYSQL_TEXT_FIELD_INDEX_PREFIX)
                } else {
                    self.quote(&name)
                }
            })
            .join(", ")
    }

    fn render_default(&self, column: &Column) -> Option<String> {
        // Before MySQL 8, mediumtext (String) columns cannot have a default.
        if column.tpe.family == ColumnTypeFamily::String {
            return None;
        }

        Some(super::common::render_default(column))
    }
}
