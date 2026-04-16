# Network Behavior

The CLI now supports explicit read-only network requests through PIT validation, `raw request` GET, typed location get/list/search, and typed contact search/get. Mutating raw methods are not implemented yet.

Requests use the selected profile base URL, `Authorization: Bearer <token>`, `Accept: application/json`, `Content-Type: application/json`, `Version: 2021-07-28`, and a browser-compatible user agent. Authorization and token-looking values are redacted from diagnostics and outputs.

Future network behavior is defined in `docs/SPEC.md`.


`locations list` and `locations search` use `GET /locations/search`. The current upstream search filter exposed by the reference client is email, so `locations search <email>` maps the query to that filter.

`contacts search` uses the reference client's read-only `POST /contacts/search` shape with `locationId`, `pageLimit`, optional fuzzy `query`, and optional exact `filters.email` / `filters.phone`. This is a read operation even though the upstream endpoint uses POST. `contacts get` uses `GET /contacts/{contact_id}`. Both commands require resolved location context from `--location` or the active profile before they run.
