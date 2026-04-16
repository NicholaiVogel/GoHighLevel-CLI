# Security

The current implementation stores PIT credentials in the local fallback credential file and supports explicit read-only network calls for PIT validation, raw GET, and typed location get/list/search.

Security commitments for later phases:

- Prefer OS keyring for stored credentials when that backend lands.
- Use owner-only permissions for the local fallback credential file on Unix.
- Never store account passwords after login bootstrap.
- Redact tokens, customer data, links, and message bodies in diagnostics.
- Refuse raw mutating methods until policy gates and audit logging are implemented.
- Apply policy gates before network mutation.
- Keep destructive and sensitive commands behind explicit confirmation.

See `docs/SPEC.md` for the full threat model.
