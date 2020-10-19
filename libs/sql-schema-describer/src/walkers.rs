//! Functions and types for conveniently traversing and querying a SqlSchema.

#![deny(missing_docs)]

use crate::{
    Column, ColumnArity, ColumnType, ColumnTypeFamily, DefaultValue, Enum, ForeignKey, Index, IndexType, PrimaryKey,
    SqlSchema, Table,
};

/// Traverse all the columns in the schema.
pub fn walk_columns<'a>(schema: &'a SqlSchema) -> impl Iterator<Item = ColumnWalker<'a>> + 'a {
    schema.tables.iter().flat_map(move |table| {
        table
            .columns
            .iter()
            .map(move |column| ColumnWalker { schema, column, table })
    })
}

/// Find a column by table and column name in the schema.
pub fn find_column<'a>(schema: &'a SqlSchema, table_name: &str, column_name: &str) -> Option<ColumnWalker<'a>> {
    schema
        .tables
        .iter()
        .find(move |table| table.name == table_name)
        .and_then(move |table| {
            table
                .columns
                .iter()
                .find(|column| column.name == column_name)
                .map(|column| ColumnWalker { schema, table, column })
        })
}

/// Traverse a table column.
#[derive(Debug, Clone, Copy)]
pub struct ColumnWalker<'a> {
    /// The schema the column is contained in. This field will become private.
    pub schema: &'a SqlSchema,
    /// The underlying column struct. This field will become private.
    pub column: &'a Column,
    /// The underlying table struct. This field will become private.
    pub table: &'a Table,
}

impl<'a> ColumnWalker<'a> {
    /// The nullability and arity of the column.
    pub fn arity(&self) -> &ColumnArity {
        &self.column.tpe.arity
    }

    /// The type family.
    pub fn column_type_family(&self) -> &'a ColumnTypeFamily {
        &self.column.tpe.family
    }

    /// Extract an `Enum` column type family, or `None` if the family is something else.
    pub fn column_type_family_as_enum(&self) -> Option<&'a Enum> {
        self.column_type_family().as_enum().map(|enum_name| {
            self.schema()
                .get_enum(enum_name)
                .ok_or_else(|| panic!("Cannot find enum referenced in ColumnTypeFamily (`{}`)", enum_name))
                .unwrap()
        })
    }

    /// The column name.
    pub fn name(&self) -> &'a str {
        &self.column.name
    }

    /// The default value for the column.
    pub fn default(&self) -> Option<&'a DefaultValue> {
        self.column.default.as_ref()
    }

    /// The full column type.
    pub fn column_type(&self) -> &'a ColumnType {
        &self.column.tpe
    }

    /// Is this column an auto-incrementing integer?
    pub fn is_autoincrement(&self) -> bool {
        self.column.auto_increment
    }

    /// Returns whether two columns are named the same and belong to the same table.
    pub fn is_same_column(&self, other: &ColumnWalker<'_>) -> bool {
        self.name() == other.name() && self.table().name() == other.table().name()
    }

    /// Returns whether this column is the primary key. If it is only part of the primary key, this will return false.
    pub fn is_single_primary_key(&self) -> bool {
        self.table()
            .primary_key()
            .map(|pk| pk.columns == [self.name()])
            .unwrap_or(false)
    }

    /// Traverse to the column's table.
    pub fn table(&self) -> TableWalker<'a> {
        TableWalker {
            schema: self.schema,
            table: self.table,
        }
    }

    /// Get a reference to the SQL schema the column is part of.
    pub fn schema(&self) -> &'a SqlSchema {
        self.schema
    }
}

/// Traverse a table.
#[derive(Clone, Copy)]
pub struct TableWalker<'a> {
    /// The schema the column is contained in. This field will become private.
    pub schema: &'a SqlSchema,
    /// The underlying table struct. This field will become private.
    pub table: &'a Table,
}

impl<'a> TableWalker<'a> {
    /// Create a TableWalker from a schema and a reference to one of its tables.
    pub fn new(schema: &'a SqlSchema, table: &'a Table) -> Self {
        Self { schema, table }
    }

