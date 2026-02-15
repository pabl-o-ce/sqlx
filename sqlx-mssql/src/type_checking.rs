// Type mappings used by the macros and `Debug` impls.

#[allow(unused_imports)]
use sqlx_core as sqlx;

use crate::Mssql;

impl_type_checking!(
    Mssql {
        u8,
        i8,
        i16,
        i32,
        i64,
        f32,
        f64,

        // ordering is important here as otherwise we might infer strings to be binary
        // NVARCHAR, VARCHAR, NCHAR, CHAR, NTEXT, TEXT
        String,

        // VARBINARY, BINARY, IMAGE
        Vec<u8>,

        #[cfg(feature = "uuid")]
        sqlx::types::Uuid,
    },
    ParamChecking::Weak,
    feature-types: _info => None,
    datetime-types: {
        chrono: {
            sqlx::types::chrono::NaiveTime,
            sqlx::types::chrono::NaiveDate,
            sqlx::types::chrono::NaiveDateTime,
            sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>,
        },
        time: { },
    },
    numeric-types: {
        bigdecimal: { },
        rust_decimal: {
            sqlx::types::Decimal,
        },
    },
);
