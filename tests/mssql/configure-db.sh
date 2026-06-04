#!/usr/bin/env bash

# Wait 60 seconds for SQL Server to start up
sleep 60

# Recent mcr.microsoft.com/mssql/server images ship sqlcmd at
# /opt/mssql-tools18 (ODBC Driver 18, which encrypts by default -> needs -C to
# trust the container's self-signed cert); older images used /opt/mssql-tools.
if [ -x /opt/mssql-tools18/bin/sqlcmd ]; then
    SQLCMD=(/opt/mssql-tools18/bin/sqlcmd -C)
else
    SQLCMD=(/opt/mssql-tools/bin/sqlcmd)
fi

# Run the setup script to create the DB and the schema in the DB
"${SQLCMD[@]}" -S localhost -U sa -P "$SA_PASSWORD" -d master -i setup.sql
