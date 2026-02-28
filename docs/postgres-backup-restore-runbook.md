# PostgreSQL Backup and Restore Runbook

This runbook documents the backup, restore, and validation path for Oris runtime data stored in PostgreSQL.

## Scope

- Runtime schema: `runtime_schema_migrations`, `runtime_attempts`, `runtime_leases`
- Default local container: `pgvector` from `sh scripts/run-pgvector`
- Default local database: `oris`

## Rehearsal Script

Use the rehearsal script to run one full backup-to-restore cycle against a PostgreSQL schema and verify that runtime data is still ready for execution after restore:

```bash
sh scripts/rehearse-postgres-backup-restore.sh
```

Environment overrides:

- `PG_CONTAINER` (default `pgvector`)
- `PGDATABASE_NAME` (default `oris`)
- `PGDATABASE_USER` (default `username`)
- `PGDATABASE_PASSWORD` (default `password`)
- `RUNTIME_SCHEMA` (default `oris_runtime_backup_rehearsal`)
- `BACKUP_FILE` (default `/tmp/oris_runtime_backup_rehearsal.sql`)

Success criteria:

- Backup file is created and non-empty.
- Schema can be dropped and restored from the captured dump.
- Restored schema still contains migration history, queued work, and lease ownership rows.
- One queued attempt remains ready for dispatch after restore.
- The script is repeatable; it drops and recreates the rehearsal schema on each run.

## Manual Backup

If you need to back up an existing runtime schema manually from the local Docker container:

```bash
docker exec -e PGPASSWORD=password pgvector \
  pg_dump -U username -d oris \
  --schema oris_runtime \
  --format=plain --no-owner --no-privileges \
  > /tmp/oris_runtime.sql
```

## Manual Restore

1. Stop Oris API and worker processes so runtime state is quiescent.
2. Drop or move aside the target schema if you are restoring in-place.
3. Restore the captured dump into PostgreSQL.

```bash
docker exec -e PGPASSWORD=password pgvector \
  psql -v ON_ERROR_STOP=1 -U username -d oris \
  -c 'DROP SCHEMA IF EXISTS "oris_runtime" CASCADE;'

docker exec -e PGPASSWORD=password -i pgvector \
  psql -v ON_ERROR_STOP=1 -U username -d oris \
  < /tmp/oris_runtime.sql
```

4. Restart Oris services after the validation queries pass.

## Validation Queries

Run these checks before re-enabling traffic:

```bash
docker exec -e PGPASSWORD=password pgvector \
  psql -t -A -U username -d oris \
  -c 'SELECT COUNT(*) FROM "oris_runtime".runtime_schema_migrations'

docker exec -e PGPASSWORD=password pgvector \
  psql -t -A -U username -d oris \
  -c 'SELECT COUNT(*) FROM "oris_runtime".runtime_attempts'

docker exec -e PGPASSWORD=password pgvector \
  psql -t -A -U username -d oris \
  -c 'SELECT COUNT(*) FROM "oris_runtime".runtime_leases'
```

If the restored runtime schema contains queued attempts with no active lease, workers can resume dispatch safely after the restore.

## Recorded Rehearsal

- Date: February 28, 2026
- Environment: local `pgvector` Docker container started from `sh scripts/run-pgvector`
- Command: `sh scripts/rehearse-postgres-backup-restore.sh`
- Backup file: `/tmp/oris_runtime_backup_rehearsal.sql`
- Observed result:

```text
backup_restore_rehearsal=ok
schema=oris_runtime_backup_rehearsal
backup_file=/tmp/oris_runtime_backup_rehearsal.sql
migration_rows=2
attempt_rows=2
lease_rows=1
queued_ready=1
owned_leases=1
```

That result means the restored schema preserved migration history, one active lease, and one queued attempt that is immediately dispatchable when Oris processes start again.
