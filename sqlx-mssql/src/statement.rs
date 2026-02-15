use super::MssqlColumn;
use crate::column::ColumnIndex;
use crate::error::Error;
use crate::ext::ustr::UStr;
use crate::HashMap;
use crate::{Mssql, MssqlArguments, MssqlTypeInfo};
use either::Either;
use sqlx_core::sql_str::SqlStr;
use std::sync::Arc;

pub(crate) use sqlx_core::statement::*;

#[derive(Debug, Clone)]
pub struct MssqlStatement {
    pub(crate) sql: SqlStr,
    pub(crate) metadata: MssqlStatementMetadata,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct MssqlStatementMetadata {
    pub(crate) columns: Arc<Vec<MssqlColumn>>,
    pub(crate) column_names: Arc<HashMap<UStr, usize>>,
    pub(crate) parameters: usize,
}

impl Statement for MssqlStatement {
    type Database = Mssql;

    fn into_sql(self) -> SqlStr {
        self.sql
    }

    fn sql(&self) -> &SqlStr {
        &self.sql
    }

    fn parameters(&self) -> Option<Either<&[MssqlTypeInfo], usize>> {
        Some(Either::Right(self.metadata.parameters))
    }

    fn columns(&self) -> &[MssqlColumn] {
        &self.metadata.columns
    }

    impl_statement_query!(MssqlArguments);
}

impl ColumnIndex<MssqlStatement> for &'_ str {
    fn index(&self, statement: &MssqlStatement) -> Result<usize, Error> {
        statement
            .metadata
            .column_names
            .get(*self)
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
            .copied()
    }
}
