<h1 align="center">
  <img src="https://ghlcentral.com/wp-content/uploads/2024/01/HighLevel_Logo_Classic_black_transparent_1-1024x203.webp" alt="HighLevel" height="72" align="center" />
  &nbsp;ghl
</h1>

<p align="center">
  <strong>Give agents and scripts a safe command-line handle on GoHighLevel.</strong>
</p>

<p align="center">
  An unofficial, local-first CLI for GoHighLevel CRM and agency operations,
  built for stable JSON output, guarded automation, and headless agent runtimes.
</p>

<p align="center">
  <a href="https://github.com/NicholaiVogel/GoHighLevel-CLI/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/NicholaiVogel/GoHighLevel-CLI/actions/workflows/ci.yml/badge.svg"></a>
  <img alt="Status" src="https://img.shields.io/badge/status-early_MVP-111111">
  <img alt="Rust" src="https://img.shields.io/badge/rust-CLI-b7410e">
  <img alt="Unofficial" src="https://img.shields.io/badge/HighLevel-unofficial-111111">
</p>

> [!NOTE]
> This is an unofficial community project. It is not affiliated with,
> endorsed by, sponsored by, or maintained by HighLevel, GoHighLevel,
> LeadConnector, or any related company. The HighLevel logo is used only to
> identify the service this client talks to.

`ghl` turns GoHighLevel into something agents, shell scripts, CI jobs, and
terminal-native workflows can call safely. It uses existing GHL HTTP APIs,
stores credentials locally, returns stable JSON, and puts a narrow command
surface between automation and your CRM.

The project is early, but the shape is intentional: start with auth, profiles,
read-only smoke tests, endpoint metadata, and redaction. Then add CRM coverage
one guarded slice at a time.

## Lineage and attribution

