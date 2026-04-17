# Network Behavior

The CLI now supports explicit read-only network requests through PIT validation, `raw request` GET, typed location get/list/search, typed contact list/search/get, typed conversation search/get/messages, typed pipeline list/get, typed opportunity search/get, typed calendar list/get/events/free-slots, guarded appointment create, typed user/team-member list/get/search, `doctor api`, and smoke checks. Mutating raw methods are not implemented yet.

Requests use the selected profile base URL, `Authorization: Bearer <token>`, `Accept: application/json`, `Content-Type: application/json`, `Version: 2021-07-28`, and a browser-compatible user agent. Authorization and token-looking values are redacted from diagnostics and outputs.

Future network behavior is defined in `docs/SPEC.md`.


`locations list` and `locations search` use `GET /locations/search`. The current upstream search filter exposed by the reference client is email, so `locations search <email>` maps the query to that filter.

`contacts list` uses `POST /contacts/search` with only `locationId`, `pageLimit`, and optional cursors, then returns counts and contact ids without printing contact bodies. `contacts search` uses the same read-only endpoint with optional fuzzy `query` and exact filters encoded as `filters: [{ field, operator: "eq", value }]` for email and phone. This is a read operation even though the upstream endpoint uses POST. `contacts get` uses `GET /contacts/{contact_id}`. These commands require resolved location context from `--location` or the active profile before they run.

`conversations search` uses `GET /conversations/search` with `locationId`, `status`, `limit`, and optional contact/query/assignment/message-type filters. `conversations get` uses `GET /conversations/{conversation_id}`. `conversations messages` uses `GET /conversations/{conversation_id}/messages` with limit and cursor filters. Conversation message bodies and preview bodies are redacted in normal JSON output.

`pipelines list` uses `GET /opportunities/pipelines?locationId=...`. `pipelines get` uses the same endpoint and filters the returned pipeline list client-side because the referenced API exposes pipeline read as a list operation. `opportunities search` uses `GET /opportunities/search` with underscore query names such as `location_id`, `pipeline_id`, `pipeline_stage_id`, and `contact_id`. `opportunities get` uses `GET /opportunities/{opportunity_id}`. Opportunity notes are redacted in normal JSON output.

`calendars list` uses `GET /calendars/?locationId=...` with optional group and draft filters. `calendars get` uses `GET /calendars/{calendar_id}`. `calendars events` uses `GET /calendars/events` with location context plus a date or from/to range, and returns event IDs/counts instead of appointment bodies. `calendars free-slots` uses `GET /calendars/{calendar_id}/free-slots` with a UTC date range and optional timezone.

`users list` and `teams list` use `GET /users/?locationId=...` and return only counts and user ids. The live endpoint rejected server-side pagination and filters during validation, so `--skip` and `--limit` are applied client-side to the sanitized id list. `users get` uses `GET /users/{user_id}` and redacts token-like fields. `users search --email` uses the location-scoped `POST /users/search/filter-by-email`; `users search --query` uses the company-scoped `GET /users/search`. User names and emails are PII, so smoke output stays summary-only.

`doctor api` reuses the smoke runner required safe-read checks and returns diagnostic status only. `doctor`, `doctor endpoint`, `doctor bundle`, and `capabilities` are local-only unless a future option explicitly opts into safe live probes.
