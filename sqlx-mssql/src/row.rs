use std::sync::Arc;

pub(crate) use sqlx_core::row::*;

use crate::column::ColumnIndex;
use crate::error::Error;
use crate::ext::ustr::UStr;
use crate::value::MssqlData;
use crate::HashMap;
use crate::{Mssql, MssqlColumn, MssqlValueRef};

/// Implementation of [`Row`] for MSSQL.
pub struct MssqlRow {
    pub(crate) values: Vec<MssqlData>,
    pub(crate) columns: Arc<Vec<MssqlColumn>>,
    pub(crate) column_names: Arc<HashMap<UStr, usize>>,
}

impl Row for MssqlRow {
    type Database = Mssql;

    fn columns(&self) -> &[MssqlColumn] {
        &self.columns
    }

    fn try_get_raw<I>(&self, index: I) -> Result<MssqlValueRef<'_>, Error>
    where
        I: ColumnIndex<Self>,
    {
        let index = index.index(self)?;
        let column = &self.columns[index];
        let data = &self.values[index];

        Ok(MssqlValueRef {
            data,
            type_info: column.type_info.clone(),
        })
    }
}

impl ColumnIndex<MssqlRow> for &'_ str {
    fn index(&self, row: &MssqlRow) -> Result<usize, Error> {
        row.column_names
            .get(*self)
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
            .copied()
    }
}

impl std::fmt::Debug for MssqlRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        debug_row(self, f)
    }
}
