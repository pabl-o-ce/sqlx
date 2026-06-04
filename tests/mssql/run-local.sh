#!/usr/bin/env bash
#
# Run the MSSQL integration tests locally, mirroring the CI `MSSQL` job in
# .github/workflows/mssql.yml. Spins up the same SQL Server container, waits for
# the `sqlx` database to be created, then runs only the mssql-using integration
# tests with the same features and environment as CI.
#
# Usage:
#   tests/mssql/run-local.sh [extra cargo test args...]   # run the suite
#   tests/mssql/run-local.sh --test mssql                 # run one target
#   tests/mssql/run-local.sh --down                       # stop & remove container
#   MSSQL_VERSION=2019 tests/mssql/run-local.sh           # use the 2019 image
#
# The container is left running between invocations for fast iteration; use
# `--down` to remove it.
set -euo pipefail

MSSQL_VERSION="${MSSQL_VERSION:-2022}"
SERVICE="mssql_${MSSQL_VERSION}"
CONTAINER="sqlx_mssql_local_${MSSQL_VERSION}"
PASSWORD='YourStrong!Passw0rd'
FEATURES="any,mssql,macros,migrate,_unstable-all-types,runtime-tokio,tls-none"

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

if [[ "${1:-}" == "--down" ]]; then
    echo ">> removing container $CONTAINER"
    docker rm -f "$CONTAINER" >/dev/null 2>&1 || true
    exit 0
fi

# Start the container if it isn't already running.
if ! docker ps --format '{{.Names}}' | grep -qx "$CONTAINER"; then
    echo ">> starting $SERVICE as $CONTAINER (first run builds the image)"
    docker rm -f "$CONTAINER" >/dev/null 2>&1 || true
    docker compose -f tests/docker-compose.yml run -d -p 1433:1433 --name "$CONTAINER" "$SERVICE"
else
    echo ">> reusing running container $CONTAINER"
fi

# Poll until the `sqlx` database exists. The container's configure-db.sh sleeps
# ~60s then creates it; use the same sqlcmd the image ships (tools18, which
# encrypts by default -> -C), with $SA_PASSWORD already set inside the container.
echo ">> waiting for the 'sqlx' database..."
ready=
for i in $(seq 1 60); do
    if docker exec "$CONTAINER" bash -c '
        S=/opt/mssql-tools18/bin/sqlcmd; C=-C
        [ -x "$S" ] || { S=/opt/mssql-tools/bin/sqlcmd; C=; }
        "$S" $C -S localhost -U sa -P "$SA_PASSWORD" -d sqlx -Q "SELECT 1" -b
    ' >/dev/null 2>&1; then
        echo ">> sqlx database ready (after ~$((i * 5))s)"
        ready=1
        break
    fi
    sleep 5
done
if [[ -z "$ready" ]]; then
    echo "!! sqlx database never came up; last container logs:" >&2
    docker logs "$CONTAINER" 2>&1 | tail -30 >&2
    exit 1
fi

mkdir -p .sqlx
export DATABASE_URL="mssql://sa:${PASSWORD}@localhost:1433/sqlx?sslmode=disabled"
export SQLX_OFFLINE_DIR=".sqlx"
export RUSTFLAGS="--cfg mssql_${MSSQL_VERSION}"

echo ">> cargo test (features: $FEATURES)"
cargo test --no-default-features --features "$FEATURES" "$@"
