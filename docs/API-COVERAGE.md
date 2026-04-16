# API Coverage

Status: Phase 1/2 seed coverage.

The bundled endpoint manifest currently includes `locations.get` and `locations.search`. They back PIT validation, location get, location list, and email-filtered location search. Additional endpoint records will be added with each auth and CRM slice.

Inspect current coverage:

```bash
ghl endpoints coverage --pretty
```

The parity map lives in `docs/FEATURE-PARITY.md`.
