# API Coverage

Status: Phase 1/2 seed coverage.

The bundled endpoint manifest currently includes:

- `locations.get`
- `locations.search`
- `contacts.search`
- `contacts.get`
- `conversations.search`
- `conversations.get`
- `conversations.messages`

They back PIT validation, location get/list/search, contact reads, and the first conversation/message read commands. Additional endpoint records will be added with each auth and CRM slice.

Inspect current coverage:

```bash
ghl endpoints coverage --pretty
```

The parity map lives in `docs/FEATURE-PARITY.md`.
