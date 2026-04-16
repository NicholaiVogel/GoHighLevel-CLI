# Endpoint Manifest

The bundled manifest lives at `data/endpoints.json`.

The first implemented endpoint records are `locations.get` and `locations.search`, used by PIT validation and read-only location commands.

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
