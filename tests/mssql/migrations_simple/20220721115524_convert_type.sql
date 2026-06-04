-- Perform a tricky conversion of the payload.
--
-- This script will only succeed once and will fail if executed twice.

-- set up temporary target column
ALTER TABLE migrations_simple_test
ADD some_payload_tmp NVARCHAR(MAX);

-- perform conversion
-- This will fail if `some_payload` is already a string column due to the addition.
-- We add a suffix after the addition to ensure that the SQL database does not silently cast the string back to an
-- integer.
-- Wrapped in EXEC() because a column added by ALTER ... ADD is not visible to
-- DML in the same batch (SQL Server resolves column names at batch compile).
EXEC('UPDATE migrations_simple_test SET some_payload_tmp = CONCAT(CAST((some_payload + 10) AS VARCHAR(3)), ''_suffix'')');

-- remove original column including the content
ALTER TABLE migrations_simple_test
DROP COLUMN some_payload;

-- prepare new payload column (nullable, so we can copy over the data)
ALTER TABLE migrations_simple_test
ADD some_payload NVARCHAR(MAX);

-- copy new values (EXEC for the same reason as above)
EXEC('UPDATE migrations_simple_test SET some_payload = some_payload_tmp');

-- "freeze" column: MSSQL uses sp_rename + re-add or ALTER COLUMN for NOT NULL
ALTER TABLE migrations_simple_test
ALTER COLUMN some_payload NVARCHAR(MAX) NOT NULL;

-- clean up
ALTER TABLE migrations_simple_test
DROP COLUMN some_payload_tmp;
