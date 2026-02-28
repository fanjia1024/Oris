#!/usr/bin/env sh
set -eu

PG_CONTAINER="${PG_CONTAINER:-pgvector}"
PGDATABASE_NAME="${PGDATABASE_NAME:-oris}"
PGDATABASE_USER="${PGDATABASE_USER:-username}"
PGDATABASE_PASSWORD="${PGDATABASE_PASSWORD:-password}"
RUNTIME_SCHEMA="${RUNTIME_SCHEMA:-oris_runtime_backup_rehearsal}"
BACKUP_FILE="${BACKUP_FILE:-/tmp/${RUNTIME_SCHEMA}.sql}"

case "$RUNTIME_SCHEMA" in
  ""|*[!A-Za-z0-9_]*)
    echo "RUNTIME_SCHEMA must match [A-Za-z0-9_]+" >&2
    exit 1
    ;;
  [0-9]*)
    echo "RUNTIME_SCHEMA must not start with a digit" >&2
    exit 1
    ;;
esac

seed_file="$(mktemp "/tmp/${RUNTIME_SCHEMA}.seed.XXXXXX.sql")"
trap 'rm -f "$seed_file"' EXIT

cat > "$seed_file" <<EOF
DROP SCHEMA IF EXISTS "$RUNTIME_SCHEMA" CASCADE;
CREATE SCHEMA "$RUNTIME_SCHEMA";

CREATE TABLE "$RUNTIME_SCHEMA".runtime_schema_migrations (
  version INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  applied_at_ms BIGINT NOT NULL
);

CREATE TABLE "$RUNTIME_SCHEMA".runtime_attempts (
  attempt_id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  attempt_no INTEGER NOT NULL,
  status TEXT NOT NULL,
  retry_at_ms BIGINT NULL
);

CREATE TABLE "$RUNTIME_SCHEMA".runtime_leases (
  lease_id TEXT PRIMARY KEY,
  attempt_id TEXT NOT NULL UNIQUE,
  worker_id TEXT NOT NULL,
  lease_expires_at_ms BIGINT NOT NULL,
  heartbeat_at_ms BIGINT NOT NULL,
  version BIGINT NOT NULL
);

CREATE INDEX idx_runtime_attempts_status_retry
  ON "$RUNTIME_SCHEMA".runtime_attempts(status, retry_at_ms);
CREATE INDEX idx_runtime_leases_expiry
  ON "$RUNTIME_SCHEMA".runtime_leases(lease_expires_at_ms);

INSERT INTO "$RUNTIME_SCHEMA".runtime_schema_migrations(version, name, applied_at_ms)
VALUES
  (1, 'baseline_runtime_tables', 1),
  (2, 'runtime_dispatch_indexes', 2);

INSERT INTO "$RUNTIME_SCHEMA".runtime_attempts(attempt_id, run_id, attempt_no, status, retry_at_ms)
VALUES
  ('attempt-backup-queued', 'run-backup-queued', 1, 'queued', NULL),
  ('attempt-backup-leased', 'run-backup-leased', 1, 'leased', NULL);

INSERT INTO "$RUNTIME_SCHEMA".runtime_leases(
  lease_id,
  attempt_id,
  worker_id,
  lease_expires_at_ms,
  heartbeat_at_ms,
  version
)
VALUES (
  'lease-backup-restore',
  'attempt-backup-leased',
  'worker-backup',
  32503680000000,
  1700000000000,
  3
);
EOF

docker exec "$PG_CONTAINER" true >/dev/null

docker exec \
  -e PGPASSWORD="$PGDATABASE_PASSWORD" \
  -i "$PG_CONTAINER" \
  psql -v ON_ERROR_STOP=1 -U "$PGDATABASE_USER" -d "$PGDATABASE_NAME" \
  < "$seed_file" >/dev/null

docker exec \
  -e PGPASSWORD="$PGDATABASE_PASSWORD" \
  "$PG_CONTAINER" \
  pg_dump -U "$PGDATABASE_USER" -d "$PGDATABASE_NAME" \
  --schema "$RUNTIME_SCHEMA" --format=plain --no-owner --no-privileges \
  > "$BACKUP_FILE"

test -s "$BACKUP_FILE"

docker exec \
  -e PGPASSWORD="$PGDATABASE_PASSWORD" \
  "$PG_CONTAINER" \
  psql -v ON_ERROR_STOP=1 -U "$PGDATABASE_USER" -d "$PGDATABASE_NAME" \
  -c "DROP SCHEMA IF EXISTS \"$RUNTIME_SCHEMA\" CASCADE;" >/dev/null

docker exec \
  -e PGPASSWORD="$PGDATABASE_PASSWORD" \
  -i "$PG_CONTAINER" \
  psql -v ON_ERROR_STOP=1 -U "$PGDATABASE_USER" -d "$PGDATABASE_NAME" \
  < "$BACKUP_FILE" >/dev/null

migration_rows="$(
  docker exec -e PGPASSWORD="$PGDATABASE_PASSWORD" "$PG_CONTAINER" \
    psql -t -A -U "$PGDATABASE_USER" -d "$PGDATABASE_NAME" \
    -c "SELECT COUNT(*) FROM \"$RUNTIME_SCHEMA\".runtime_schema_migrations"
)"
attempt_rows="$(
  docker exec -e PGPASSWORD="$PGDATABASE_PASSWORD" "$PG_CONTAINER" \
    psql -t -A -U "$PGDATABASE_USER" -d "$PGDATABASE_NAME" \
    -c "SELECT COUNT(*) FROM \"$RUNTIME_SCHEMA\".runtime_attempts"
)"
lease_rows="$(
  docker exec -e PGPASSWORD="$PGDATABASE_PASSWORD" "$PG_CONTAINER" \
    psql -t -A -U "$PGDATABASE_USER" -d "$PGDATABASE_NAME" \
    -c "SELECT COUNT(*) FROM \"$RUNTIME_SCHEMA\".runtime_leases"
)"
queued_ready="$(
  docker exec -e PGPASSWORD="$PGDATABASE_PASSWORD" "$PG_CONTAINER" \
    psql -t -A -U "$PGDATABASE_USER" -d "$PGDATABASE_NAME" \
    -c "SELECT COUNT(*) FROM \"$RUNTIME_SCHEMA\".runtime_attempts a LEFT JOIN \"$RUNTIME_SCHEMA\".runtime_leases l ON l.attempt_id = a.attempt_id WHERE a.status = 'queued' AND l.attempt_id IS NULL"
)"
owned_leases="$(
  docker exec -e PGPASSWORD="$PGDATABASE_PASSWORD" "$PG_CONTAINER" \
    psql -t -A -U "$PGDATABASE_USER" -d "$PGDATABASE_NAME" \
    -c "SELECT COUNT(*) FROM \"$RUNTIME_SCHEMA\".runtime_leases WHERE worker_id = 'worker-backup'"
)"

echo "backup_restore_rehearsal=ok"
echo "schema=$RUNTIME_SCHEMA"
echo "backup_file=$BACKUP_FILE"
echo "migration_rows=$migration_rows"
echo "attempt_rows=$attempt_rows"
echo "lease_rows=$lease_rows"
echo "queued_ready=$queued_ready"
echo "owned_leases=$owned_leases"
