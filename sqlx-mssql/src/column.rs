use crate::ext::ustr::UStr;
use crate::{Mssql, MssqlTypeInfo};
pub(crate) use sqlx_core::column::*;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub struct MssqlColumn {
    pub(crate) ordinal: usize,
    pub(crate) name: UStr,
    pub(crate) type_info: MssqlTypeInfo,
    pub(crate) origin: ColumnOrigin,
}

impl Column for MssqlColumn {
    type Database = Mssql;

    fn ordinal(&self) -> usize {
        self.ordinal
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn type_info(&self) -> &MssqlTypeInfo {
        &self.type_info
    }

    fn origin(&self) -> ColumnOrigin {
        self.origin.clone()
    }
}
