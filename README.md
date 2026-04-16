# Go High Level CLI

`ghl-cli` is an unofficial local command line client for Go High Level, built for humans, shell scripts, and AI agents that need stable JSON access to CRM and agency operations.

Status: Phase 1 auth/profile and HTTP spine, with the first Phase 2 read-only location command. The current implementation can persist profiles, store local PIT credential references, validate a PIT with an explicit read-only request, run guarded raw GET requests, and fetch one location by id.

## Install from source

```bash
cargo build --workspace
cargo run -p ghl-cli -- commands schema --pretty
```

The workspace builds two binaries:

- `ghl-cli`
- `ghl`

## Current commands

```bash
ghl commands schema
ghl config path
ghl config show
ghl config doctor
ghl auth pit add --token-stdin --location <location-id>
ghl auth pit validate
ghl auth pit list-local
ghl auth pit remove-local <credential-ref>
ghl auth status
ghl profiles list
ghl profiles show <name>
ghl profiles set-default <name>
ghl profiles set-default-location <name> <location-id>
ghl profiles policy show <name>
ghl profiles policy set <name> [...flags]
ghl profiles policy reset <name> --yes
ghl errors list
ghl errors show <error-code>
ghl endpoints list
ghl endpoints show <endpoint-key>
ghl endpoints coverage
ghl raw request --surface services --method get --path /locations/<location-id> --dry-run=local
ghl locations get <location-id>
ghl completions bash|zsh|fish|powershell
ghl man
```

## Safety note

This project is unofficial and is not affiliated with Go High Level. Network behavior is intentionally narrow: read-only PIT validation, raw GET, and typed `locations get` only. Future slices will add broader HTTP behavior and guarded CRM commands from `docs/SPEC.md`.

## Validation

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```
