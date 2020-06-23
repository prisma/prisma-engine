use super::column::ColumnDiffer;
use crate::sql_schema_helpers::{ColumnRef, TableRef};
use sql_schema_describer::{ForeignKey, Index};

pub(crate) struct TableDiffer<'a> {
    pub(crate) diffing_options: &'a super::DiffingOptions,
    pub(crate) previous: TableRef<'a>,
    pub(crate) next: TableRef<'a>,
}

impl<'schema> TableDiffer<'schema> {
    pub(crate) fn column_pairs<'a>(&'a self) -> impl Iterator<Item = ColumnDiffer<'schema>> + 'a {
        self.previous_columns()
            .filter_map(move |previous_column| {
                self.next_columns()
                    .find(|next_column| columns_match(&previous_column, next_column))
                    .map(|next_column| (previous_column, next_column))
            })
            .map(move |(previous, next)| ColumnDiffer {
                diffing_options: self.diffing_options,
                previous,
                next,
            })
    }

    pub(crate) fn dropped_columns<'a>(&'a self) -> impl Iterator<Item = ColumnRef<'schema>> + 'a {
        self.previous_columns().filter(move |previous_column| {
            self.next_columns()
                .find(|next_column| columns_match(previous_column, next_column))
                .is_none()
        })
    }

    pub(crate) fn added_columns<'a>(&'a self) -> impl Iterator<Item = ColumnRef<'schema>> + 'a {
        self.next_columns().filter(move |next_column| {
            self.previous_columns()
                .find(|previous_column| columns_match(previous_column, next_column))
                .is_none()
        })
    }

    pub(crate) fn created_foreign_keys(&self) -> impl Iterator<Item = &ForeignKey> {
        self.next_foreign_keys().filter(move |next_fk| {
            self.previous_foreign_keys()
                .find(|previous_fk| super::foreign_keys_match(previous_fk, next_fk))
                .is_none()
        })
    }

    pub(crate) fn dropped_foreign_keys(&self) -> impl Iterator<Item = &ForeignKey> {
        self.previous_foreign_keys().filter(move |previous_fk| {
            self.next_foreign_keys()
                .find(|next_fk| super::foreign_keys_match(previous_fk, next_fk))
                .is_none()
        })
    }

    pub(crate) fn created_indexes<'a>(&'a self) -> impl Iterator<Item = &'schema Index> + 'a {
        self.next_indexes().filter(move |next_index| {
            !self
                .previous_indexes()
                .any(move |previous_index| indexes_match(previous_index, next_index))
        })
    }

    pub(crate) fn dropped_indexes<'a>(&'a self) -> impl Iterator<Item = &'schema Index> + 'a {
        self.previous_indexes().filter(move |previous_index| {
            !self
                .next_indexes()
                .any(|next_index| indexes_match(previous_index, next_index))
        })
    }

    pub(crate) fn index_pairs<'a>(&'a self) -> impl Iterator<Item = (&'schema Index, &'schema Index)> + 'a {
        self.previous_indexes().filter_map(move |previous_index| {
            self.next_indexes()
                .find(|next_index| indexes_match(previous_index, next_index) && previous_index.name != next_index.name)
                .map(|renamed_index| (previous_index, renamed_index))
        })
    }

    fn previous_columns<'a>(&'a self) -> impl Iterator<Item = ColumnRef<'schema>> + 'a {
        self.previous.columns()
    }

    fn next_columns<'a>(&'a self) -> impl Iterator<Item = ColumnRef<'schema>> + 'a {
        self.next.columns()
    }

    fn previous_foreign_keys(&self) -> impl Iterator<Item = &ForeignKey> {
        self.previous.table.foreign_keys.iter()
    }

    fn next_foreign_keys(&self) -> impl Iterator<Item = &ForeignKey> {
        self.next.table.foreign_keys.iter()
    }

    fn previous_indexes<'a>(&'a self) -> impl Iterator<Item = &'schema Index> + 'a {
        self.previous.table.indices.iter()
    }

    fn next_indexes<'a>(&'a self) -> impl Iterator<Item = &'schema Index> + 'a {
        self.next.table.indices.iter()
    }
}

fn columns_match(a: &ColumnRef<'_>, b: &ColumnRef<'_>) -> bool {
    a.name() == b.name()
}

/// Compare two SQL indexes and return whether they only differ by name.
fn indexes_match(first: &Index, second: &Index) -> bool {
    first.columns == second.columns && first.tpe == second.tpe
}
