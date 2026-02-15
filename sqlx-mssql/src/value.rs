use std::borrow::Cow;

pub(crate) use sqlx_core::value::*;

use crate::error::BoxDynError;
use crate::{Mssql, MssqlTypeInfo};

/// Internal storage for an MSSQL value, decoupled from tiberius lifetimes.
#[derive(Debug, Clone)]
pub(crate) enum MssqlData {
    Null,
    Bool(bool),
    U8(u8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    String(String),
    Binary(Vec<u8>),
}

/// Implementation of [`Value`] for MSSQL.
#[derive(Debug, Clone)]
pub struct MssqlValue {
    pub(crate) data: MssqlData,
    pub(crate) type_info: MssqlTypeInfo,
}

/// Implementation of [`ValueRef`] for MSSQL.
#[derive(Debug, Clone)]
pub struct MssqlValueRef<'r> {
    pub(crate) data: &'r MssqlData,
    pub(crate) type_info: MssqlTypeInfo,
}

impl<'r> MssqlValueRef<'r> {
    pub(crate) fn as_str(&self) -> Result<&'r str, BoxDynError> {
        match self.data {
            MssqlData::String(ref s) => Ok(s.as_str()),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected string, got {:?}", self.data).into()),
        }
    }

    pub(crate) fn as_bytes(&self) -> Result<&'r [u8], BoxDynError> {
        match self.data {
            MssqlData::Binary(ref b) => Ok(b.as_slice()),
            MssqlData::String(ref s) => Ok(s.as_bytes()),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected binary, got {:?}", self.data).into()),
        }
    }
}

impl Value for MssqlValue {
    type Database = Mssql;

    fn as_ref(&self) -> MssqlValueRef<'_> {
        MssqlValueRef {
            data: &self.data,
            type_info: self.type_info.clone(),
        }
    }

    fn type_info(&self) -> Cow<'_, MssqlTypeInfo> {
        Cow::Borrowed(&self.type_info)
    }

    fn is_null(&self) -> bool {
        matches!(self.data, MssqlData::Null)
    }
}

impl<'r> ValueRef<'r> for MssqlValueRef<'r> {
    type Database = Mssql;

    fn to_owned(&self) -> MssqlValue {
        MssqlValue {
            data: self.data.clone(),
            type_info: self.type_info.clone(),
        }
    }

    fn type_info(&self) -> Cow<'_, MssqlTypeInfo> {
        Cow::Borrowed(&self.type_info)
    }

    fn is_null(&self) -> bool {
        matches!(self.data, MssqlData::Null)
    }
}

/// Convert a `tiberius::ColumnData` into our owned `MssqlData`.
pub(crate) fn column_data_to_mssql_data(data: &tiberius::ColumnData<'_>) -> MssqlData {
    match data {
        tiberius::ColumnData::U8(Some(v)) => MssqlData::U8(*v),
        tiberius::ColumnData::I16(Some(v)) => MssqlData::I16(*v),
        tiberius::ColumnData::I32(Some(v)) => MssqlData::I32(*v),
        tiberius::ColumnData::I64(Some(v)) => MssqlData::I64(*v),
        tiberius::ColumnData::F32(Some(v)) => MssqlData::F32(*v),
        tiberius::ColumnData::F64(Some(v)) => MssqlData::F64(*v),
        tiberius::ColumnData::Bit(Some(v)) => MssqlData::Bool(*v),
        tiberius::ColumnData::String(Some(v)) => MssqlData::String(v.to_string()),
        tiberius::ColumnData::Binary(Some(v)) => MssqlData::Binary(v.to_vec()),
        // All None variants and unhandled types map to Null
        _ => MssqlData::Null,
    }
}
