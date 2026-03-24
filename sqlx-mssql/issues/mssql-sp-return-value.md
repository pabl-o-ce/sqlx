# MSSQL: surface stored procedure return values through the executor

## Context

`sp_getapplock` communicates success/failure through its **return value**, not through SQL errors:

| Return code | Meaning |
|---|---|
| `0` | Lock granted immediately |
| `1` | Lock granted after waiting |
| `-1` | Timed out |
| `-2` | Cancelled |
| `-3` | Deadlock victim |
| `-999` | Parameter validation error |

## Current workaround

We wrap the call in a `DECLARE @r / IF @r < 0 THROW` pattern so that a failed lock becomes a SQL error that `execute` can catch:

```sql
DECLARE @r INT;
EXEC @r = sp_getapplock @Resource = 'sqlx_migrations',
    @LockMode = 'Exclusive', @LockOwner = 'Session', @LockTimeout = -1;
IF @r < 0 THROW 50000, 'Failed to acquire migration lock', 1;
```

This is sufficient for production use — the lock works correctly in all realistic scenarios, and failures are now surfaced as errors instead of being silently ignored.

## Ideal long-term solution

The proper fix is for the MSSQL executor to capture the TDS `RETURNSTATUS` token that SQL Server sends after stored procedure execution, and expose it through the driver's result types.

### What would need to change

1. **`collect_results` in `executor.rs`** — currently only handles `QueryItem::Metadata` and `QueryItem::Row`. The TDS return status token is not surfaced by tiberius's `QueryStream`. Investigate whether tiberius exposes this via `ExecuteResult` (from `.execute()`) or if it requires upstream changes.

2. **`MssqlQueryResult`** — currently only holds `rows_affected: u64`. Would need an additional field like `return_status: Option<i32>` to carry the stored procedure return value.

3. **`Migrate::lock` trait** — the signature is `Result<(), MigrateError>`, which is fine (we'd just check the return status and map negatives to `Err`). No trait changes needed.

### Why this matters beyond migrations

Any user calling stored procedures via `execute` today cannot inspect return values. This is a general limitation of the MSSQL driver, not specific to migrations. The `THROW` workaround only works when you control the SQL — it doesn't help when calling third-party procedures that use return codes for flow control.

## Priority

**Low** — the THROW workaround fully covers the migration lock case, and stored procedure return values are a niche use case. This is a correctness/completeness improvement, not a bug fix.
