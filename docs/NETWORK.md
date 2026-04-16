# Network Behavior

The CLI now supports explicit read-only network requests through PIT validation, `raw request` GET, and typed location get/list/search. Mutating raw methods are not implemented yet.

Requests use the selected profile base URL, `Authorization: Bearer <token>`, `Accept: application/json`, `Content-Type: application/json`, `Version: 2021-07-28`, and a browser-compatible user agent. Authorization and token-looking values are redacted from diagnostics and outputs.

Future network behavior is defined in `docs/SPEC.md`.


`locations list` and `locations search` use `GET /locations/search`. The current upstream search filter exposed by the reference client is email, so `locations search <email>` maps the query to that filter.
