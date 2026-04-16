# API Coverage

Status: Phase 1/2 seed coverage.

The bundled endpoint manifest currently includes `locations.get` as the first implemented record. It backs both `auth pit validate` and `locations get`. Additional endpoint records will be added with each auth and CRM slice.

Inspect current coverage:

```bash
ghl endpoints coverage --pretty
```

The parity map lives in `docs/FEATURE-PARITY.md`.
