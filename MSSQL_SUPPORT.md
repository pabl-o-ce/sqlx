# MSSQL (SQL Server) Support for SQLx

This document covers all MSSQL/SQL Server additions in the `feat/mssql-support` branch, built on top of the [Tiberius](https://github.com/prisma/tiberius) TDS driver.

---

## Table of Contents

- [Overview](#overview)
- [Getting Started](#getting-started)
- [Connection & Authentication](#connection--authentication)
- [SSL/TLS](#ssltls)
- [Type Mappings](#type-mappings)
- [Compile-Time Query Macros](#compile-time-query-macros)
- [Any Driver Support](#any-driver-support)
- [Migrations](#migrations)
- [Transactions & Isolation Levels](#transactions--isolation-levels)
- [Advisory Locks](#advisory-locks)
- [Bulk Insert](#bulk-insert)
- [QueryBuilder](#querybuilder)
- [XML Type](#xml-type)
- [Examples](#examples)
- [Docker & CI](#docker--ci)
- [Test Coverage](#test-coverage)
- [Feature Flags](#feature-flags)

---

## Overview

Full SQL Server support has been added to SQLx, bringing feature parity with PostgreSQL, MySQL, and SQLite where applicable. The implementation provides:

- Complete type system mapping between Rust and SQL Server types
- Four authentication methods (SQL Server, Windows/NTLM, Integrated/GSSAPI, Azure AD)
- SSL/TLS with configurable modes
- Compile-time checked queries via macros
- Runtime-polymorphic `Any` driver support
- Database migrations with `sqlx migrate`
- RAII advisory locks via `sp_getapplock`/`sp_releaseapplock`
- Bulk insert via the TDS `INSERT BULK` protocol
- Transaction isolation levels
- Testing infrastructure with Docker Compose (MSSQL 2019 & 2022)

**URL schemes:** `mssql://` and `sqlserver://`

---

## Getting Started

Add SQLx with the `mssql` feature to your `Cargo.toml`:

```toml
[dependencies]
sqlx = { version = "0.8", features = ["mssql", "runtime-tokio"] }
```

Connect to a database:

```rust
use sqlx::mssql::MssqlPool;

let pool = MssqlPool::connect("mssql://sa:YourPassword@localhost/mydb").await?;

let row: (i32,) = sqlx::query_as("SELECT @p1")
    .bind(42i32)
    .fetch_one(&pool)
    .await?;
```

---

## Connection & Authentication

**Connection string format:**

```
mssql://[user[:password]@]host[:port][/database][?properties]
```

**Connection options:**

| Option | Default | Description |
|--------|---------|-------------|
| `host` | `localhost` | Database server hostname |
| `port` | `1433` | Port number |
| `username` | `sa` | Username |
| `password` | — | Password |
| `database` | — | Database name |
| `instance` | — | SQL Server named instance |
| `app_name` | `sqlx` | Application name sent to server |
| `statement-cache-capacity` | `100` | Max cached prepared statements |
| `application_intent` | `read_write` | `read_write` or `read_only` (Always On replicas) |

### Authentication Methods

**1. SQL Server Auth (default)**

Standard username/password authentication.

```rust
let pool = MssqlPool::connect("mssql://sa:password@localhost/mydb").await?;
```

**2. Windows/NTLM Auth** (feature: `winauth`)

Supports `domain\user` syntax.

```rust
let opts = MssqlConnectOptions::new()
    .host("localhost")
    .windows_auth(true);
```

**3. Integrated Auth / GSSAPI** (feature: `integrated-auth-gssapi`)

Uses SSPI on Windows and Kerberos on Unix.

```rust
let opts = MssqlConnectOptions::new()
    .host("localhost")
    .integrated_auth(true);
```

**4. Azure AD Token Auth**

Pass a bearer token for Azure Active Directory authentication.

```rust
let opts = MssqlConnectOptions::new()
    .host("your-server.database.windows.net")
    .aad_token("eyJ0eX...");
```

---

## SSL/TLS

Configurable encryption modes for the TDS connection.

| Mode | Description |
|------|-------------|
| `Disabled` | No encryption |
| `LoginOnly` | Encrypt login packet only |
| `Preferred` (default) | Encrypt if server supports it |
| `Required` | Always encrypt, fail otherwise |

**Connection string parameters:**

| Parameter | Description |
|-----------|-------------|
| `sslmode` / `ssl_mode` | `disabled`, `login_only`, `preferred`, `required` |
| `encrypt` | Legacy alias: `true` = required, `false` = disabled |
| `trust_server_certificate` | Trust without validation (default: `false`) |
| `trust_server_certificate_ca` | Path to CA certificate file (`.pem`, `.crt`, `.der`) |

```
mssql://sa:password@localhost/mydb?sslmode=required&trust_server_certificate=true
```

---

## Type Mappings

### Primitive Types

| Rust Type | SQL Server Type(s) |
|-----------|-------------------|
| `bool` | `BIT` |
| `u8` | `TINYINT` (0–255) |
| `i8` | `TINYINT` (0–127) |
| `i16` | `SMALLINT` |
| `i32` | `INT` |
| `i64` | `BIGINT` |
| `f32` | `REAL`, `FLOAT` |
| `f64` | `REAL`, `FLOAT`, `MONEY`, `SMALLMONEY` |
| `&str` / `String` | `NVARCHAR` |
| `&[u8]` / `Vec<u8>` | `VARBINARY` |

### Feature-Gated Types

#### `uuid`

| Rust Type | SQL Server Type |
|-----------|----------------|
| `uuid::Uuid` | `UNIQUEIDENTIFIER` |

#### `rust_decimal`

| Rust Type | SQL Server Type(s) |
|-----------|-------------------|
| `rust_decimal::Decimal` | `DECIMAL`, `NUMERIC`, `MONEY`, `SMALLMONEY` |

#### `bigdecimal`

| Rust Type | SQL Server Type(s) |
|-----------|-------------------|
| `bigdecimal::BigDecimal` | `DECIMAL`, `NUMERIC`, `MONEY` |

#### `chrono`

| Rust Type | SQL Server Type(s) |
|-----------|-------------------|
| `chrono::NaiveDate` | `DATE` |
| `chrono::NaiveTime` | `TIME` |
| `chrono::NaiveDateTime` | `DATETIME2`, `DATETIME`, `SMALLDATETIME` |
| `chrono::DateTime<Utc>` | `DATETIME2`, `DATETIMEOFFSET` |
| `chrono::DateTime<FixedOffset>` | `DATETIMEOFFSET`, `DATETIME2` |

#### `time`

| Rust Type | SQL Server Type(s) |
|-----------|-------------------|
| `time::Date` | `DATE` |
| `time::Time` | `TIME` |
| `time::PrimitiveDateTime` | `DATETIME2`, `DATETIME`, `SMALLDATETIME` |
| `time::OffsetDateTime` | `DATETIMEOFFSET`, `DATETIME2` |

#### `json`

| Rust Type | SQL Server Type |
|-----------|----------------|
| `serde_json::Value` / `Json<T>` | `NVARCHAR` (stored as JSON string) |

#### XML

| Rust Type | SQL Server Type |
|-----------|----------------|
| `MssqlXml` | `XML` |

### Nullable Types

All types above support `Option<T>` for nullable columns.

---

## Compile-Time Query Macros

The standard SQLx macros work with MSSQL when `DATABASE_URL` is set to an `mssql://` connection string:

```rust
// Compile-time checked query
let row = sqlx::query!("SELECT @p1 AS value", 42i32)
    .fetch_one(&pool)
    .await?;

// With custom return type
#[derive(sqlx::FromRow)]
struct User {
    id: i32,
    name: String,
}

let user = sqlx::query_as!(User, "SELECT id, name FROM users WHERE id = @p1", 1i32)
    .fetch_one(&pool)
    .await?;

// Scalar queries
let count = sqlx::query_scalar!("SELECT COUNT(*) FROM users")
    .fetch_one(&pool)
    .await?;
```

**Offline mode** is also supported — run `cargo sqlx prepare` to generate query metadata for CI builds without a live database.

---

## Any Driver Support

MSSQL is fully integrated with the `Any` runtime-polymorphic driver, enabled via the `any` feature flag.

```rust
use sqlx::any::AnyPool;

// Connects to whichever database the URL points to
let pool = AnyPool::connect("mssql://sa:password@localhost/mydb").await?;

let rows = sqlx::query("SELECT 1 + 1 AS result")
    .fetch_all(&pool)
    .await?;
```

All standard operations work through `Any`: queries, transactions, ping, close, and prepared statements.

---

## Migrations

MSSQL supports the full `sqlx migrate` workflow.

```bash
# Create a new migration
sqlx migrate add create_users_table

# Run pending migrations
sqlx migrate run

# Revert the last migration
sqlx migrate revert
```

**Programmatic usage:**

```rust
sqlx::migrate!("./migrations")
    .run(&pool)
    .await?;
```

**Database lifecycle:**

- `create_database(url)` — Creates a database via `CREATE DATABASE [name]`
- `database_exists(url)` — Checks existence via `DB_ID()`
- `drop_database(url)` — Drops with `ALTER DATABASE SET SINGLE_USER WITH ROLLBACK IMMEDIATE` for cleanup

**No-transaction migrations** are supported for DDL operations that cannot run inside a transaction.

---

## Transactions & Isolation Levels

Standard transaction support with configurable isolation levels.

```rust
let mut tx = pool.begin().await?;

sqlx::query("INSERT INTO users (name) VALUES (@p1)")
    .bind("Alice")
    .execute(&mut *tx)
    .await?;

tx.commit().await?;
```

### Isolation Levels

| Level | Description |
|-------|-------------|
| `ReadUncommitted` | Dirty reads allowed |
| `ReadCommitted` | Default SQL Server isolation |
| `RepeatableRead` | Prevents non-repeatable reads |
| `Snapshot` | Row versioning-based isolation |
| `Serializable` | Strictest isolation |

```rust
use sqlx::mssql::MssqlIsolationLevel;

let mut tx = pool
    .begin_with_isolation(MssqlIsolationLevel::Snapshot)
    .await?;
```

---

## Advisory Locks

Application-level named locks using SQL Server's `sp_getapplock` and `sp_releaseapplock`, with an RAII guard pattern.

### Lock Modes

| Mode | Compatible With |
|------|----------------|
| `Shared` | Shared, Update |
| `Update` | Shared only |
| `Exclusive` (default) | None |

### Usage

```rust
use sqlx::mssql::{MssqlAdvisoryLock, MssqlAdvisoryLockMode};

// Create an exclusive lock
let lock = MssqlAdvisoryLock::new("my_resource");

// Or with a specific mode
let lock = MssqlAdvisoryLock::with_mode("my_resource", MssqlAdvisoryLockMode::Shared);

// RAII guard (preferred) — lock released when guard is dropped
let guard = lock.acquire_guard(&mut conn).await?;
// ... do work while lock is held ...
let conn = guard.release_now().await?; // explicit release

// Non-blocking attempt
if let Some(guard) = lock.try_acquire_guard(&mut conn).await? {
    // lock acquired
}
```

---

## Bulk Insert

High-performance data loading via the TDS `INSERT BULK` protocol.

```rust
let mut bulk = conn.bulk_insert("my_table").await?;

for item in &data {
    bulk.send(tiberius::IntoRow::into_row(item)).await?;
}

let rows_affected = bulk.finalize().await?;
```

Supports tuples up to 10 elements via `tiberius::IntoRow`.

---

## QueryBuilder

MSSQL uses `@p1`, `@p2`, etc. as parameter placeholders. The `QueryBuilder` handles this automatically:

```rust
let mut qb = QueryBuilder::<Mssql>::new("SELECT * FROM users WHERE ");
qb.push("name = ").push_bind("Alice");
qb.push(" AND age > ").push_bind(21);
// Produces: SELECT * FROM users WHERE name = @p1 AND age > @p2
```

---

## XML Type

A dedicated `MssqlXml` wrapper type distinguishes XML columns from regular strings.

```rust
use sqlx::mssql::MssqlXml;

let xml = MssqlXml::from("<root><item>hello</item></root>".to_string());

sqlx::query("INSERT INTO docs (content) VALUES (@p1)")
    .bind(&xml)
    .execute(&pool)
    .await?;

let result: MssqlXml = sqlx::query_scalar("SELECT content FROM docs")
    .fetch_one(&pool)
    .await?;
```

---

## Examples

A full CRUD Todo application is available at `examples/mssql/todos/`, demonstrating:

- Connection pooling
- Migrations
- Query execution
- Error handling

---

## Docker & CI

### Docker Compose

The test suite includes Docker Compose configurations for MSSQL 2019 and 2022:

```bash
docker compose -f tests/docker-compose.yml up mssql_2022 -d
```

**Services:**

| Service | Image | Port |
|---------|-------|------|
| `mssql_2022` | `mcr.microsoft.com/mssql/server:2022-latest` | 1433 |
| `mssql_2019` | `mcr.microsoft.com/mssql/server:2019-latest` | 1433 |

### CI Matrix

The GitHub Actions workflow tests across:

- **MSSQL versions:** 2019, 2022
- **Async runtimes:** tokio, async-global-executor, smol
- **TLS backends:** native-tls, rustls-aws-lc-rs, rustls-ring, none

---

## Test Coverage

Comprehensive test suite in `tests/mssql/`:

| Area | File | What's Tested |
|------|------|---------------|
| Core queries | `mssql.rs` | Connections, SELECT, INSERT, parameters, large result sets, error handling |
| Type round-trips | `types.rs` | All primitive and feature-gated types with boundary values, NULLs, Unicode, large data |
| Test attribute | `test-attr.rs` | `#[sqlx_macros::test]` macro with automatic test DB setup |
| Isolation levels | `isolation-level.rs` | All five isolation level configurations |
| Advisory locks | `advisory-lock.rs` | Acquire, release, guard pattern, all lock modes |
| Bulk insert | `bulk-insert.rs` | High-performance loading, multi-row operations |
| Derives | `derives.rs` | `#[derive(FromRow)]`, custom field mappings |
| Query builder | `query_builder.rs` | Dynamic query construction, parameter handling |
| Error handling | `error.rs` | Database error inspection, error details |
| Compile-time macros | `tests/mssql-macros/` | Online and offline macro verification |

---

## Feature Flags

| Feature | Description |
|---------|-------------|
| `mssql` | Enable the MSSQL driver |
| `any` | Enable runtime-polymorphic `Any` driver |
| `migrate` | Enable database migrations |
| `json` | JSON type support via `serde_json` |
| `uuid` | `uuid::Uuid` type support |
| `chrono` | `chrono` datetime types |
| `time` | `time` crate datetime types |
| `rust_decimal` | `rust_decimal::Decimal` support |
| `bigdecimal` | `bigdecimal::BigDecimal` support |
| `winauth` | Windows/NTLM authentication |
| `integrated-auth-gssapi` | Integrated auth (Kerberos on Unix, SSPI on Windows) |
| `offline` | Offline mode for compile-time macros |
