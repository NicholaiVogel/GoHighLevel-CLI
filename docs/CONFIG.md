# Configuration

The CLI now implements path resolution, profile persistence, local PIT credential storage, read-only PIT validation, raw GET, and typed location get/list/search.

Resolution precedence:

1. `--config-dir <path>`
2. `GHL_CLI_CONFIG_DIR`
3. Platform config directory, usually `~/.config/ghl-cli` on Linux

Inspect resolved paths:

```bash
ghl config path --pretty
```

Check local path status without creating files:

```bash
ghl config doctor --pretty
```

## Profile files

Profiles are stored in `profiles.json` under the resolved config directory. Local fallback credentials are stored separately in `credentials.json` with owner-only permissions on Unix platforms. Normal CLI output never prints full credential values.


Profile context currently stores optional `company_id` and `location_id`. Commands resolve context from explicit CLI/env overrides first, then profile defaults. Missing required company or location context returns `ambiguous_context`.
