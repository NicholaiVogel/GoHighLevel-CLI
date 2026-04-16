# Error Registry

The stable error registry is exposed with:

```bash
ghl errors list --pretty
ghl errors show validation_error --pretty
```

Errors use this envelope:

```json
{
  "ok": false,
  "error": {
    "code": "validation_error",
    "message": "Human-readable message.",
    "exit_code": 2,
    "details": {},
    "hint": null
  },
  "meta": {
    "schema_version": "ghl-cli.v1"
  }
}
```
