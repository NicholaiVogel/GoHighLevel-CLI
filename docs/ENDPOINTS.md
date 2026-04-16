# Endpoint Manifest

The bundled manifest lives at `data/endpoints.json`.

The first implemented endpoint records are:

- `locations.get`, used by PIT validation and read-only location get.
- `locations.search`, used by read-only location list/search.
- `contacts.search`, used by read-only contact search.
- `contacts.get`, used by read-only contact get.
- `conversations.search`, used by read-only conversation search.
- `conversations.get`, used by read-only conversation get.
- `conversations.messages`, used by read-only message listing with body redaction.

Manifest fields:

- `endpoint_key`
- `surface`
- `method`
- `path_template`
- `auth_classes`
- `source_refs`
- `risk`
- `status`
- `phase`
- `command_keys`
- `response_schema`

Commands:

```bash
ghl endpoints list
ghl endpoints coverage
ghl endpoints show <endpoint-key>
```
