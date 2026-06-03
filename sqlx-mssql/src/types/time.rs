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
            #[cfg(not(feature = "chrono"))]
            MssqlData::TimeDate(v) => Ok(*v),
            #[cfg(not(feature = "chrono"))]
            MssqlData::TimePrimitiveDateTime(v) => Ok(v.date()),
            // When `chrono` is also enabled it wins the decoder, so DATE is
            // stored as a chrono variant; convert it to `time`.
            #[cfg(feature = "chrono")]
            MssqlData::NaiveDate(v) => date_from_chrono(*v),
            #[cfg(feature = "chrono")]
            MssqlData::NaiveDateTime(v) => date_from_chrono(v.date()),
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
            #[cfg(not(feature = "chrono"))]
            MssqlData::TimeTime(v) => Ok(*v),
            #[cfg(not(feature = "chrono"))]
            MssqlData::TimePrimitiveDateTime(v) => Ok(v.time()),
            // When `chrono` is also enabled it wins the decoder, so TIME is
            // stored as a chrono variant; convert it to `time`.
            #[cfg(feature = "chrono")]
            MssqlData::NaiveTime(v) => time_from_chrono(*v),
            #[cfg(feature = "chrono")]
            MssqlData::NaiveDateTime(v) => time_from_chrono(v.time()),
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
            #[cfg(not(feature = "chrono"))]
            MssqlData::TimePrimitiveDateTime(v) => Ok(*v),
            // When `chrono` is also enabled it wins the decoder, so DATETIME2 is
            // stored as `MssqlData::NaiveDateTime`; convert it to `time`.
            #[cfg(feature = "chrono")]
            MssqlData::NaiveDateTime(v) => Ok(PrimitiveDateTime::new(
                date_from_chrono(v.date())?,
                time_from_chrono(v.time())?,
            )),
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
            #[cfg(not(feature = "chrono"))]
            MssqlData::TimeOffsetDateTime(v) => Ok(*v),
            #[cfg(not(feature = "chrono"))]
            MssqlData::TimePrimitiveDateTime(v) => Ok(v.assume_utc()),
            // When `chrono` is also enabled it wins the decoder, so
            // DATETIMEOFFSET is stored as `MssqlData::DateTimeFixedOffset` (and
            // DATETIME2 as `NaiveDateTime`); convert to `time`.
            #[cfg(feature = "chrono")]
            MssqlData::DateTimeFixedOffset(v) => {
                let local = v.naive_local();
                let offset = time::UtcOffset::from_whole_seconds(v.offset().local_minus_utc())?;
                Ok(
                    PrimitiveDateTime::new(date_from_chrono(local.date())?, time_from_chrono(local.time())?)
                        .assume_offset(offset),
                )
            }
            #[cfg(feature = "chrono")]
            MssqlData::NaiveDateTime(v) => Ok(PrimitiveDateTime::new(
                date_from_chrono(v.date())?,
                time_from_chrono(v.time())?,
            )
            .assume_utc()),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected datetimeoffset, got {:?}", value.data).into()),
        }
    }
}

/// Convert a `chrono::NaiveDate` to a `time::Date`.
///
/// Needed only when both `chrono` and `time` are enabled: the decoder stores
/// temporal values as chrono variants (chrono wins), so the `time` `Decode`
/// impls must convert.
#[cfg(feature = "chrono")]
fn date_from_chrono(d: chrono::NaiveDate) -> Result<Date, BoxDynError> {
    use chrono::Datelike as _;
    let month = time::Month::try_from(u8::try_from(d.month())?)?;
    Ok(Date::from_calendar_date(
        d.year(),
        month,
        u8::try_from(d.day())?,
    )?)
}

/// Convert a `chrono::NaiveTime` to a `time::Time`.
///
/// Leap-second nanoseconds (≥ 1e9) are clamped to the last representable
/// nanosecond, since `time::Time` has no leap-second representation.
#[cfg(feature = "chrono")]
fn time_from_chrono(t: chrono::NaiveTime) -> Result<Time, BoxDynError> {
    use chrono::Timelike as _;
    Ok(Time::from_hms_nano(
        u8::try_from(t.hour())?,
        u8::try_from(t.minute())?,
        u8::try_from(t.second())?,
        std::cmp::min(t.nanosecond(), 999_999_999),
    )?)
}