    /// Get a column in the table, by name.
    pub fn column(&self, column_name: &str) -> Option<ColumnWalker<'a>> {
        self.columns().find(|column| column.name() == column_name)
    }

    /// Traverse the table's columns.
    pub fn columns<'b>(&'b self) -> impl Iterator<Item = ColumnWalker<'a>> + 'b {
        self.table.columns.iter().map(move |column| ColumnWalker {
            column,
            schema: self.schema,
            table: self.table,
        })
    }

    /// Traverse the indexes on the table.
    pub fn indexes<'b>(&'b self) -> impl Iterator<Item = IndexWalker<'a>> + 'b {
        self.table.indices.iter().map(move |index| IndexWalker {
            index,
            schema: self.schema,
            table: self.table,
        })
    }

    /// Traverse the foreign keys on the table.
    pub fn foreign_keys(self) -> impl Iterator<Item = ForeignKeyWalker<'a>> {
        self.table.foreign_keys.iter().map(move |foreign_key| ForeignKeyWalker {
            foreign_key,
            table: self,
        })
    }

    /// The table name.
    pub fn name(&self) -> &'a str {
        &self.table.name
    }

    /// Try to traverse a foreign key for a single column.
    pub fn foreign_key_for_column(&self, column: &str) -> Option<&'a ForeignKey> {
        self.table.foreign_key_for_column(column)
    }

    /// Traverse to the primary key of the table.
    pub fn primary_key(&self) -> Option<&'a PrimaryKey> {
        self.table.primary_key.as_ref()
    }
}

/// Traverse a foreign key.
pub struct ForeignKeyWalker<'schema> {
    table: TableWalker<'schema>,
    foreign_key: &'schema ForeignKey,
}

impl<'a, 'schema> ForeignKeyWalker<'schema> {
    /// The names of the foreign key columns on the referencing table.
    pub fn constrained_column_names(&self) -> &[String] {
        &self.foreign_key.columns
    }

    /// The foreign key columns on the referencing table.
    pub fn constrained_columns<'b>(&'b self) -> impl Iterator<Item = ColumnWalker<'schema>> + 'b {
        self.table()
            .columns()
            .filter(move |column| self.foreign_key.columns.contains(&column.column.name))
    }

    /// The name of the foreign key constraint.
    pub fn constraint_name(&self) -> Option<&'schema str> {
        self.foreign_key.constraint_name.as_deref()
    }

    /// Access the underlying ForeignKey struct.
    pub fn inner(&self) -> &'schema ForeignKey {
        self.foreign_key
    }

    /// The number of columns referenced by the constraint.
    pub fn referenced_columns_count(&self) -> usize {
        self.foreign_key.referenced_columns.len()
    }

    /// The table the foreign key "points to".
    pub fn referenced_table(&self) -> TableWalker<'schema> {
        TableWalker {
            schema: self.table.schema,
            table: self
                .table
                .schema
                .table(&self.foreign_key.referenced_table)
                .expect("foreign key references unknown table"),
        }
    }

    /// Traverse to the referencing table.
    pub fn table(&self) -> &TableWalker<'schema> {
        &self.table
    }
}

/// Traverse an index.
pub struct IndexWalker<'a> {
    schema: &'a SqlSchema,
    table: &'a Table,
    index: &'a Index,
}

impl<'a> IndexWalker<'a> {
    /// The names of the indexed columns.
    pub fn column_names(&self) -> &[String] {
        &self.index.columns
    }

    /// Traverse the indexed columns.
    pub fn columns<'b>(&'b self) -> impl Iterator<Item = ColumnWalker<'a>> + 'b {
        self.index
            .columns
            .iter()
            .map(move |column_name| {
                self.table
                    .columns
                    .iter()
                    .find(|column| &column.name == column_name)
                    .expect("Failed to find column referenced in index")
            })
            .map(move |column| ColumnWalker {
                schema: self.schema,
                column,
                table: self.table,
            })
    }

    /// The underlying index struct.
    pub fn index(&self) -> &Index {
        &self.index
    }

    /// The IndexType
    pub fn index_type(&self) -> &IndexType {
        &self.index.tpe
    }

    /// The name of the index.
    pub fn name(&self) -> &str {
        &self.index.name
    }
}

/// Extension methods for the traversal of a SqlSchema.
pub trait SqlSchemaExt {
    /// Find a table by name.
    fn table_walker<'a>(&'a self, name: &str) -> Option<TableWalker<'a>>;
}

impl SqlSchemaExt for SqlSchema {
    fn table_walker<'a>(&'a self, name: &str) -> Option<TableWalker<'a>> {
        Some(TableWalker {
            table: self.table(name).ok()?,
            schema: self,
        })
    }
}
