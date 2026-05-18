#!/usr/bin/env bash

# Wait 60 seconds for SQL Server to start up
sleep 60

# Locate sqlcmd: newer mssql/server images ship mssql-tools18 (sqlcmd in
# /opt/mssql-tools18/bin); older images shipped mssql-tools (no version
# suffix). Prefer the newer one.
if [ -x /opt/mssql-tools18/bin/sqlcmd ]; then
    SQLCMD=/opt/mssql-tools18/bin/sqlcmd
    # mssql-tools18 requires explicit cert handling — trust the self-signed
    # cert the image ships with.
    SQLCMD_TLS_ARGS=(-C)
elif [ -x /opt/mssql-tools/bin/sqlcmd ]; then
    SQLCMD=/opt/mssql-tools/bin/sqlcmd
    SQLCMD_TLS_ARGS=()
else
    echo "configure-db.sh: no sqlcmd binary found under /opt/mssql-tools*" >&2
    exit 1
fi

# Run the setup script to create the DB and the schema in the DB
"$SQLCMD" "${SQLCMD_TLS_ARGS[@]}" -S localhost -U sa -P "$SA_PASSWORD" -d master -i setup.sql
