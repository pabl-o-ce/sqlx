use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};

use crate::database::MssqlArgumentValue;
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::value::MssqlData;
use crate::{Mssql, MssqlTypeInfo, MssqlValueRef};

// ── NaiveDateTime ───────────────────────────────────────────────────────────

impl Type<Mssql> for NaiveDateTime {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("DATETIME2")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(
            ty.name.as_str(),
            "DATETIME2" | "DATETIME" | "SMALLDATETIME"
        )
    }
}

impl Encode<'_, Mssql> for NaiveDateTime {
    fn encode_by_ref(
        &self,
        buf: &mut Vec<MssqlArgumentValue>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::NaiveDateTime(*self));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for NaiveDateTime {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            MssqlData::NaiveDateTime(v) => Ok(*v),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected datetime, got {:?}", value.data).into()),
        }
    }
}

// ── NaiveDate ───────────────────────────────────────────────────────────────

impl Type<Mssql> for NaiveDate {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("DATE")
    }
}

impl Encode<'_, Mssql> for NaiveDate {
    fn encode_by_ref(
        &self,
        buf: &mut Vec<MssqlArgumentValue>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::NaiveDate(*self));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for NaiveDate {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            MssqlData::NaiveDate(v) => Ok(*v),
            MssqlData::NaiveDateTime(v) => Ok(v.date()),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected date, got {:?}", value.data).into()),
        }
    }
}

// ── NaiveTime ───────────────────────────────────────────────────────────────

impl Type<Mssql> for NaiveTime {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("TIME")
    }
}

impl Encode<'_, Mssql> for NaiveTime {
    fn encode_by_ref(
        &self,
        buf: &mut Vec<MssqlArgumentValue>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::NaiveTime(*self));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for NaiveTime {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            MssqlData::NaiveTime(v) => Ok(*v),
            MssqlData::NaiveDateTime(v) => Ok(v.time()),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected time, got {:?}", value.data).into()),
        }
    }
}

// ── DateTime<Utc> ───────────────────────────────────────────────────────────

impl Type<Mssql> for DateTime<Utc> {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("DATETIME2")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(
            ty.name.as_str(),
            "DATETIME2" | "DATETIMEOFFSET"
        )
    }
}

impl Encode<'_, Mssql> for DateTime<Utc> {
    fn encode_by_ref(
        &self,
        buf: &mut Vec<MssqlArgumentValue>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::NaiveDateTime(self.naive_utc()));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for DateTime<Utc> {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            MssqlData::NaiveDateTime(v) => Ok(v.and_utc()),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected datetime, got {:?}", value.data).into()),
        }
    }
}
