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
//! | `f32`                                 | REAL                                                 |
//! | `f64`                                 | FLOAT                                                |
//! | `&str`, [`String`]                    | NVARCHAR                                             |
//! | `&[u8]`, `Vec<u8>`                   | VARBINARY                                            |
//!
//! ### Feature-gated
//!
//! | Rust type                             | MSSQL type(s)                                        |
//! |---------------------------------------|------------------------------------------------------|
//! | `uuid::Uuid`                          | UNIQUEIDENTIFIER                                     |
//! | `rust_decimal::Decimal`               | DECIMAL, NUMERIC, MONEY                              |
//!
//! # Nullable
//!
//! In addition, `Option<T>` is supported where `T` implements `Type`. An `Option<T>` represents
//! a potentially `NULL` value from MSSQL.

pub(crate) use sqlx_core::types::*;

mod bool;
mod bytes;
#[cfg(feature = "chrono")]
mod chrono;
mod float;
mod int;
#[cfg(feature = "rust_decimal")]
mod rust_decimal;
mod str;
#[cfg(feature = "uuid")]
mod uuid;
