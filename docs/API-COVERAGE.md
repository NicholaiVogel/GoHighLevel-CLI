# API Coverage

Status: Phase 1/2 seed coverage.

The bundled endpoint manifest currently includes:

- `locations.get`
- `locations.search`
- `contacts.search` (contacts list/search)
- `contacts.get`
- `conversations.search`
- `conversations.get`
- `conversations.messages`
- `pipelines.list`
- `opportunities.search`
- `opportunities.get`
- `calendars.list`
- `calendars.get`
- `calendars.events`
- `calendars.free_slots`
- `users.list`
- `users.get`
- `users.search`
- `users.filter_by_email`

They back PIT validation, location get/list/search, contact reads, conversation/message reads, pipeline reads, opportunity reads, calendar reads, and user/team-member reads. Additional endpoint records will be added with each auth and CRM slice.

Inspect current coverage:

```bash
ghl endpoints coverage --pretty
```

The parity map lives in `docs/FEATURE-PARITY.md`.
