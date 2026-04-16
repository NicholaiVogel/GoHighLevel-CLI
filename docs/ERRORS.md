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


`ambiguous_context` uses exit code 2 when a command needs company or location context and the CLI cannot resolve it from flags, environment, or profile defaults.
