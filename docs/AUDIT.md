# Audit Journal and Idempotency Cache

Status: implemented local spine. Real GHL write commands are still being built, so the current commands mainly inspect and manage local state.

## Audit journal

The resolved audit directory is reported by:

```bash
ghl config path --pretty
```

The journal file is stored at:

```text
<audit_dir>/audit.jsonl
```

Each line is one redacted JSON audit entry. On Unix-like systems, newly created journal/export files use owner-only `0600` permissions. Secrets, tokens, message bodies, and sensitive request fields are redacted before writing.

Implemented commands:

```bash
ghl audit list [--from <datetime>] [--to <datetime>] [--action <name>] [--resource <id>] [--limit <n>]
ghl audit show <entry-id>
ghl audit export [--from <datetime>] [--to <datetime>] [--action <name>] [--resource <id>] [--out <path>]
```

`--from` and `--to` accept RFC3339 datetimes or Unix milliseconds. `audit export` writes a redacted JSON array when `--out` is provided; without `--out`, matching entries are returned in the normal JSON envelope.

## Idempotency cache

The local cache file is stored at:

```text
<data_dir>/idempotency/idempotency.jsonl
```

Records are keyed by profile, location, command, user-provided idempotency key, and a stable redacted request hash. Reusing a key with a different request hash fails before mutation.

Implemented commands:

```bash
ghl idempotency list
ghl idempotency show <key>
ghl idempotency clear <key> --yes
```

`idempotency clear` accepts either the raw key or scoped key and requires explicit confirmation.
