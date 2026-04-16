# Command Reference

Status: Phase 1 auth/profile and HTTP spine, plus read-only location and contact commands.

Machine-readable command metadata is available with:

```bash
ghl commands schema --pretty
```

Implemented commands:

- `ghl commands schema`
- `ghl config path`
- `ghl config show`
- `ghl config doctor`
- `ghl auth pit add --token-stdin --location <location-id> [--company <company-id>]`
- `ghl auth pit validate`
- `ghl auth pit list-local`
- `ghl auth pit remove-local <credential-ref>`
- `ghl auth status`
- `ghl profiles list`
- `ghl profiles show <name>`
- `ghl profiles set-default <name>`
- `ghl profiles set-default-company <name> <company-id>`
- `ghl profiles set-default-location <name> <location-id>`
- `ghl profiles policy show <name>`
- `ghl profiles policy set <name> [...flags]`
- `ghl profiles policy reset <name> --yes`
- `ghl errors list`
- `ghl errors show <error-code>`
- `ghl endpoints list`
- `ghl endpoints show <endpoint-key>`
- `ghl endpoints coverage`
- `ghl raw request --surface services|backend --method get --path <path> [--include-body]`
- `ghl locations get <location-id>`
- `ghl locations list [--company <company-id>] [--skip <n>] [--limit <n>] [--order asc|desc]`
- `ghl locations search <email> [--company <company-id>] [--skip <n>] [--limit <n>] [--order asc|desc]`
- `ghl contacts search [<query>] [--email <email>] [--phone <phone>] [--limit <n>] [--start-after-id <id>] [--start-after <cursor>]`
- `ghl contacts get <contact-id>`
- `ghl completions bash|zsh|fish|powershell`
- `ghl man`

Network support is deliberately narrow: PIT validation, raw GET, read-only location get/list/search, and read-only contact search/get only. Use `--dry-run=local` to preview network commands without credentials or network access. Contact commands require resolved location context from `--location` or the active profile. PIT tokens are stored locally and redacted from normal output.
