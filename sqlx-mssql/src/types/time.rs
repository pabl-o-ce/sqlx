use time::{Date, OffsetDateTime, PrimitiveDateTime, Time};

use crate::database::MssqlArgumentValue;
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::value::MssqlData;
use crate::{Mssql, MssqlTypeInfo, MssqlValueRef};

// ── Date ───────────────────────────────────────────────────────────────────

impl Type<Mssql> for Date {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("DATE")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        ty.base_name() == "DATE"
    }
}

impl Encode<'_, Mssql> for Date {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::TimeDate(*self));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for Date {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            MssqlData::TimeDate(v) => Ok(*v),
            MssqlData::TimePrimitiveDateTime(v) => Ok(v.date()),
            #[cfg(feature = "chrono")]
            MssqlData::NaiveDate(v) => chrono_to_time_date(*v),
            #[cfg(feature = "chrono")]
            MssqlData::NaiveDateTime(v) => chrono_to_time_date(v.date()),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected date, got {:?}", value.data).into()),
        }
    }
}

// ── Time ───────────────────────────────────────────────────────────────────

impl Type<Mssql> for Time {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("TIME")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        ty.base_name() == "TIME"
    }
}

impl Encode<'_, Mssql> for Time {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::TimeTime(*self));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for Time {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            MssqlData::TimeTime(v) => Ok(*v),
            MssqlData::TimePrimitiveDateTime(v) => Ok(v.time()),
            #[cfg(feature = "chrono")]
            MssqlData::NaiveTime(v) => chrono_to_time_time(*v),
            #[cfg(feature = "chrono")]
            MssqlData::NaiveDateTime(v) => chrono_to_time_time(v.time()),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected time, got {:?}", value.data).into()),
        }
    }
}

// ── PrimitiveDateTime ──────────────────────────────────────────────────────

impl Type<Mssql> for PrimitiveDateTime {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("DATETIME2")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(ty.base_name(), "DATETIME2" | "DATETIME" | "SMALLDATETIME")
    }
}

impl Encode<'_, Mssql> for PrimitiveDateTime {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::TimePrimitiveDateTime(*self));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for PrimitiveDateTime {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            MssqlData::TimePrimitiveDateTime(v) => Ok(*v),
            #[cfg(feature = "chrono")]
            MssqlData::NaiveDateTime(v) => chrono_to_time_pdt(*v),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected datetime, got {:?}", value.data).into()),
        }
    }
}

// ── OffsetDateTime ─────────────────────────────────────────────────────────

impl Type<Mssql> for OffsetDateTime {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("DATETIMEOFFSET")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(ty.base_name(), "DATETIMEOFFSET" | "DATETIME2")
    }
}

impl Encode<'_, Mssql> for OffsetDateTime {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::TimeOffsetDateTime(*self));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for OffsetDateTime {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            MssqlData::TimeOffsetDateTime(v) => Ok(*v),
            MssqlData::TimePrimitiveDateTime(v) => Ok(v.assume_utc()),
            #[cfg(feature = "chrono")]
            MssqlData::DateTimeFixedOffset(v) => chrono_to_time_odt(*v),
            #[cfg(feature = "chrono")]
            MssqlData::NaiveDateTime(v) => chrono_to_time_pdt(*v).map(|p| p.assume_utc()),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected datetimeoffset, got {:?}", value.data).into()),
        }
    }
}

// When both `time` and `chrono` are enabled, `column_data_to_mssql_data`
// routes wire date/time data into chrono variants, so the `time::*` decode
// impls must convert from those.

#[cfg(feature = "chrono")]
fn chrono_to_time_date(d: chrono::NaiveDate) -> Result<Date, BoxDynError> {
    use chrono::Datelike;
    let month = u8::try_from(d.month())
        .ok()
        .and_then(|m| time::Month::try_from(m).ok())
        .ok_or_else(|| format!("invalid month value from chrono: {}", d.month()))?;
    let day = u8::try_from(d.day())
        .map_err(|_| format!("invalid day value from chrono: {}", d.day()))?;
    Date::from_calendar_date(d.year(), month, day)
        .map_err(|e| format!("failed to convert chrono::NaiveDate to time::Date: {e}").into())
}

#[cfg(feature = "chrono")]
fn chrono_to_time_time(t: chrono::NaiveTime) -> Result<Time, BoxDynError> {
    use chrono::Timelike;
    let hour = u8::try_from(t.hour()).map_err(|_| format!("invalid hour: {}", t.hour()))?;
    let minute = u8::try_from(t.minute()).map_err(|_| format!("invalid minute: {}", t.minute()))?;
    let second = u8::try_from(t.second()).map_err(|_| format!("invalid second: {}", t.second()))?;
    // chrono represents leap seconds as nanos >= 1_000_000_000; SQL Server
    // does not emit leap seconds, so cap defensively.
    let nanos = std::cmp::min(t.nanosecond(), 999_999_999);
    Time::from_hms_nano(hour, minute, second, nanos)
        .map_err(|e| format!("failed to convert chrono::NaiveTime to time::Time: {e}").into())
}

#[cfg(feature = "chrono")]
fn chrono_to_time_pdt(dt: chrono::NaiveDateTime) -> Result<PrimitiveDateTime, BoxDynError> {
    let date = chrono_to_time_date(dt.date())?;
    let time = chrono_to_time_time(dt.time())?;
    Ok(PrimitiveDateTime::new(date, time))
}

#[cfg(feature = "chrono")]
fn chrono_to_time_odt(
    dt: chrono::DateTime<chrono::FixedOffset>,
) -> Result<OffsetDateTime, BoxDynError> {
    let naive = chrono_to_time_pdt(dt.naive_local())?;
    let offset_secs = dt.offset().local_minus_utc();
    let offset = time::UtcOffset::from_whole_seconds(offset_secs)
        .map_err(|e| format!("invalid UTC offset {offset_secs}s from chrono: {e}"))?;
    Ok(naive.assume_offset(offset))
}
