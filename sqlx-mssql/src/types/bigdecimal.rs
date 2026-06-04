use bigdecimal::BigDecimal;

use crate::database::MssqlArgumentValue;
use crate::decode::Decode;
use crate::encode::{Encode, IsNull};
use crate::error::BoxDynError;
use crate::types::Type;
use crate::value::MssqlData;
use crate::{Mssql, MssqlTypeInfo, MssqlValueRef};

impl Type<Mssql> for BigDecimal {
    fn type_info() -> MssqlTypeInfo {
        MssqlTypeInfo::new("DECIMAL")
    }

    fn compatible(ty: &MssqlTypeInfo) -> bool {
        matches!(
            ty.base_name(),
            "DECIMAL" | "NUMERIC" | "MONEY" | "SMALLMONEY"
        )
    }
}

impl Encode<'_, Mssql> for BigDecimal {
    fn encode_by_ref(&self, buf: &mut Vec<MssqlArgumentValue>) -> Result<IsNull, BoxDynError> {
        buf.push(MssqlArgumentValue::BigDecimal(self.clone()));
        Ok(IsNull::No)
    }
}

impl Decode<'_, Mssql> for BigDecimal {
    fn decode(value: MssqlValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.data {
            // When `rust_decimal` is also enabled it wins the decoder, so NUMERIC
            // is stored as `MssqlData::Decimal`; convert it via its string form.
            #[cfg(not(feature = "rust_decimal"))]
            MssqlData::BigDecimal(ref v) => Ok(v.clone()),
            #[cfg(feature = "rust_decimal")]
            MssqlData::Decimal(v) => v
                .to_string()
                .parse::<BigDecimal>()
                .map_err(|e| format!("failed to convert Decimal to BigDecimal: {e}").into()),
            MssqlData::I32(v) => Ok(BigDecimal::from(*v)),
            MssqlData::I64(v) => Ok(BigDecimal::from(*v)),
            // Go through the shortest round-tripping decimal string rather than
            // `from_f64` (which keeps the full binary artifact, e.g. MONEY
            // 1234.5678 -> 1234.567800000000033...).
            MssqlData::F64(v) => v
                .to_string()
                .parse::<BigDecimal>()
                .map_err(|e| format!("failed to convert f64 {v} to BigDecimal: {e}").into()),
            MssqlData::String(ref s) => s
                .parse::<BigDecimal>()
                .map_err(|e| format!("failed to parse BigDecimal from string: {e}").into()),
            MssqlData::Null => Err("unexpected NULL".into()),
            _ => Err(format!("expected DECIMAL, got {:?}", value.data).into()),
        }
    }
}