This CLI is based on the API map and automation blueprint from
[`BusyBee3333/Go-High-Level-MCP-2026-Complete`](https://github.com/BusyBee3333/Go-High-Level-MCP-2026-Complete).
That project made the broad GoHighLevel surface legible: hundreds of tools,
auth patterns, endpoint notes, and working examples across the GHL ecosystem.

Credit for that expanded 2026 blueprint goes to **Jake Shore**
([`@BusyBee3333`](https://github.com/BusyBee3333)), a friend and collaborator.
The upstream MCP project also credits
[`@mastanley13`](https://github.com/mastanley13) for the original foundation.

This repository is the CLI adaptation of that work. The goal is not to erase or
rename the blueprint. The goal is to turn it into a polished local command-line
client with tighter safety boundaries, stable output contracts, tests, docs,
and agent-friendly behavior.

## Why this exists

GoHighLevel already has the CRM, conversations, opportunities, calendars,
forms, automations, payments, reporting, and agency operations people want to
wire into scripts. The hard part is not that the APIs do not exist. The hard
part is giving agents and operators a command surface they can trust.

Raw API access gives an agent too much rope. A hosted MCP server is powerful,
but it is not always the right boundary for local automation, CI, SSH sessions,
or audited workflows.

`ghl` is that boundary.

It gives you:

- **Local control:** credentials stay on the machine running the CLI.
- **Agent-ready output:** JSON by default, command metadata, and stable error
  envelopes.
- **Narrow live testing:** start with read-only validation before touching real
  CRM data.
- **Guarded expansion:** write commands will arrive behind dry-run, policy,
  confirmation, and audit behavior.
- **Reference-backed coverage:** endpoint work is mapped back to the 2026 MCP
  reference, the internal API bible, and `docs/SPEC.md`.
- **Portable invocation:** short `ghl` for daily use, explicit `ghl-cli` for
  scripts and documentation.

## What it can do today

| Area | Current support |
| --- | --- |
| Config | Resolve config/data/cache/audit paths, show redacted config, local doctor |
| Profiles | Create through auth, list, show, set default, set default company, set default location |
| PIT auth | Store local Private Integration Token references, list redacted previews, remove local refs |
| Live validation | Validate PIT with `GET /locations/{location_id}` without printing the body |
| Locations | Get by id, list by company, search with the current upstream email filter |
| Contacts | Summary-only list, search by query/exact email/phone, get one contact by id |
| Conversations | Search by contact/query/status, get one conversation, list messages with bodies redacted |
| Pipelines | List pipelines; get one pipeline by id from the location pipeline list |
| Opportunities | Search by contact/pipeline/stage/status; get one opportunity by id |
| Smoke | Read-only `smoke run` with status/count output and no customer data |
| Raw requests | Guarded read-only `GET` against `services` or `backend` surfaces |
| Safety | Token redaction, owner-only local credential file on Unix, offline blocking |
| Metadata | Command schema, endpoint manifest, error registry, shell completions |
| Docs | Product spec, command reference, network notes, smoke guide, coverage plan |

The current implementation is intentionally small. It can connect to a test
location, prove the auth path works, and perform the first read-only CRM slices.
All writes are still being built.

## Quick start

Build from source:

```bash
cargo build --workspace
```

Use either command name:

```bash
./target/debug/ghl --help
./target/debug/ghl-cli --help
```

Inspect the machine-readable command schema:

```bash
./target/debug/ghl commands schema --pretty
```

## Connect a GoHighLevel location

For the current live smoke path, you need two things:

1. A dedicated test Location ID.
2. A Private Integration Token with the smallest useful read scope, starting
   with location read access.

In GoHighLevel, create a Private Integration from the location settings, copy
the generated token, and keep it out of chat logs and shell history. Prefer a
password manager, Signet secret injection, or an environment variable.

Store the token locally:

```bash
export GHL_PIT="your-private-integration-token"

./target/debug/ghl --profile default auth pit add \
  --token-env GHL_PIT \
  --location <location-id> \
  --company <company-id>
```

Validate it with the safest live request we have:

```bash
./target/debug/ghl --profile default auth pit validate --pretty
```

Then run the read-only smoke runner. It prints statuses and counts, not
customer records:

```bash
./target/debug/ghl --profile default smoke run --pretty
```

If you have known test resource IDs, you can include optional read checks:

```bash
./target/debug/ghl --profile default smoke run \
  --contact-email test@example.com \
  --contact-id <contact-id> \
  --conversation-id <conversation-id> \
  --pipeline-id <pipeline-id> \
  --opportunity-id <opportunity-id> \
  --pretty
```

You can also fetch individual resources through the typed commands:

```bash
./target/debug/ghl --profile default locations get <location-id> --pretty
./target/debug/ghl --profile default locations list --pretty
./target/debug/ghl --profile default locations search user@example.com --pretty
./target/debug/ghl --profile default contacts list --limit 5 --pretty
./target/debug/ghl --profile default contacts search "Sarah" --limit 10 --pretty
./target/debug/ghl --profile default contacts search --email sarah@example.com --pretty
./target/debug/ghl --profile default contacts get <contact-id> --pretty
./target/debug/ghl --profile default conversations search --contact <contact-id> --pretty
./target/debug/ghl --profile default conversations get <conversation-id> --pretty
./target/debug/ghl --profile default conversations messages <conversation-id> --pretty
./target/debug/ghl --profile default pipelines list --pretty
./target/debug/ghl --profile default pipelines get <pipeline-id> --pretty
./target/debug/ghl --profile default opportunities search --contact <contact-id> --pretty
./target/debug/ghl --profile default opportunities get <opportunity-id> --pretty
```

`locations search` currently maps the search value to GHL's upstream email filter.
`contacts search --email` uses the exact email filter from the contact search
payload; the positional query stays fuzzy.

If you only want to preview the request shape without credentials or network
access, use local dry-run:

```bash
./target/debug/ghl locations get <location-id> --dry-run=local
./target/debug/ghl --location <location-id> contacts list --limit 5 --dry-run=local
./target/debug/ghl --location <location-id> contacts search "Sarah" --dry-run=local
./target/debug/ghl --location <location-id> conversations search --dry-run=local
./target/debug/ghl --location <location-id> opportunities search --dry-run=local
./target/debug/ghl --location <location-id> smoke run --dry-run=local --pretty
```

## Current command surface

```bash
ghl commands schema

ghl config path
ghl config show
ghl config doctor

ghl auth pit add --token-stdin --location <location-id>
ghl auth pit add --token-env GHL_PIT --location <location-id> --company <company-id>
ghl auth pit validate
ghl auth pit list-local
ghl auth pit remove-local <credential-ref>
ghl auth status

ghl profiles list
ghl profiles show <name>
ghl profiles set-default <name>
ghl profiles set-default-company <name> <company-id>
ghl profiles set-default-location <name> <location-id>
ghl profiles policy show <name>
ghl profiles policy set <name> [...flags]
ghl profiles policy reset <name> --yes

ghl locations get <location-id>
ghl locations list [--company <company-id>]
ghl locations search <email> [--company <company-id>]

ghl contacts list [--limit <n>]
ghl contacts search [<query>] [--email <email>] [--phone <phone>] [--limit <n>]
ghl contacts get <contact-id>

ghl conversations search [--contact <contact-id>] [--query <query>] [--status all|read|unread|starred|recents] [--limit <n>]
ghl conversations get <conversation-id>
ghl conversations messages <conversation-id> [--limit <n>] [--last-message-id <id>] [--message-type <type>]

ghl pipelines list
ghl pipelines get <pipeline-id>

ghl opportunities search [--contact <contact-id>] [--pipeline <pipeline-id>] [--stage <stage-id>] [--status open|won|lost|abandoned|all] [--limit <n>]
ghl opportunities get <opportunity-id>

ghl smoke run [--limit <n>] [--skip-optional]

ghl raw request --surface services --method get --path /locations/<location-id>
ghl raw request --surface backend --method get --path <path>

ghl errors list
ghl errors show <error-code>
ghl endpoints list
ghl endpoints show <endpoint-key>
ghl endpoints coverage

ghl completions bash|zsh|fish|powershell
ghl man
```

## Built for agents

`ghl` treats agent use as a product requirement, not a side effect.

```bash
ghl commands schema --pretty
ghl endpoints coverage --pretty
ghl errors list --pretty
```

Agents can discover commands, inspect endpoint coverage, understand which
commands require network access, and rely on consistent success and error
envelopes.

Normal success output looks like this:

```json
{
  "ok": true,
  "data": {},
  "meta": {
    "schema_version": "ghl-cli.v1"
  }
}
```

Errors keep the same shape:

```json
{
  "ok": false,
  "error": {
    "code": "validation_error",
    "message": "...",
    "exit_code": 2,
    "details": {},
    "hint": "..."
  },
  "meta": {
    "schema_version": "ghl-cli.v1"
  }
}
```

## Safety model

`ghl` is designed for boring, inspectable automation.

- Normal command output never prints full credential values.
- PIT credentials are stored separately from profile config.
- The local fallback credential file uses owner-only permissions on Unix.
- `--token-stdin` and `--token-env` are preferred over passing tokens directly.
- Raw requests are currently `GET` only.
- `--offline` blocks real network commands unless `--dry-run=local` is set.
- Response redaction covers authorization headers, cookies, tokens, API keys,
  secrets, passwords, OTPs, message bodies, and token-like values.
- Future write commands must pass through dry-run, profile policy, audit, and
  explicit confirmation gates before real execution.

This matters because an agent should be useful around CRM data without being
handed a raw token and a loaded crossbow.

## Project status

`ghl` is pre-release and under active development.

Implemented now:

- Rust workspace with `ghl` library crate and `ghl-cli` binary crate.
- Short alias binary: `ghl`.
- Config path resolution.
- Profile persistence.
- Local PIT credential storage.
- Read-only PIT validation.
- Guarded raw GET.
- Typed `locations get`, `locations list`, and `locations search`.
- Typed `contacts list`, `contacts search`, and `contacts get`.
- Typed `conversations search`, `conversations get`, and `conversations messages`.
- Typed `pipelines list`, `pipelines get`, `opportunities search`, and `opportunities get`.
- Read-only `smoke run` for safe real-account validation.
- Endpoint manifest seed.
- Command metadata.
- Stable error registry.
- Shell completions and manual output.
- CI for fmt, clippy, tests, and build.

Current focus:

- first real GHL account validation with a dedicated test location
- stronger pagination and response normalization for CRM commands
- rate limiting, retries, and read-only cache
- OS keyring credential backend
- Signet secret references

Future feature planning lives in [`docs/ROADMAP.md`](docs/ROADMAP.md) and the
full product contract lives in [`docs/SPEC.md`](docs/SPEC.md).

## Documentation

- [`docs/SPEC.md`](docs/SPEC.md), full product spec and implementation plan
- [`docs/COMMANDS.md`](docs/COMMANDS.md), implemented command surface
- [`docs/CONFIG.md`](docs/CONFIG.md), config and credential behavior
- [`docs/NETWORK.md`](docs/NETWORK.md), request surfaces, headers, and network rules
- [`docs/SMOKE.md`](docs/SMOKE.md), safe local and real-account validation
- [`docs/ENDPOINTS.md`](docs/ENDPOINTS.md), endpoint manifest shape
- [`docs/API-COVERAGE.md`](docs/API-COVERAGE.md), coverage status
- [`docs/FEATURE-PARITY.md`](docs/FEATURE-PARITY.md), reference parity plan
- [`docs/ERRORS.md`](docs/ERRORS.md), stable error-code registry
- [`docs/SECURITY.md`](docs/SECURITY.md), security commitments
- [`docs/INSTALL.md`](docs/INSTALL.md), install notes

## Development

Run the standard validation suite:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

Run a network-free smoke preview:

```bash
./target/debug/ghl --location loc_test smoke run --dry-run=local --pretty
```

## License

Apache-2.0. See [`LICENSE`](LICENSE).
