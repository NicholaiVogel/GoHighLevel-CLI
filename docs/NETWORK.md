# Network Behavior

The CLI now supports explicit read-only network requests through PIT validation, `raw request` GET, typed location get/list/search, typed contact search/get, and typed conversation search/get/messages. Mutating raw methods are not implemented yet.

Requests use the selected profile base URL, `Authorization: Bearer <token>`, `Accept: application/json`, `Content-Type: application/json`, `Version: 2021-07-28`, and a browser-compatible user agent. Authorization and token-looking values are redacted from diagnostics and outputs.

Future network behavior is defined in `docs/SPEC.md`.


`locations list` and `locations search` use `GET /locations/search`. The current upstream search filter exposed by the reference client is email, so `locations search <email>` maps the query to that filter.

`contacts search` uses the reference client's read-only `POST /contacts/search` shape with `locationId`, `pageLimit`, optional fuzzy `query`, and optional exact `filters.email` / `filters.phone`. This is a read operation even though the upstream endpoint uses POST. `contacts get` uses `GET /contacts/{contact_id}`. Both commands require resolved location context from `--location` or the active profile before they run.

`conversations search` uses `GET /conversations/search` with `locationId`, `status`, `limit`, and optional contact/query/assignment/message-type filters. `conversations get` uses `GET /conversations/{conversation_id}`. `conversations messages` uses `GET /conversations/{conversation_id}/messages` with limit and cursor filters. Conversation message bodies and preview bodies are redacted in normal JSON output.
