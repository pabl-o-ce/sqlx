//! Conversions between Rust and **MSSQL** types.
//!
//! # Types
//!
//! | Rust type                             | MSSQL type(s)                                        |
//! |---------------------------------------|------------------------------------------------------|
//! | `bool`                                | BIT                                                  |
//! | `u8`                                  | TINYINT (unsigned, 0-255)                            |
//! | `i8`                                  | TINYINT (0-127 only)                                 |
//! | `i16`                                 | SMALLINT                                             |
//! | `i32`                                 | INT                                                  |
//! | `i64`                                 | BIGINT                                               |
//! | `f32`                                 | REAL, FLOAT                                          |
//! | `f64`                                 | REAL, FLOAT, MONEY, SMALLMONEY                       |
//! | `&str`, [`String`]                    | NVARCHAR                                             |
//! | `&[u8]`, `Vec<u8>`                   | VARBINARY                                            |
//!
//! ### Feature-gated
//!
//! | Rust type                             | MSSQL type(s)                                        |
//! |---------------------------------------|------------------------------------------------------|
//! | `uuid::Uuid`                          | UNIQUEIDENTIFIER                                     |
//! | `rust_decimal::Decimal`               | DECIMAL, NUMERIC, MONEY                              |
//! | `bigdecimal::BigDecimal`              | DECIMAL, NUMERIC, MONEY                              |
//! | `time::Date`                          | DATE                                                 |
//! | `time::Time`                          | TIME                                                 |
//! | `time::PrimitiveDateTime`             | DATETIME2, DATETIME, SMALLDATETIME                   |
//! | `time::OffsetDateTime`                | DATETIMEOFFSET, DATETIME2                            |
//! | `serde_json::Value` (`Json<T>`)       | NVARCHAR (JSON stored as string)                     |
//!
//! # Nullable
//!
//! In addition, `Option<T>` is supported where `T` implements `Type`. An `Option<T>` represents
//! a potentially `NULL` value from MSSQL.

pub(crate) use sqlx_core::types::*;

#[cfg(feature = "bigdecimal")]
mod bigdecimal;
mod bool;
mod bytes;
#[cfg(feature = "chrono")]
mod chrono;
mod float;
mod int;
#[cfg(feature = "json")]
mod json;
#[cfg(feature = "rust_decimal")]
mod rust_decimal;
mod str;
#[cfg(feature = "time")]
mod time;
#[cfg(feature = "uuid")]
mod uuid;
pub mod xml;
