# Go High Level CLI Product Spec

Status: Draft product specification  
Primary binary: `ghl-cli`  
Short alias: `ghl`  
Target milestone: Agent-safe MVP  
Primary references: `references/cli`, `references/Nextcloud-CLI`, `references/ghl-internal-api-bible`, `references/Go-High-Level-MCP-2026-Complete`

## How to Use This Spec

This file is the canonical product contract for the first implementation pass. It
is intentionally more specific than a normal planning document because the work is
likely to be delegated across agents and sessions. When implementation reveals a
Go High Level API detail that differs from this document, update this spec in the
same change that updates the code.

The spec should be read in four layers:

1. Product contract: sections 1 through 12 define what the CLI is, who it serves,
   how it is named, how it authenticates, and how it talks to GHL.
2. Operational safety contract: sections 13 through 47 define coverage,
   pagination, bulk execution, idempotency, audit, redaction, scopes, drift,
   fixtures, Signet integration, capabilities, compliance, dry-run fidelity,
   schemas, undo, jobs, locking, smoke cleanup, config lifecycle, offline mode,
   confirmation UX, network behavior, threat model, disclaimers, endpoint
   manifest, context resolution, query shaping, support bundles, credential
   hardening, token lifecycle, error registry, normalization, exports, and
   retention.
3. Feature contracts: sections 48 through 62 define the public command surface,
   output shapes, safety rules, and completion gates.
4. Delivery contract: sections 63 through 74 define tests, architecture,
   compatibility, distribution, documentation, release readiness, supply-chain
   security, completions, and terms review.

The first implementation should move in thin vertical slices. Auth, profiles,
configuration, HTTP client, JSON output, errors, contacts, opportunities,
conversations, and calendar reads come before broad endpoint coverage. That gives
agents a working CRM spine before we bolt on every last GHL subsystem.

## Implementation Status Snapshot

Last updated: 2026-04-16 during the conversations read slice.

Implemented so far:

- Rust workspace with `ghl` library crate and `ghl-cli` binary crate.
- Primary binary `ghl-cli` and short alias `ghl`.
- Stable JSON success and error envelopes.
- Config path resolution with `GHL_CLI_CONFIG_DIR` and `--config-dir`.
- Command metadata registry, error registry, endpoint manifest scaffold, shell completions, docs skeleton, and CI.
- Profile schema and persistence in `profiles.json`.
- Local fallback credential store in `credentials.json` with owner-only permissions on Unix.
- Local PIT commands: `auth pit add`, `auth pit list-local`, `auth pit remove-local`, `auth pit validate`, and `auth status`.
- Profile commands: list, show, set default, set default company, set default location, and policy show/set/reset.
- HTTP client spine for `services` and `backend` surfaces with PIT auth headers, redacted response handling, explicit `raw request` GET, and read-only PIT validation through `GET /locations/{location_id}`.
- First typed read-only CRM commands: `locations get <location-id>`, `locations list`, `locations search <email>`, `contacts search [<query>] [--email <email>] [--phone <phone>]`, `contacts get <contact-id>`, `conversations search`, `conversations get`, and `conversations messages`.
- Contact and conversation read commands require resolved location context from `--location` or the active profile and support local dry-run previews.
- Conversation message bodies and preview bodies are redacted from normal response output.

Remaining initial implementation priorities:

- Credential backend abstraction with OS keyring preferred and owner-only local
  fallback for headless environments.
- Private Integration Token auth for public API surfaces.
- Session JWT auth with refresh-token rotation for internal API surfaces.
- Optional Firebase auth for A2P, Trust Center, and domain-management surfaces.
- Per-location rate limiting, retries, and read-only caching.
- Agent-safe write policy with dry-run, confirmation flags, and destructive
  guards.
- Remaining initial command groups: opportunities, pipelines, calendars,
  workflows read, smoke run, broader contact subcommands, and guarded messaging.

## 1. Product Summary

`ghl-cli` is a local command line client for Go High Level, built for humans,
shell scripts, and AI agents that need structured access to GHL CRM and agency
operations without writing custom API glue for each workflow.

The CLI authenticates against GHL using the safest available credential for the
requested surface, talks to existing GHL HTTP APIs, and returns stable JSON that
agents can consume without custom adapters. It supports both documented public
API endpoints and selected internal API endpoints captured in the local reference
bible.

The product has three surfaces:

1. Curated commands for high-value CRM and agency workflows.
2. A safe raw HTTP escape hatch for long-tail endpoints.
3. Agent skills and command metadata so an AI system can discover capabilities,
   understand safety rules, and call the CLI predictably.

The implementation follows the spirit of `references/cli` and
`references/Nextcloud-CLI`:

- Rust workspace with reusable library crate and thin binary crate.
- Structured JSON output by default.
- Useful human output formats where safe.
- Explicit auth commands.
- Secure credential storage.
- Dry-run support for write operations.
- Stable error codes and envelopes.
- Testable command contracts.
- Release binaries, npm wrapper, curl installer, and agent skills.

GHL does not expose one complete discovery document for every surface. This
product is therefore curated first, with generated metadata from command
definitions and an explicit raw request command for advanced users.

## 2. Goals

1. Provide an agent-safe CLI for Go High Level CRM and agency operations.
2. Return stable JSON by default for every normal command.
3. Support multiple auth methods because GHL endpoints require different token
   classes.
4. Store tokens securely and rotate refresh tokens correctly.
5. Prefer documented public APIs when they satisfy the workflow.
6. Use internal APIs only when the reference bible identifies them as necessary.
7. Apply per-location rate limiting because GHL rate limits are location-scoped.
8. Make destructive actions difficult to trigger accidentally.
9. Provide dry-run behavior for writes that can change customer data, billing,
   messaging, workflows, phone numbers, or compliance state.
10. Support scripts, CI jobs, and headless agent runtimes.
11. Preserve enough raw upstream detail for debugging without exposing secrets.
12. Ship with docs, examples, smoke tests, and agent skills.

## 3. Non-goals

The MVP excludes:

- A hosted MCP service.
- An MCP server, MCP bridge, or `ghl mcp serve` command. MCP is out of scope.
- A GHL web UI replacement.
- Browser scraping as a normal runtime dependency.
- Unmonitored autonomous campaign execution.
- Bulk destructive cleanup tools without explicit safety policy.
- Full page-builder visual editing.
- Full workflow visual layout rendering.
- A complete clone of all 562 MCP tools in the first milestone.
- Unstable experimental endpoints without tests or reference notes.
- Multi-tenant SaaS hosting concerns.
- Storing user account passwords after login bootstrap.
- Sending SMS, email, invoices, payment links, or review requests without an
  explicit command and policy allowance.

## 4. Source References

Local product and architecture references:

- `references/cli`: Google Workspace CLI style and architecture reference.
- `references/Nextcloud-CLI`: Rust CLI, JSON contracts, auth/profile patterns,
  release packaging, npm wrapper, curl installer, and agent-skill structure.

Local Go High Level references:

- `references/ghl-internal-api-bible/README.md`: API domain map, auth methods,
  headers, and section index.
- `references/ghl-internal-api-bible/01-auth/README.md`: token classes, refresh
  behavior, PIT usage, Firebase auth, and Cloudflare User-Agent note.
- `references/ghl-internal-api-bible/01-auth/headless-login-3step.md`: 2FA login
  flow with email OTP, stable `deviceId`, and the required step-two `token`.
- `references/ghl-internal-api-bible/25-private-integration-tokens/README.md`:
  PIT CRUD endpoints and token-return behavior.
- `references/ghl-internal-api-bible/14-crm-contacts/endpoints.md`: contacts,
  conversations, opportunities, workflows, and rate-limit patterns.
- `references/ghl-internal-api-bible/15-pipelines/endpoints.md`: pipeline and
  stage CRUD rules.
- `references/ghl-internal-api-bible/02-workflows/endpoints.md`: hidden workflow
  builder API, action schema, trigger schema, and workflow update rules.
- `references/ghl-internal-api-bible/*/endpoints.md`: feature-specific endpoint
  references for funnels, forms, surveys, media, reporting, reputation,
  snapshots, domains, templates, custom fields, custom values, smart lists,
  social planner, A2P, and Agent Studio.
- `references/Go-High-Level-MCP-2026-Complete/src/clients/ghl-api-client.ts`:
  broad public API client patterns.
- `references/Go-High-Level-MCP-2026-Complete/src/enhanced-ghl-client.ts`: retry,
  cache, keep-alive, and rate-limit patterns.
- `references/Go-High-Level-MCP-2026-Complete/src/tool-registry.ts`: module
  registry, tool categorization, and read/write/destructive inference.
- `references/Go-High-Level-MCP-2026-Complete/src/tools/*.ts`: 562 working MCP
  tool definitions across 45 GHL categories. These are API coverage references,
  not a requirement to expose MCP.

### 4.1 Reference CLI lessons to preserve

When product or implementation details are uncertain, inspect `references/cli`
and `references/Nextcloud-CLI` before inventing a new pattern. These projects
already made several useful decisions:

- Workspace split between reusable library crate and binary crate.
- JSON output as the agent-facing surface.
- `commands schema` for command metadata.
- Stable error envelopes.
- Explicit profile and credential commands.
- Release artifacts per target with checksum files.
- npm package that downloads GitHub Release binaries rather than bundling native
  binaries into the package.
- Installer tests using local release fixtures.
- Smoke tests that avoid printing private data.
- Agent skills that document installation, setup, and safe command usage.

If this project diverges from those references, the implementation PR should say
why.

## 5. User Stories and Use Cases

### 5.1 Agent CRM lookup

As an AI agent, I can search contacts, inspect a contact timeline, read recent
conversations, and summarize what has happened with a lead before suggesting a
next action.

Example:

```bash
ghl contacts search "sarah@example.com"
ghl contacts timeline <contact-id> --limit 25
ghl conversations messages --conversation <conversation-id> --limit 50
```

### 5.2 Human CRM scripting

As an operator, I can create or update contacts, add tags, create opportunities,
and move deals between pipeline stages from shell scripts.

Example:

```bash
ghl contacts upsert --email sarah@example.com --first-name Sarah --tag webinar

ghl opportunities create \
  --contact <contact-id> \
  --pipeline <pipeline-id> \
  --stage <stage-id> \
  --name "Sarah - Discovery" \
  --value 15000 \
  --dry-run
```

### 5.3 Appointment operations

As an agent, I can check free slots, create appointments, update appointment
notes, and avoid double-booking.

Example:

```bash
ghl calendars free-slots --calendar <calendar-id> --date 2026-04-14
ghl appointments create --calendar <calendar-id> --contact <contact-id> --starts-at 2026-04-14T14:00:00-06:00 --dry-run
```

### 5.4 Messaging with guardrails

As an agent, I can draft and send SMS or email through GHL only when the active
profile policy allows messaging. The command returns a dry-run preview unless
real sending is explicitly allowed.

Example:

```bash
ghl messages send-sms --contact <contact-id> --body "Following up from our call." --dry-run
```

### 5.5 Workflow inspection and controlled edits

As a developer, I can list workflows, fetch the full hidden workflow definition,
validate a proposed change, and publish only with explicit confirmation.

Example:

```bash
ghl workflows get <workflow-id> --full
ghl workflows validate --from-file workflow.json
ghl workflows update <workflow-id> --from-file workflow.json --dry-run
```

### 5.6 Agency and multi-location operations

As an agency operator, I can list locations, switch default location, manage
private integration tokens, run reports across selected locations, and perform
bulk reads without crossing rate limits.

Example:

```bash
ghl locations list --company <company-id>
ghl profiles set-default-location default <location-id>
ghl reports pipeline --location <location-id> --from 2026-04-01 --to 2026-04-30
```

### 5.7 Raw endpoint debugging

As a developer, I can call a referenced endpoint directly while preserving auth,
headers, redaction, retries, and JSON error envelopes.

Example:

```bash
ghl raw request --surface services --method GET --path '/contacts/?locationId=<location-id>&limit=10'
```

## 6. Product Principles

### 6.1 JSON-first

Every command returns machine-readable JSON by default. Human table output is
allowed through `--format table` only when it is safe and unambiguous.

### 6.2 Agent-safe

The CLI assumes agents will use it. Commands that mutate state must support
`--dry-run` where possible. Destructive actions require `--yes`. Sensitive
classes of actions require profile policy allowances.

### 6.3 Credentials stay out of sight

Tokens, refresh tokens, PITs, OTPs, account passwords, private integration active
tokens, message bodies, and payment links must not be logged. Commands may report
that a secret exists, its type, and the credential backend used.

### 6.4 Curated commands first

High-value workflows get first-class commands with normalized output. Long-tail
API calls go through `raw request` until they earn a stable command contract.

### 6.5 Auth is explicit

Each profile records which auth methods are available. Each command declares the
minimum auth class it needs. When the active profile cannot satisfy a command, the
CLI returns an auth error with a specific setup hint.

### 6.6 Policy is part of the profile

Every profile has a local policy object. Policy gates messaging, public links,
payment actions, workflow publishing, phone number purchase/release,
destructive actions, and default dry-run behavior.

### 6.7 Reference before inventing

Internal endpoint behavior must be grounded in local references. If a command
uses an undocumented endpoint, the feature spec must name the reference file and
include a test fixture.

## 7. CLI Surface

### 7.1 Global syntax

```bash
ghl [global-options] <group> <command> [command-options]
ghl-cli [global-options] <group> <command> [command-options]
```

### 7.2 Global flags

| Flag | Meaning |
| --- | --- |
| `--profile <name>` | Use a named profile instead of the default profile. |
| `--location <id>` | Override the active location for this command. |
| `--company <id>` | Override the active company for this command. |
| `--config-dir <path>` | Override config directory. |
| `--format json|table|ndjson` | Output format. Default: `json`. |
| `--pretty` | Pretty-print JSON. |
| `--quiet` | Suppress progress diagnostics on stderr. |
| `--verbose` | Print redacted request diagnostics to stderr. |
| `--dry-run[=local|validated]` | Preview a write without changing remote state where supported. |
| `--yes` | Confirm a destructive or sensitive action. |
| `--no-cache` | Bypass read cache for this command. |
| `--timeout <duration>` | Override request timeout. |
| `--offline` | Refuse network access and run only local commands. |
| `--lock-timeout <duration>` | Override local lock wait timeout. |

### 7.3 Environment variables

| Variable | Meaning |
| --- | --- |
| `GHL_CLI_CONFIG_DIR` | Config directory override. |
| `GHL_CLI_PROFILE` | Default profile override. |
| `GHL_CLI_LOCATION_ID` | Default location override. |
| `GHL_CLI_COMPANY_ID` | Default company override. |
| `GHL_CLI_FORMAT` | Default output format. |
| `GHL_CLI_NO_CACHE` | Disable read cache when set to `1`. |
| `GHL_CLI_KEYRING_BACKEND` | `auto`, `os`, or `file`. |
| `GHL_CLI_PIT_TOKEN` | Ephemeral PIT token. Highest credential precedence for PIT commands. |
| `GHL_CLI_SESSION_JWT` | Ephemeral session JWT for internal commands. |
| `GHL_CLI_REFRESH_JWT` | Ephemeral refresh token for session refresh. |
| `GHL_CLI_FIREBASE_ID_TOKEN` | Ephemeral Firebase ID token for Firebase surfaces. |
| `GHL_CLI_USER_AGENT` | Override the browser User-Agent used for PIT calls. |
| `GHL_CLI_TIMEOUT` | Default request timeout. |
| `GHL_CLI_RETRY_MAX` | Max retries for retryable reads. |

### 7.4 Credential precedence

| Priority | Source | Use case |
| --- | --- | --- |
| 1 | Explicit command flag or stdin secret | One-off scripting. |
| 2 | Environment variable | CI and ephemeral agents. |
| 3 | Profile credential reference | Normal local use. |
| 4 | Imported credential file | Migration or headless bootstrap. |

The CLI must never store account passwords. `auth login` may accept a password
through stdin or prompt input to complete bootstrap, then stores only tokens.

### 7.5 Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Success. |
| `1` | General error. |
| `2` | CLI validation error. |
| `3` | Authentication error. |
| `4` | Authorization or permission error. |
| `5` | Network, DNS, TLS, or Cloudflare error. |
| `6` | GHL returned a structured API error. |
| `7` | Required auth class unavailable for this command. |
| `8` | Parse error from malformed JSON, HTML, CSV, or file input. |
| `9` | Local file I/O error. |
| `10` | Timeout. |
| `11` | Rate limit exceeded and wait was not permitted. |
| `12` | Profile policy denied the requested action. |
| `13` | Capability, feature, or plan unavailable. |
| `14` | Schema validation failed. |
| `15` | Confirmation required. |
| `16` | Local resource lock unavailable. |
| `17` | Offline mode blocked a network command. |

### 7.6 Planned command groups

The first public command surface should include:

```bash
ghl commands schema

ghl config path
ghl config show
ghl config doctor
ghl config export
ghl config import <path>
ghl config migrate

ghl auth pit add
ghl auth pit list-local
ghl auth login
ghl auth refresh
ghl auth firebase exchange
ghl auth status
ghl auth logout
ghl auth export

ghl profiles list
ghl profiles show <name>
ghl profiles set-default <name>
ghl profiles set-default-company <name> <company-id>
ghl profiles policy show <name>
ghl profiles policy set <name> [...flags]
ghl profiles policy reset <name> --yes
ghl profiles set-default-location <name> <location-id>

ghl locations list
ghl locations get <location-id>
ghl locations search <email>

ghl contacts search [<query>] [--email <email>] [--phone <phone>] [--limit <n>]
ghl contacts get <contact-id>
ghl contacts create [...fields]
ghl contacts update <contact-id> [...fields]
ghl contacts upsert [...fields]
ghl contacts delete <contact-id> --yes
ghl contacts tags add <contact-id> <tag>...
ghl contacts tags remove <contact-id> <tag>...
ghl contacts tasks list <contact-id>
ghl contacts tasks create <contact-id> [...fields]
ghl contacts notes list <contact-id>
ghl contacts notes create <contact-id> [...fields]
ghl contacts timeline <contact-id>
ghl contacts export --out <path>

ghl conversations search [--contact <id>] [--query <query>] [--status all|read|unread|starred|recents] [--limit <n>]
ghl conversations get <conversation-id>
ghl conversations messages <conversation-id> [--limit <n>] [--last-message-id <id>] [--message-type <type>]
ghl messages send-sms --contact <id>|--conversation <id> --body <text> [--dry-run] [--yes]
ghl messages send-email --contact <id>|--conversation <id> --subject <text> --body <text> [--dry-run] [--yes]

ghl opportunities search [...filters]
ghl opportunities get <opportunity-id>
ghl opportunities create [...fields] [--dry-run]
ghl opportunities update <opportunity-id> [...fields] [--dry-run]
ghl opportunities move <opportunity-id> --stage <stage-id> [--dry-run]
ghl opportunities delete <opportunity-id> --yes
ghl opportunities export --out <path>

ghl pipelines list
ghl pipelines get <pipeline-id>
ghl pipelines create --name <name> --stages <csv|json> [--dry-run]
ghl pipelines update <pipeline-id> [...fields] [--dry-run]
ghl pipelines delete <pipeline-id> --yes

ghl calendars list
ghl calendars groups list
ghl calendars events [...filters]
ghl calendars free-slots --calendar <id> --date <date>
ghl appointments create [...fields] [--dry-run]
ghl appointments update <appointment-id> [...fields] [--dry-run]
ghl appointments delete <appointment-id> --yes

ghl workflows list
ghl workflows get <workflow-id> [--full]
ghl workflows status <workflow-id> --status active|inactive [--dry-run]
ghl workflows trigger <workflow-id> --contact <contact-id> [--dry-run] [--yes]

ghl raw request --surface services|backend|firebase --method <method> --path <path> [--body <json|@file>] [--auth <class>]

ghl auth scopes
ghl auth scopes check <scope>...
ghl commands requirements <command-key>

ghl endpoints list
ghl endpoints show <endpoint-key>
ghl endpoints coverage

ghl context show
ghl context resolve --command <command-key>

ghl capabilities
ghl capabilities location <location-id>
ghl capabilities command <command-key>

ghl schema list
ghl schema show <schema-key>
ghl schema validate <schema-key> --file <path>
ghl schema example <schema-key>

ghl undo plan <audit-entry-id>
ghl undo apply <audit-entry-id> --yes

ghl jobs list
ghl jobs get <job-id>
ghl jobs wait <job-id>

ghl errors list
ghl errors show <error-code>

ghl maintenance status
ghl maintenance prune [--dry-run] --yes

ghl audit list
ghl audit show <entry-id>
ghl audit export

ghl doctor api
ghl doctor endpoint <endpoint-key>
ghl doctor bundle --out <path> --redact

ghl dev capture-fixture <command-key> --redact --out <path>
ghl dev scan-fixtures <path>

ghl signet doctor

ghl smoke run [--skip-writes] [--include-internal]
ghl smoke seed [--dry-run] [--yes]
ghl smoke cleanup --before <datetime> [--dry-run] --yes

ghl completions bash|zsh|fish|powershell
ghl man

ghl update check
```

## 8. Command Naming and Namespace Contract

Command names must separate local CLI state from remote GHL resources. GHL uses
similar words for different layers, and a loose namespace will make the CLI
unsafe for agents.

### 8.1 Canonical namespace rules

| Namespace | Owns | Does not own |
| --- | --- | --- |
| `auth` | Local credentials, auth status, refresh, local stored PIT references, scope checks | Remote Private Integration records |
| `profiles` | Local profile defaults and policy | Remote locations or users |
| `integrations` | Remote GHL integrations, Private Integration records, OAuth apps, marketplace installs | Local credential storage |
| `locations` | Remote GHL locations and location metadata | Local profile selection |
| `contacts` | Contacts, contact tags, contact tasks, contact notes, contact timeline | Conversation-level message sends |
| `conversations` | Threads, conversation metadata, message history | Outbound send commands |
| `messages` | Sending, scheduling, canceling, and inspecting individual messages | Contact notes or timeline synthesis |
| `opportunities` | Deals/opportunities | Pipeline schema management |
| `pipelines` | Pipeline and stage schema | Individual opportunities |
| `calendars` | Calendar definitions, groups, availability, resources | Booked appointment lifecycle |
| `appointments` | Appointment create/update/delete and appointment notes | Calendar schema |
| `workflows` | Workflow definitions, execution, status, publish, trigger | Trigger links and redirect links |
| `links` | Trigger links and trackable redirect links | Public snapshot links |
| `phone` | Phone numbers, recordings, voicemail, forwarding, BYOC | Messaging send commands |
| `dev` | Fixture capture and development-only diagnostics | Normal user workflows |
| `doctor` | API drift and environment diagnostics | Mutating repair operations unless explicitly named |
| `audit` | Local action journal reads/exports | Remote GHL audit logs |

### 8.2 PIT naming decision

There are two PIT concepts:

1. A local PIT credential stored for a CLI profile.
2. A remote GHL Private Integration record that can create or revoke PIT tokens.

These must never share the same command namespace.

Local credentials:

```bash
ghl auth pit add
ghl auth pit list-local
ghl auth pit remove-local <credential-id>
```

Remote GHL Private Integration records:

```bash
ghl integrations pit list --scope company|location
ghl integrations pit create --name <name> --scopes <csv|@file> [--dry-run] [--yes]
ghl integrations pit delete <integration-id> [--dry-run] --yes
```

A top-level `ghl pit ...` namespace is forbidden. My mother did not raise a fool,
and if she did, it was one of my brothers.

### 8.3 Alias rules

- Aliases may be added only for ergonomics, not ambiguity.
- `ghl` is the short binary alias for `ghl-cli`.
- Deprecated command aliases must print a warning on stderr in human mode and a
  `deprecated_alias` metadata field in JSON mode.
- Once public, command removals require at least one minor release of alias
  support unless the command is unsafe.

### 8.4 MCP out of scope

The MCP server reference is source material for endpoint coverage and schema
ideas. This CLI must not add an MCP server, MCP bridge, or `ghl mcp serve`
command in the MVP or near-term roadmap. MCP serving is out of scope because it
adds token overhead and duplicates the CLI's intended agent surface.


## 9. Output Contract

### 9.1 Success shape

Each command may return a command-specific object, but the top-level shape must
be stable per command.

List commands should generally return:

```json
{
  "items": [],
  "count": 0,
  "next_cursor": null
}
```

Domain commands may use clearer collection keys:

```json
{
  "contacts": [],
  "count": 0,
  "next_cursor": null
}
```

Write commands should include normalized resource identity and execution status:

```json
{
  "ok": true,
  "dry_run": false,
  "action": "contacts.update",
  "contact": {
    "id": "contact-id",
    "location_id": "location-id"
  }
}
```

Dry-run commands should include the intended request without secrets:

```json
{
  "ok": true,
  "dry_run": true,
  "action": "messages.send_sms",
  "would_send": {
    "type": "SMS",
    "contact_id": "contact-id",
    "body_redacted": true
  }
}
```

### 9.2 Error envelope

Errors must be printed to stdout as JSON when `--format json` is active or when
stdout is expected to be machine-readable. Human diagnostics may be printed to
stderr only when they do not contain secrets.

```json
{
  "error": {
    "code": "auth_class_unavailable",
    "message": "Profile 'default' does not have session auth for workflow updates.",
    "hint": "Run `ghl auth login --profile default` or use a PIT-compatible command.",
    "status": 401,
    "source": "auth"
  }
}
```

Required error fields:

- `code`: stable snake_case error code.
- `message`: concise human-readable message.

Optional error fields:

- `hint`: remediation step.
- `status`: HTTP status or upstream status code.
- `source`: `cli`, `config`, `auth`, `services`, `backend`, `firebase`,
  `filesystem`, `policy`, or a feature group such as `contacts`.
- `request_id`: upstream request or trace id when available.
- `details`: structured extra context without secrets.

### 9.3 Date and time format

All CLI-produced timestamps must be RFC 3339 strings. If an upstream endpoint
returns Unix timestamps or date-only values, preserve the raw value only when it
is useful and add a normalized field.

### 9.4 Null and empty values

- Empty collections must be `[]`.
- Unknown optional fields may be `null`.
- Missing upstream fields should be normalized to `null` where the field is part
  of the command contract.

### 9.5 Stdout and stderr

- JSON results go to stdout.
- Progress, retry notices, and redacted request diagnostics go to stderr.
- Secrets never go to stdout or stderr unless the user explicitly requests an
  unmasked export command.
- File downloads write bytes to the requested local path.

## 10. Auth Feature Spec

### 10.1 Auth classes

The CLI recognizes these auth classes:

| Auth class | Source | Best for | Credential stored |
| --- | --- | --- | --- |
| `pit` | Private Integration Token | Public API subset, most CRM reads/writes | PIT token |
| `session` | GHL login and refresh | Internal APIs, workflows, pipelines, forms, templates, media | rotating refresh token plus current JWT |
| `firebase` | Firebase token exchange | A2P, Trust Center, Firestore-backed surfaces | Firebase refresh token |
| `cookie` | Browser extracted `m_a` cookie | Legacy fallback only | short-lived cookie token |
| `oauth` | OAuth location token | Future public app integration | refresh/access token pair |

Command implementations must declare their required auth class and acceptable
fallback classes.

### 10.2 `auth pit add`

Adds a PIT token to a profile.

```bash
ghl auth pit add --profile default --token-stdin --location <location-id>
```

Requirements:

- Accept token through stdin, prompt, environment variable, or `--token`.
- Prefer stdin and prompt in docs.
- Store token through credential backend.
- Validate with a low-risk read command before marking active.
- Record selected location id, company id if known, and granted scope metadata if
  discoverable.
- Never print the full PIT token.

### 10.3 `auth login`

Bootstraps session auth with the GHL email/password and 2FA flow documented in
`references/ghl-internal-api-bible/01-auth/headless-login-3step.md`.

```bash
ghl auth login --profile default --email user@example.com
```

Requirements:

- Generate one `deviceId` and reuse it for all steps.
- Prompt for password without echo when not provided by stdin.
- Step 1 validates credentials.
- Step 2 requests email OTP using `otpChannel: email`.
- Step 3 submits OTP plus the step-two `token` field.
- Store only the returned token pairs and profile metadata.
- Support `--otp <code>` for scripted use.
- Support `--otp-command <cmd>` for controlled automation.
- Do not retry Step 1 aggressively because OTP emails can be rate-limited.
- On success, store `authToken` or equivalent current JWT, `refreshToken`,
  `jwt`, `refreshJwt`, user id, company id, role, account type, and permissions
  when present.

### 10.4 `auth refresh`

Refreshes session tokens for the profile.

```bash
ghl auth refresh --profile default
```

Requirements:

- Rotate refresh tokens atomically. If GHL returns a new refresh token, write it
  before returning success.
- Keep previous credential material only until the new credential is safely
  persisted.
- Report token class and expiration metadata without printing token values.
- Return exit code `3` when refresh fails due to expired or consumed token.

### 10.5 `auth firebase exchange`

Exchanges the Firebase custom token from login for Firebase ID and refresh tokens.

```bash
ghl auth firebase exchange --profile default
```

Requirements:

- Use the Firebase project information from the auth reference.
- Store Firebase refresh token securely.
- Refresh Firebase ID token automatically for Firebase surfaces.
- Return a clear setup hint when no Firebase custom token is available.

### 10.6 `auth status`

Reports available auth classes for a profile.

```bash
ghl auth status --profile default
```

Example output:

```json
{
  "profile": "default",
  "location_id": "location-id",
  "company_id": "company-id",
  "auth": {
    "pit": { "available": true, "validated_at": "2026-04-13T09:00:00Z" },
    "session": { "available": true, "expires_at": "2026-04-13T10:00:00Z" },
    "firebase": { "available": false, "hint": "Run `ghl auth firebase exchange`." }
  }
}
```

### 10.7 `auth export`

Exports credentials only when explicitly requested.

```bash
ghl auth export --profile default --unmasked > ghl-credentials.json
```

Requirements:

- Default export is masked.
- `--unmasked` requires `--yes` when stdout is a terminal.
- Export format includes schema version and profile metadata.
- Import/export tests must assert that secrets do not appear in normal logs.

## 11. Profile and Policy Feature Spec

### 11.1 Profile schema

Profiles live under the config directory and reference secrets by key.

```json
{
  "schema_version": 1,
  "default_profile": "default",
  "profiles": {
    "default": {
      "name": "default",
      "base_urls": {
        "services": "https://services.leadconnectorhq.com",
        "backend": "https://backend.leadconnectorhq.com"
      },
      "company_id": null,
      "location_id": null,
      "user_id": null,
      "credential_refs": {
        "pit": null,
        "session": null,
        "firebase": null
      },
      "policy": {
        "agent_mode": true,
        "default_dry_run": true,
        "allow_destructive": false,
        "allow_messaging": false,
        "allow_payment_actions": false,
        "allow_public_links": false,
        "allow_workflow_publish": false,
        "allow_phone_purchase": false,
        "allow_private_integration_token_create": false
      }
    }
  }
}
```

### 11.2 Policy enforcement

Policy checks happen before network requests. A denied command returns
`policy_denied` with exit code `12` and a hint showing the exact policy flag.

Sensitive actions:

| Action class | Policy flag |
| --- | --- |
| Delete resources | `allow_destructive` |
| Send SMS or email | `allow_messaging` |
| Send invoices, estimates, payment links, coupons, order payment records | `allow_payment_actions` |
| Create public links, trigger links, snapshot share links | `allow_public_links` |
| Publish or trigger workflows | `allow_workflow_publish` |
| Buy, release, or reconfigure phone numbers | `allow_phone_purchase` |
| Create PITs and API keys | `allow_private_integration_token_create` |

## 12. HTTP Client and API Surface Spec

### 12.1 API surfaces

| Surface | Base URL | Typical auth | Use |
| --- | --- | --- | --- |
| `services` | `https://services.leadconnectorhq.com` | PIT, OAuth, session | Public and hidden v2 APIs. |
| `backend` | `https://backend.leadconnectorhq.com` | session, cookie | Internal APIs, workflows, legacy backend. |
| `firebase` | Google Identity Toolkit and Firestore endpoints | Firebase refresh and ID token | A2P, Trust Center, selected domain flows. |

### 12.2 Required headers

Public/PIT requests use:

```json
{
  "Authorization": "Bearer <token>",
  "Content-Type": "application/json",
  "Accept": "application/json",
  "Version": "2021-07-28",
  "User-Agent": "Mozilla/5.0 ..."
}
```

Internal session requests use:

```json
{
  "Authorization": "Bearer <jwt>",
  "Content-Type": "application/json",
  "Accept": "application/json",
  "Version": "2021-07-28",
  "channel": "APP",
  "source": "WEB_USER"
}
```

The CLI must allow per-command version overrides for endpoints that require a
different `Version` header.

### 12.3 Rate limiting

The CRM reference states a 100 requests per minute limit per location. The client
must track rate limiting by location id, not globally.

Requirements:

- Token-bucket limiter per `location_id`.
- Respect upstream `x-ratelimit-*` headers when present.
- Retry 429 and 5xx responses with bounded exponential backoff and jitter.
- Never retry non-idempotent writes unless the command provides an idempotency key
  or the API behavior is documented as safe.
- Surface wait behavior on stderr when verbose mode is enabled.

### 12.4 Read caching

Read-only GET commands may use a short TTL cache.

Requirements:

- Default TTL: 30 seconds.
- Max entries: implementation-defined, with a documented default.
- `--no-cache` bypasses cache.
- Writes invalidate affected resource paths.
- Cache keys include profile, location id, surface, method, path, and query.

## 13. API Coverage Matrix

The CLI should eventually cover every API family represented in the local GHL
references. Full coverage does not mean every endpoint ships in v0.1. It means
every referenced surface has an owner namespace, auth model, phase, risk level,
and explicit status.

### 13.1 Coverage status values

| Status | Meaning |
| --- | --- |
| `mvp` | Required before the first serious agent-safe MVP. |
| `planned` | In scope after MVP. |
| `research` | Reference exists, but endpoint behavior needs validation before commands ship. |
| `deferred` | Known surface, intentionally postponed. |
| `out_of_scope` | Not part of this CLI. |

### 13.2 GHL internal API bible coverage

| Reference | Surface | CLI namespace | Auth class | Phase | Status | Risk |
| --- | --- | --- | --- | --- | --- | --- |
| `01-auth` | Auth, refresh, Firebase, PIT bootstrap, 2FA login | `auth` | pit, session, firebase, cookie | 1 | mvp | high |
| `02-workflows` | Workflow CRUD, triggers, actions, publish | `workflows` | session | 3 | planned | high |
| `03-funnels` | Funnels, pages, redirects | `funnels` | pit, session, oauth | 3 | planned | medium |
| `04-page-builder` | Page builder schemas and page mutations | `pages` or `funnels pages` | session, oauth | 5 | research | high |
| `05-memberships-courses` | Courses, products, offers, enrollments | `courses` | pit, session | 5 | planned | medium |
| `06-reputation` | Reviews, review requests, replies, settings | `reputation` | pit | 4 | planned | high |
| `07-domains` | Domain management and verification | `domains` | session, firebase | 5 | research | high |
| `08-snippets-templates` | Snippets and reusable message templates | `snippets`, `templates` | pit, session | 3 | planned | medium |
| `09-reporting-analytics` | Appointments, calls, attribution, pipeline stats | `reports` | pit | 4 | planned | low |
| `10-social-planner` | Social posts, accounts, CSV import | `social` | pit, oauth | 5 | planned | high |
| `11-snapshots` | Snapshot list, share, push to sub-accounts | `snapshots` | pit, session | 5 | planned | high |
| `12-custom-fields` | Contact/opportunity custom field CRUD | `custom-fields` | pit, session | 3 | planned | medium |
| `13-custom-values` | Location custom values | `custom-values` | session | 3 | planned | medium |
| `14-crm-contacts` | Contacts, conversations, messaging, tasks, notes | `contacts`, `conversations`, `messages` | pit, session | 2 | mvp | high |
| `15-pipelines` | Pipelines and stages | `pipelines`, `opportunities` | session, pit | 2 | mvp | medium |
| `16-forms` | Forms and submissions | `forms` | session | 3 | planned | medium |
| `17-surveys` | Surveys and submissions | `surveys` | session | 3 | planned | medium |
| `18-email-templates` | Email campaigns and templates | `templates email`, `email` | session, pit | 3 | planned | high |
| `19-sms-templates` | SMS templates | `templates sms` | session | 3 | planned | high |
| `20-trigger-links` | Trigger links and redirect links | `links` | session | 3 | planned | high |
| `21-media-library` | Media files and folders | `media` | session | 3 | planned | medium |
| `22-smart-lists` | Smart lists and dynamic segments | `smart-lists` | session | 3 | planned | medium |
| `23-a2p-compliance` | A2P and 10DLC compliance | `a2p` | firebase, session | 5 | research | high |
| `24-agent-studio` | GHL AI agent builder | `agent-studio` | oauth, session | 5 | research | high |
| `25-private-integration-tokens` | Remote Private Integration records and active tokens | `integrations pit` | session | 5 | planned | high |

### 13.3 MCP reference category coverage

The MCP reference remains a catalog for endpoint families and schema examples.
The CLI will not expose an MCP server.

| MCP module | Tool count | CLI namespace | Phase | Status |
| --- | ---: | --- | --- | --- |
| `affiliates-tools.ts` | 17 | `affiliates` | 5 | planned |
| `agent-studio-tools.ts` | 8 | `agent-studio` | 5 | research |
| `association-tools.ts` | 10 | `associations` | 5 | planned |
| `blog-tools.ts` | 7 | `blogs` | 5 | planned |
| `businesses-tools.ts` | 5 | `businesses` | 5 | planned |
| `calendar-tools.ts` | 39 | `calendars`, `appointments` | 2 | mvp |
| `campaigns-tools.ts` | 12 | `campaigns` | 5 | planned |
| `companies-tools.ts` | 5 | `companies` | 5 | planned |
| `contact-tools.ts` | 31 | `contacts` | 2 | mvp |
| `conversation-tools.ts` | 20 | `conversations`, `messages` | 2 | mvp |
| `courses-tools.ts` | 32 | `courses` | 5 | planned |
| `custom-field-v2-tools.ts` | 8 | `custom-fields` | 3 | planned |
| `custom-menus-tools.ts` | 5 | `custom-menus` | 5 | planned |
| `email-isv-tools.ts` | 9 | `email domains` | 5 | planned |
| `email-tools.ts` | 5 | `email`, `templates email` | 3 | planned |
| `forms-tools.ts` | 4 | `forms` | 3 | planned |
| `funnels-tools.ts` | 8 | `funnels` | 3 | planned |
| `invoices-tools.ts` | 18 | `invoices`, `estimates` | 4 | planned |
| `links-tools.ts` | 6 | `links` | 3 | planned |
| `location-tools.ts` | 28 | `locations` | 2 | mvp |
| `marketplace-tools.ts` | 7 | `integrations marketplace` | 5 | planned |
| `media-tools.ts` | 7 | `media` | 3 | planned |
| `oauth-tools.ts` | 10 | `integrations oauth`, `auth oauth` | 5 | planned |
| `object-tools.ts` | 9 | `objects` | 5 | planned |
| `opportunity-tools.ts` | 10 | `opportunities`, `pipelines` | 2 | mvp |
| `payments-tools.ts` | 22 | `payments`, `orders`, `coupons` | 4 | planned |
| `phone-system-tools.ts` | 15 | `phone` | 5 | planned |
| `phone-tools.ts` | 20 | `phone` | 5 | planned |
| `products-tools.ts` | 11 | `products` | 4 | planned |
| `proposals-tools.ts` | 4 | `proposals` | 4 | planned |
| `reporting-tools.ts` | 12 | `reports` | 4 | planned |
| `reputation-tools.ts` | 15 | `reputation` | 4 | planned |
| `saas-tools.ts` | 12 | `saas` | 5 | planned |
| `smartlists-tools.ts` | 8 | `smart-lists` | 3 | planned |
| `snapshots-tools.ts` | 7 | `snapshots` | 5 | planned |
| `social-media-tools.ts` | 19 | `social` | 5 | planned |
| `store-tools.ts` | 18 | `store`, `shipping` | 4 | planned |
| `survey-tools.ts` | 9 | `surveys` | 3 | planned |
| `templates-tools.ts` | 18 | `templates`, `snippets` | 3 | planned |
| `triggers-tools.ts` | 11 | `triggers` | 5 | planned |
| `users-tools.ts` | 7 | `users` | 5 | planned |
| `voice-ai-tools.ts` | 11 | `voice-ai` | 5 | planned |
| `webhooks-tools.ts` | 9 | `webhooks` | 5 | planned |
| `workflow-builder-tools.ts` | 7 | `workflows` | 3 | planned |
| `workflow-tools.ts` | 7 | `workflows` | 3 | planned |

### 13.4 Coverage metadata requirements

Every command must carry coverage metadata in `commands schema`:

```json
{
  "command": "contacts.search",
  "source_refs": ["ghl-internal-api-bible/14-crm-contacts/endpoints.md", "Go-High-Level-MCP-2026-Complete/src/tools/contact-tools.ts"],
  "surface": "services",
  "auth_classes": ["pit", "session"],
  "phase": 2,
  "status": "mvp",
  "risk": "medium",
  "api_confidence": "high"
}
```

## 14. Pagination Contract

GHL pagination is inconsistent across endpoint families. The CLI must normalize
pagination without hiding upstream metadata.

### 14.1 Pagination flags

All list commands should support the relevant subset of:

| Flag | Meaning |
| --- | --- |
| `--limit <n>` | Maximum items requested from one upstream page. |
| `--page <n>` | Page number for page-based endpoints. |
| `--cursor <value>` | Cursor or start-after token for cursor endpoints. |
| `--page-all` | Fetch all pages until exhausted or `--max-items` is reached. |
| `--max-items <n>` | Hard total item cap across pages. |
| `--format ndjson` | Stream items one JSON object per line. |

### 14.2 Normalized pagination output

List commands must include a `pagination` object when upstream pagination exists:

```json
{
  "items": [],
  "count": 0,
  "pagination": {
    "limit": 100,
    "returned": 0,
    "has_more": false,
    "next_page": null,
    "next_cursor": null,
    "raw": {}
  }
}
```

Rules:

- Preserve upstream pagination under `pagination.raw`.
- Use `next_page` for page-number APIs.
- Use `next_cursor` for cursor, `startAfter`, `startAfterId`, or token APIs.
- `count` is the number returned in this response, not total account count.
- If total count is known, expose `total` separately.
- `--page-all --format json` returns one aggregate object.
- `--page-all --format ndjson` streams each item and writes page progress to
  stderr only.
- `--page-all` must respect per-location rate limiting.

### 14.3 Pagination completion gate

A paginated command is complete when tests cover first page, final page, empty
page, malformed pagination metadata, and `--page-all` with a max item cap.

## 15. Bulk Execution Contract

Bulk operations are necessary, but they are where CRM mistakes become expensive.
Every bulk-capable command must share one execution model.

### 15.1 Bulk flags

| Flag | Meaning |
| --- | --- |
| `--from-file <path>` | Read JSON, JSONL, or CSV input. |
| `--input-format json|jsonl|csv` | Override input format detection. |
| `--concurrency <n>` | Maximum in-flight operations. Default: `1` for writes, `4` for reads. |
| `--continue-on-error` | Continue after item failures. Default: fail fast for writes. |
| `--max-errors <n>` | Abort once this many item failures occur. |
| `--resume-from <id-or-row>` | Resume a previously interrupted bulk run. |
| `--id-column <name>` | CSV column used as stable item id. |
| `--dry-run` | Preview all planned writes. Required by default when profile `default_dry_run` is true. |
| `--yes` | Confirm real bulk writes. |

### 15.2 Bulk result envelope

```json
{
  "ok": false,
  "dry_run": true,
  "bulk": {
    "operation": "contacts.upsert",
    "input_count": 3,
    "planned": 3,
    "succeeded": 0,
    "failed": 0,
    "skipped": 0,
    "concurrency": 1
  },
  "items": [
    {
      "index": 0,
      "input_id": "row-1",
      "status": "planned",
      "resource_id": null,
      "error": null
    }
  ]
}
```

### 15.3 Bulk safety rules

- Bulk writes require `--dry-run` first unless the user passes both `--yes` and a
  profile policy that allows the action class.
- Bulk messaging requires `allow_messaging`, `--yes`, and `--max-items` or
  `--max-recipients`.
- Bulk payment actions require `allow_payment_actions`, `--yes`, and item-level
  result logging.
- Bulk workflow triggers require `allow_workflow_publish`, `--yes`, and a max
  item cap.
- Bulk deletes require `allow_destructive`, `--yes`, and fail fast by default.
- Every real bulk write creates audit journal entries per item plus one parent
  bulk-run entry.

## 16. Idempotency, Dependency, and Duplicate Prevention

The CLI must protect users from duplicate creates, bad retries, and operations
whose dependencies do not belong together.

### 16.1 Idempotency keys

Write commands that create resources should accept:

```bash
--idempotency-key <key>
```

Rules:

- Use upstream idempotency support when available.
- When upstream support is unavailable, store a local idempotency record in the
  audit journal or a dedicated idempotency cache.
- The cache key includes profile, location id, command, idempotency key, and a
  redacted request hash.
- Reusing the same idempotency key with a different request hash must fail.
- Successful idempotent creates return the previous resource id when known.

### 16.2 Retry rules for writes

- GET and safe read operations may retry 429 and 5xx responses.
- PUT/PATCH may retry only when the command is idempotent by contract.
- POST create/send/trigger commands must not auto-retry after the request may
  have reached GHL unless an idempotency key is active.
- Message sends, invoice sends, workflow triggers, phone purchases, and snapshot
  pushes are never retried automatically without idempotency protection.

### 16.3 Duplicate prevention

Command-specific duplicate checks:

| Command family | Duplicate check |
| --- | --- |
| `contacts create` | Exact email and phone lookup when provided. |
| `contacts upsert` | Prefer update when exact email or external id exists. |
| `opportunities create` | Optional `--dedupe contact,pipeline,name` preflight. |
| `appointments create` | Free-slot check plus optional contact/time duplicate check. |
| `pipelines create` | Name uniqueness within location. |
| `templates create` | Name uniqueness within location and template type when supported. |
| `integrations pit create` | Name and scope preflight, plus account hard-limit check. |
| `phone numbers buy` | Availability check immediately before purchase. |

### 16.4 Dependency preflight

Before mutating, commands must validate resource dependencies that can be checked
cheaply:

- Contact belongs to the active location.
- Pipeline stage belongs to the selected pipeline.
- Opportunity belongs to the active location.
- Calendar belongs to the active location.
- Workflow belongs to the active location.
- Snapshot push target locations belong to the active company when discoverable.
- Custom field/object keys exist before record writes.

Dependency failures use error code `dependency_check_failed`.

## 17. Audit Log and Local Action Journal

Every real mutation and every dry-run for a sensitive action must write a local
audit journal entry. This is provenance for agents and operators.

### 17.1 Journal storage

Default location:

```text
<GHL_CLI_CONFIG_DIR>/audit/audit.jsonl
```

Requirements:

- File mode `0600` on Unix-like systems.
- Append-only writes.
- One JSON object per line.
- Secrets and sensitive bodies redacted before write.
- Journal writes must not block returning a successful remote mutation. If the
  journal write fails, return success with `audit_warning` metadata and print a
  warning to stderr.

### 17.2 Audit commands

```bash
ghl audit list [--from <datetime>] [--to <datetime>] [--action <name>] [--resource <id>]
ghl audit show <entry-id>
ghl audit export [--from <datetime>] [--to <datetime>] [--out <path>]
ghl audit prune --before <datetime> --yes
```

### 17.3 Audit entry shape

```json
{
  "id": "audit-entry-id",
  "timestamp": "2026-04-13T10:00:00Z",
  "profile": "default",
  "company_id": "company-id",
  "location_id": "location-id",
  "command": "opportunities.move",
  "action_class": "write",
  "dry_run": false,
  "policy_flags": ["allow_destructive"],
  "resource": {
    "type": "opportunity",
    "id": "opportunity-id"
  },
  "request_summary": {
    "surface": "services",
    "method": "PUT",
    "path_template": "/opportunities/{id}",
    "body_redacted": true
  },
  "upstream": {
    "status": 200,
    "request_id": null
  },
  "result": "success",
  "error": null
}
```

### 17.4 Audit redaction

Audit entries may include resource ids and command names. They must not include
secrets, OTPs, cookies, full JWTs, PIT values, message bodies, payment links,
raw invoice PDFs, or full workflow custom-code bodies.

## 18. Data Classification and Redaction Policy

The CLI uses data minimization by default. Output should contain the data needed
for the command and no more.

### 18.1 Data classes

| Class | Examples | Direct command output | Logs, audit, dry-run, fixtures |
| --- | --- | --- | --- |
| Secret | PITs, JWTs, refresh tokens, cookies, OTPs, passwords, API keys | Never printed except explicit unmasked export | Never printed |
| Auth metadata | token type, expiry, scope names, credential backend | Allowed | Allowed |
| Customer PII | names, emails, phones, addresses | Allowed for direct resource reads | Redacted or hashed by default |
| Message content | SMS bodies, email bodies, chat messages, voicemail transcripts | Allowed for direct message read commands | Redacted by default |
| Financial | invoice links, payment links, transaction ids, order details | Allowed for direct finance commands | Links and sensitive details redacted |
| Compliance | A2P brand/campaign details, Trust Center data | Allowed for direct compliance commands | Redacted summary only |
| Internal automation | workflow JSON, custom code, trigger payloads | Allowed only for explicit workflow get/export | Redacted summary by default |
| Public metadata | ids, counts, statuses, timestamps, names of public templates | Allowed | Allowed unless tied to PII |
| Operational metadata | command, duration, status, endpoint template, retry count | Allowed | Allowed |

### 18.2 Redaction rules

- Token-looking strings are redacted even outside known secret fields.
- Fields named `token`, `jwt`, `refreshToken`, `refreshJwt`, `password`, `otp`,
  `authorization`, `cookie`, `activeToken`, `apiKey`, or similar are redacted.
- Dry-run output for messages reports body length and hash, not body content, by
  default.
- Fixture capture must fail if redaction cannot prove the fixture is safe.
- `--unredacted` is allowed only for direct local export commands and must never
  be accepted by logs, audit, or fixture capture.

## 19. Scope Verification Contract

Private Integration Tokens are scoped. The CLI must detect missing scopes before
users learn about them through vague upstream failures.

### 19.1 Scope commands

```bash
ghl auth scopes
ghl auth scopes check contacts.write opportunities.readonly
ghl commands requirements contacts.create
```

Example output:

```json
{
  "profile": "default",
  "auth_class": "pit",
  "scopes": {
    "contacts.readonly": "available",
    "contacts.write": "missing"
  },
  "missing": ["contacts.write"]
}
```

### 19.2 Command requirement metadata

Every command must declare:

- Required auth class or classes.
- Required PIT scopes when PIT is supported.
- Required profile policy flags.
- Whether scope verification is exact, inferred, or unavailable.

If scopes cannot be inspected directly, the CLI may infer scope availability from
safe probe endpoints and must mark the result as `inferred`.

### 19.3 Scope failure behavior

Missing scope failures use error code `missing_scope` and should include the
minimum scope names required when known.

## 20. API Drift Detection Contract

Internal GHL APIs will change. The CLI must detect drift deliberately instead of
failing with confusing parse errors.

### 20.1 Endpoint confidence metadata

Each endpoint-backed command must declare:

```json
{
  "endpoint_key": "workflows.update",
  "surface": "backend",
  "method": "PUT",
  "path_template": "/workflow/{locationId}/{workflowId}",
  "source_refs": ["ghl-internal-api-bible/02-workflows/endpoints.md"],
  "api_confidence": "medium",
  "undocumented": true,
  "required_response_fields": ["id", "version"]
}
```

Confidence values:

| Value | Meaning |
| --- | --- |
| `high` | Documented or heavily fixture-tested endpoint. |
| `medium` | Internal endpoint with working references and tests. |
| `low` | Internal or unstable endpoint requiring smoke validation. |

### 20.2 Doctor commands

```bash
ghl doctor api [--include-internal]
ghl doctor endpoint <endpoint-key>
ghl doctor bundle --out <path> --redact
ghl doctor shapes --from-fixtures tests/fixtures/ghl
```

Requirements:

- Compare live safe-read responses to expected required fields.
- Report missing fields, renamed fields, unexpected auth failures, and status
  changes.
- Do not print customer data.
- Commands using undocumented endpoints must expose that fact in doctor output.
- Unexpected response shape errors use code `api_shape_changed`.

## 21. Fixture Capture Tooling

Fixture capture is a development feature for preserving API behavior without
leaking client data.

### 21.1 Dev commands

```bash
ghl dev capture-fixture <command-key> --redact --out tests/fixtures/ghl/<path>.json
ghl dev scan-fixture tests/fixtures/ghl/<path>.json
ghl dev scan-fixtures tests/fixtures/ghl
```

### 21.2 Capture rules

- `--redact` is mandatory.
- Capture refuses to write if token-like strings remain.
- Capture refuses to write if obvious emails, phone numbers, payment links, or
  message bodies remain outside explicitly synthetic fields.
- Request metadata may include method, surface, endpoint template, query keys,
  and redacted body keys.
- Authorization, Cookie, and full URLs with tokens must never be written.
- Fixtures should preserve upstream shape and pagination metadata.

### 21.3 Fixture metadata

Captured fixtures include a metadata wrapper:

```json
{
  "fixture": {
    "endpoint_key": "contacts.search",
    "captured_at": "2026-04-13T10:00:00Z",
    "redacted": true,
    "source_ref": "references/Go-High-Level-MCP-2026-Complete/src/tools/contact-tools.ts"
  },
  "request": {
    "surface": "services",
    "method": "GET",
    "path_template": "/contacts/"
  },
  "response": {}
}
```

## 22. Signet Integration Contract

Signet integration is a first-class optional path for agent runtimes. The CLI
must work without Signet, but when Signet is available it should use Signet
secrets without exposing values to the model context.

This contract is based on `docs/SECRETS.md` in the local Signet repo, where
secret values are stored encrypted, `signet secret list` and `signet secret has`
show names only, `$secret:NAME` references are resolved internally, and
subprocess execution can inject secrets while redacting them from output.

### 22.1 Signet credential references

Profile credential references may point to Signet secrets:

```json
{
  "credential_refs": {
    "pit": { "type": "signet", "name": "GHL_PIT_TOKEN" },
    "session_password": { "type": "signet", "name": "GHL_PASSWORD" },
    "firebase": { "type": "signet", "name": "GHL_FIREBASE_REFRESH" }
  }
}
```

1Password references managed through Signet may use `op://vault/item/field`.

### 22.2 Signet-aware commands

```bash
ghl auth pit add --token-signet GHL_PIT_TOKEN
ghl auth login --password-signet GHL_PASSWORD
ghl auth firebase exchange --secret-signet GHL_FIREBASE_CUSTOM_TOKEN
ghl signet doctor
```

Requirements:

- `ghl signet doctor` checks whether Signet is available and whether named
  secrets exist, without retrieving values into output.
- Signet secret names may be printed. Secret values must never be printed.
- If Signet resolution fails, return `signet_secret_unavailable` with a setup
  hint.
- Signet is optional. Normal keyring and file credential backends remain
  supported.

### 22.3 Agent execution model

When an agent needs to run a command with Signet secrets, the preferred pattern is
Signet secret execution so secrets are injected into the subprocess environment
and redacted from stdout/stderr. The CLI should document this path in the agent
skill once implementation exists.


## 23. Permission and Capability Discovery Contract

Auth success is not the same thing as command capability. GHL permissions can be
restricted by agency role, location role, user permissions, feature flags,
installed products, scopes, and account plan. The CLI must expose capability
checks directly.

### 23.1 Capability commands

```bash
ghl capabilities
ghl capabilities location <location-id>
ghl capabilities command <command-key>
```

Example output:

```json
{
  "profile": "default",
  "company_id": "company-id",
  "location_id": "location-id",
  "user": {
    "id": "user-id",
    "role": "admin",
    "type": "agency"
  },
  "auth_classes": ["pit", "session"],
  "features": {
    "contacts": "available",
    "workflows": "available",
    "a2p": "unknown"
  },
  "commands": {
    "contacts.search": "expected_available",
    "workflows.update": "requires_session",
    "messages.send_sms": "blocked_by_policy"
  }
}
```

### 23.2 Capability sources

Capability detection may use:

- Auth status and token claims.
- Known PIT scopes.
- Safe read probes.
- User and location metadata.
- GHL permissions fields returned during login.
- Feature-specific safe endpoints.

Each capability result must identify confidence:

| Value | Meaning |
| --- | --- |
| `known` | Directly reported by GHL or local policy. |
| `inferred` | Derived from successful safe probes or token claims. |
| `unknown` | Not checked or not knowable without a risky call. |

### 23.3 Capability failure behavior

Commands blocked by permissions, plan, feature availability, or role must return
`capability_unavailable` instead of a generic auth error when the distinction is
known.

## 24. Messaging Compliance Contract

Messaging commands must respect contact-level and channel-level communication
state before sending SMS, email, review requests, or other outbound messages.

### 24.1 Pre-send checks

Before any real send, the CLI should check what it can safely determine:

- Contact exists and belongs to the active location.
- Required destination exists, such as phone for SMS or email for email.
- Contact DND state.
- Channel-specific DND state when exposed by GHL.
- Email unsubscribe status when exposed by GHL.
- SMS unsubscribe or opt-out status when exposed by GHL.
- A2P or phone-number compliance state when detectable.
- Message type, such as transactional or marketing, when the API supports it.

### 24.2 Compliance behavior

- If DND or unsubscribe status is known and blocking, the command must refuse the
  send by default.
- Override requires a dedicated compliance override flag, policy allowance, and
  explicit confirmation. The initial MVP should not implement override unless a
  real use case demands it.
- Bulk messaging must report counts of skipped contacts by reason.
- Dry-runs should include compliance status summaries without printing message
  bodies.

Example dry-run output:

```json
{
  "ok": true,
  "dry_run": true,
  "action": "messages.send_sms",
  "compliance": {
    "checked": true,
    "sendable": false,
    "reasons": ["contact_dnd_enabled"]
  }
}
```

## 25. Dry-run Fidelity Contract

Dry-run must be explicit about whether it performs network preflight checks.

### 25.1 Dry-run modes

Commands that support dry-run should accept:

```bash
--dry-run
--dry-run=validated
--dry-run=local
```

| Mode | Network behavior | Use |
| --- | --- | --- |
| `local` | No network calls. Builds and redacts the planned request from local inputs only. | Safe offline planning and fixture review. |
| `validated` | Allows safe reads for auth, scope, dependency, duplicate, capability, and compliance checks. No mutations. | Default for agents when online. |

`--dry-run` without a value means `validated` unless `--offline` is active, in
which case it means `local`.

### 25.2 Dry-run output requirements

Dry-run output must include:

- `dry_run: true`.
- `dry_run_mode: local|validated`.
- Planned action and resource type.
- Whether dependencies were checked.
- Whether scopes were checked.
- Whether compliance was checked for messaging.
- Redacted request summary.
- Exact policy flags required for real execution.

## 26. Input Schema and Validation Contract

File-driven commands must expose schemas so agents can generate valid payloads
without guessing.

### 26.1 Schema commands

```bash
ghl schema list
ghl schema show <schema-key>
ghl schema validate <schema-key> --file <path>
ghl schema example <schema-key>
```

Examples:

```bash
ghl schema show contacts.bulk-upsert
ghl schema validate contacts.bulk-upsert --file contacts.csv
ghl schema example workflows.update > workflow.example.json
```

### 26.2 Schema requirements

- Every `--from-file` command must have a schema key.
- Schemas may be JSON Schema or a documented Rust-derived equivalent.
- CSV commands must document required columns, optional columns, and type
  coercion rules.
- Validation must run before any network mutation.
- Schema validation errors use code `schema_validation_failed`.
- `commands schema` must reference the relevant input schema keys.

## 27. Rollback and Compensation Contract

The audit journal records what happened. The CLI should also know what can be
undone and what cannot.

### 27.1 Undo commands

```bash
ghl undo plan <audit-entry-id>
ghl undo apply <audit-entry-id> --yes
ghl undo bulk-plan <bulk-run-id>
ghl undo bulk-apply <bulk-run-id> --yes
```

### 27.2 Undo metadata

Every write command must declare one undo class:

| Undo class | Meaning |
| --- | --- |
| `supported` | CLI can automatically perform a reasonable inverse operation. |
| `partial` | CLI can suggest or perform a partial compensation. |
| `unsupported` | No safe undo exists. |

Examples:

| Operation | Undo class | Compensation |
| --- | --- | --- |
| Add contact tag | `supported` | Remove the same tag. |
| Move opportunity | `supported` when previous stage is known | Move back to previous stage. |
| Create note | `supported` | Delete created note. |
| Create appointment | `supported` | Delete created appointment. |
| Send SMS/email | `unsupported` | No undo, only follow-up message. |
| Send invoice | `partial` | Void/cancel only if supported and not paid. |
| Trigger workflow | `unsupported` | Manual review required. |
| Push snapshot | `unsupported` | Manual review required. |
| Buy phone number | `partial` | Release number only if acceptable. |

### 27.3 Undo safety

- Undo apply requires `--yes`.
- Undo must run dependency checks before mutating.
- Undo writes its own audit entry linked to the original entry.
- Unsupported undo must produce a human-readable explanation and no network
  mutation.

## 28. Output Schema Versioning Contract

Stable JSON needs explicit schema versioning so agents can depend on output
without brittle parsing.

### 28.1 Command output version metadata

Every command entry in `commands schema` must include:

```json
{
  "command": "contacts.search",
  "output_schema_version": 1,
  "output_schema_key": "contacts.search.output.v1"
}
```

### 28.2 Versioning rules

- Adding optional fields is non-breaking.
- Removing fields is breaking.
- Renaming fields is breaking.
- Changing field type is breaking.
- Changing nullability from nullable to required is breaking.
- Breaking changes increment the output schema version.
- Commands may support older output schema versions in the future, but v0.1 only
  needs to expose the current version clearly.

## 29. Long-running Job Contract

Some GHL operations are delayed, asynchronous, or large enough that they should
not be treated as simple request/response calls.

### 29.1 Job commands

```bash
ghl jobs list
ghl jobs get <job-id>
ghl jobs wait <job-id> [--timeout <duration>]
ghl jobs cancel <job-id> [--dry-run] --yes
```

### 29.2 Job sources

The job model applies to:

- Bulk operations.
- Snapshot pushes.
- CSV imports.
- Social planner CSV uploads.
- Workflow publish propagation when detectable.
- Media processing when detectable.
- Any command that returns an upstream job id or requires polling.

### 29.3 Local job shape

```json
{
  "id": "job-id",
  "created_at": "2026-04-13T10:00:00Z",
  "profile": "default",
  "location_id": "location-id",
  "command": "snapshots.push",
  "status": "running",
  "upstream_job_id": "upstream-id",
  "audit_entry_id": "audit-id"
}
```

Jobs may be local-only wrappers around upstream operations.

## 30. Resource Locking and Race Prevention Contract

Multiple agents or shells may run the CLI at the same time. The CLI must use
simple local locks around state that can be corrupted or duplicated.

### 30.1 Lock targets

Use file locks or equivalent for:

- Config writes.
- Credential refresh and token rotation.
- Audit journal writes.
- Idempotency cache writes.
- Bulk resume state.
- Fixture capture writes.
- Per-resource high-risk mutations when resource id is known.

### 30.2 Lock behavior

- Default lock wait should be short and bounded.
- `--lock-timeout <duration>` may override the wait.
- Failure to acquire a lock returns `resource_locked`.
- Lock files must not contain secrets.
- Stale lock recovery must be conservative and logged.

## 31. Test Account, Seed Data, and Smoke Cleanup Contract

Real-account smoke tests need recognizable test resources and safe cleanup.

### 31.1 Smoke marker

All smoke-created resources must use a recognizable marker:

```text
ghl-cli-smoke-<timestamp>
```

When possible, also attach a tag:

```text
ghl-cli-smoke
```

### 31.2 Smoke commands

```bash
ghl smoke run [--skip-writes] [--include-internal] [--include-messaging-dry-run]
ghl smoke seed [--dry-run] [--yes]
ghl smoke cleanup --before <datetime> [--dry-run] --yes
```

### 31.3 Cleanup rules

- Cleanup may only delete resources with the smoke marker or smoke tag.
- Cleanup must refuse unmarked resources.
- Cleanup requires `--yes` for real deletion.
- Cleanup should print counts and resource types, not customer data.
- Smoke seed must be optional and should prefer dry-run.

## 32. Config Import, Export, and Migration Contract

The CLI needs a complete local configuration lifecycle.

### 32.1 Config commands

```bash
ghl config export [--out <path>] [--unmasked]
ghl config import <path> [--dry-run] [--yes]
ghl config migrate [--dry-run] [--yes]
ghl config doctor [--repair] [--dry-run] [--yes]
```

### 32.2 Config rules

- Export masks secrets by default.
- Unmasked export requires `--unmasked` and `--yes` when stdout is a terminal.
- Import validates schema before writing.
- Migration must be atomic.
- Repair must never delete credentials.
- Config backups must not contain unmasked secrets unless explicitly requested.

## 33. Offline Mode Contract

Agents often need to inspect local state without touching the network.

### 33.1 Offline flag

```bash
--offline
```

### 33.2 Offline-allowed commands

Offline mode allows:

- `commands schema`.
- `commands requirements`.
- `config path`, `config show`, `config export`, `config doctor` without repair.
- `profiles list`, `profiles show`, `profiles policy show`.
- `audit list`, `audit show`, `audit export`.
- `schema list`, `schema show`, `schema validate`, `schema example`.
- `dev scan-fixture`, `dev scan-fixtures`.
- `raw request --dry-run=local`.
- Local dry-run planning where no network preflight is required.

### 33.3 Offline behavior

- Offline mode must fail before auth refresh or network calls.
- Commands requiring network return `offline_unavailable`.
- `--dry-run=validated --offline` downgrades to local dry-run only when the
  command can still produce a useful result. Otherwise it fails clearly.

## 34. Human Confirmation UX Contract

Dangerous commands must require concrete confirmation in interactive terminals
and fail safely in non-interactive environments.

### 34.1 Interactive confirmation

When stdin is a TTY and `--yes` is absent, dangerous commands may prompt with a
specific phrase:

```text
This will send 42 SMS messages from location DZEp...
Type SEND 42 to continue:
```

Examples:

| Action | Confirmation phrase |
| --- | --- |
| Bulk SMS send | `SEND <count>` |
| Bulk delete | `DELETE <count>` |
| Workflow publish | `PUBLISH <workflow-id>` |
| Snapshot push | `PUSH <count>` |
| Phone purchase | `BUY <number>` |

### 34.2 Non-interactive behavior

When stdin is not a TTY, commands must not prompt. They return
`confirmation_required` with a hint showing the required `--yes` flag and policy
flag.

### 34.3 Confirmation output

Dry-run output for dangerous commands must show the exact confirmation phrase
that would be required for real execution.

## 35. Environment, Proxy, and Network Configuration Contract

GHL sits behind Cloudflare and the CLI will run from desktops, servers, CI, and
agent sandboxes. Network behavior must be explicit.

### 35.1 Network inputs

Support standard environment variables where the HTTP stack supports them:

| Variable | Meaning |
| --- | --- |
| `HTTPS_PROXY` | HTTPS proxy. |
| `HTTP_PROXY` | HTTP proxy. |
| `NO_PROXY` | Proxy bypass list. |
| `SSL_CERT_FILE` | Custom CA bundle file when supported. |
| `GHL_CLI_TIMEOUT` | Default request timeout. |
| `GHL_CLI_RETRY_MAX` | Max retries for retryable reads. |
| `GHL_CLI_USER_AGENT` | Browser-like User-Agent override. |

### 35.2 Network docs

`docs/NETWORK.md` must document:

- API surfaces and base URLs.
- Required headers.
- Cloudflare User-Agent behavior for PIT calls.
- Proxy variables.
- TLS and custom CA behavior.
- Timeouts.
- Retry limits.
- Rate-limit behavior.

## 36. Security Threat Model

The security model should be short, concrete, and maintained with the code.

### 36.1 Threats to address

| Threat | Mitigation |
| --- | --- |
| Prompt injection tries to exfiltrate secrets | Secrets are referenced by name, redacted in output, and never printed by normal commands. |
| Fixture capture leaks customer data | Mandatory redaction, scanner deny rules, no write if unsafe strings remain. |
| Agent performs destructive action accidentally | Policy gates, dry-run defaults, `--yes`, confirmation phrases, audit journal. |
| Bulk send mistake | Bulk dry-run, max recipient caps, compliance checks, per-item result records. |
| Refresh-token race loses valid token | Credential refresh lock and atomic rotation. |
| API drift mutates wrong field | Endpoint confidence metadata, shape checks, fixture tests, drift doctor. |
| Local config permissions expose credentials | Owner-only files, OS keyring preferred, config doctor warnings. |
| Logs expose message content or payment links | Data classification and redaction policy. |

### 36.2 Security documentation requirement

`docs/SECURITY.md` or a dedicated security section in README must include:

- Unofficial project disclaimer.
- Secret handling model.
- Redaction model.
- Agent safety model.
- Responsible disclosure or issue-reporting guidance once public.

## 37. Unofficial Product Disclaimer Contract

Every user-facing distribution surface must state that this is an unofficial CLI
and is not affiliated with, endorsed by, or supported by Go High Level,
HighLevel, or LeadConnector.

Required surfaces:

- README.
- npm package description or README.
- GitHub repository description where practical.
- `--help` footer or about text.
- Release notes.
- Docs site or generated docs.

The disclaimer should be plain and short. No need to laminate the pancake.


## 38. Machine-readable Endpoint Manifest Contract

The coverage matrix explains ownership at the category level. The endpoint
manifest is the machine-readable source of truth for individual endpoints.

### 38.1 Endpoint commands

```bash
ghl endpoints list [--status <status>] [--surface <surface>]
ghl endpoints show <endpoint-key>
ghl endpoints coverage
```

### 38.2 Endpoint manifest shape

```json
{
  "endpoint_key": "contacts.search",
  "surface": "services",
  "method": "GET",
  "path_template": "/contacts/",
  "auth_classes": ["pit", "session"],
  "pit_scopes": ["contacts.readonly"],
  "source_refs": [
    "references/ghl-internal-api-bible/14-crm-contacts/endpoints.md",
    "references/Go-High-Level-MCP-2026-Complete/src/tools/contact-tools.ts"
  ],
  "cli_commands": ["contacts.search"],
  "status": "mvp",
  "api_confidence": "high",
  "undocumented": false,
  "required_response_fields": ["contacts"]
}
```

### 38.3 Endpoint manifest rules

- Every endpoint used by a command must have one endpoint manifest entry.
- Every endpoint manifest entry should link back to source references.
- `ghl endpoints coverage` reports referenced endpoints without command coverage,
  command endpoints without fixture coverage, and undocumented endpoints without
  drift checks.
- Manifest entries must not contain account-specific ids, tokens, or customer
  data.

## 39. Context Resolution Contract

Commands that act on GHL resources need a deterministic company and location
context.

### 39.1 Context commands

```bash
ghl context show
ghl context resolve --command <command-key>
ghl context set-location <location-id>
ghl context set-company <company-id>
```

### 39.2 Context precedence

Resolve context in this order:

1. Explicit CLI flag, such as `--location` or `--company`.
2. Environment variable, such as `GHL_CLI_LOCATION_ID`.
3. Profile default.
4. Token claims or auth response metadata when unambiguous.
5. Command-specific fallback, only for read-only discovery commands.

If a mutating command requires `location_id` and multiple locations are
available, the CLI must fail with `ambiguous_context` and ask for `--location` or
a profile default. Wrong-location writes are not an acceptable failure mode.

### 39.3 Context output shape

```json
{
  "profile": "default",
  "company_id": {
    "value": "company-id",
    "source": "profile"
  },
  "location_id": {
    "value": "location-id",
    "source": "--location"
  },
  "ambiguous": false
}
```

## 40. Query, Filter, Sort, and Field Selection Contract

List and search commands need consistent shaping controls so agents can request
only what they need.

### 40.1 Common flags

Commands should support these flags where applicable:

| Flag | Meaning |
| --- | --- |
| `--filter <expr>` | Command-specific filter expression. |
| `--sort <field:asc|desc>` | Sort output or pass upstream sort when supported. |
| `--fields <csv>` | Select fields in CLI output. |
| `--include-raw` | Include upstream object under `raw`. |
| `--raw` | Return minimally processed upstream response. |

### 40.2 Rules

- `--fields` affects CLI output only unless the endpoint explicitly supports
  upstream field selection.
- `--include-raw` preserves normalized output and adds upstream payloads.
- `--raw` bypasses normalization where safe, but still applies secret redaction.
- Filter syntax is command-specific and must be documented in command metadata.
- Unknown fields in `--fields` return `unknown_field` unless `--ignore-missing-fields`
  is passed.

## 41. Debug and Support Bundle Contract

Users should be able to produce a redacted support bundle without pasting secrets
or customer data into chat.

### 41.1 Bundle command

```bash
ghl doctor bundle --out ghl-debug.zip --redact
```

### 41.2 Bundle contents

A support bundle may include:

- CLI version, platform, install method, and build commit.
- Config with secrets masked.
- Profile metadata with credential refs, not credential values.
- Command metadata.
- Endpoint manifest.
- Recent audit entries, redacted.
- Recent local error logs, redacted.
- Fixture scanner results.
- Network configuration summary without proxy credentials.
- `doctor api` output without customer data.

### 41.3 Bundle rules

- Redaction is mandatory.
- Bundle creation must run the fixture/data scanner against included files.
- Bundle creation fails if unsafe strings remain.
- Bundles must not include local credential stores, Signet secret stores, raw
  fixtures that fail scanning, or unredacted message bodies.

## 42. Credential Fallback Encryption Contract

GHL credentials are high-value. The credential backend must prefer secure stores
and make insecure fallbacks obvious.

### 42.1 Backend preference order

1. Signet secret reference, when explicitly configured or running in a Signet
   agent environment.
2. OS keyring, when available.
3. Encrypted local file fallback, when implemented.
4. Plain owner-only local file fallback, only with explicit warning.

### 42.2 File fallback rules

- Plaintext fallback requires an explicit config choice or confirmation in
  interactive setup.
- `auth status` and `config doctor` must warn when plaintext fallback is active.
- File fallback secrets must never be included in config export unless
  `--unmasked` is explicitly requested.
- If encryption is implemented, document algorithm, key derivation, storage path,
  and recovery limitations.

## 43. Token Lifecycle and Clock-skew Contract

Rotating tokens create footguns. The CLI must make token refresh predictable.

### 43.1 Refresh timing

- Refresh session tokens before expiry with a default five-minute safety window.
- Account for local clock skew by treating near-expiry tokens as expired.
- Use credential locks so parallel commands do not refresh the same token at the
  same time.
- Never overwrite a newer refresh token with an older one.
- Preserve the previous token only until the new token is validated and stored.

### 43.2 Token error classification

When detectable, auth errors should distinguish:

| Error code | Meaning |
| --- | --- |
| `token_expired` | Access token expired and refresh was unavailable or failed. |
| `refresh_token_consumed` | Rotating refresh token was already used. |
| `token_revoked` | Token appears revoked or invalidated. |
| `clock_skew_detected` | Local clock likely invalidates token timing. |
| `auth_state_corrupt` | Stored token metadata is inconsistent. |

## 44. Standard Error-code Registry Contract

Named error codes must be centralized so agents do not pattern-match prose.

### 44.1 Error commands

```bash
ghl errors list
ghl errors show <error-code>
```

### 44.2 Error registry requirements

Each error entry includes:

- Stable error code.
- Exit code.
- Description.
- Whether retry may help.
- Whether user action is required.
- Common hints.

Examples:

| Error code | Exit | Meaning |
| --- | ---: | --- |
| `missing_scope` | 4 | PIT lacks required scope. |
| `policy_denied` | 12 | Local profile policy blocked action. |
| `capability_unavailable` | 13 | Feature, plan, role, or permission blocks command. |
| `api_shape_changed` | 6 | Upstream response shape drifted. |
| `schema_validation_failed` | 14 | File input failed schema validation. |
| `confirmation_required` | 15 | Non-interactive or missing confirmation. |
| `resource_locked` | 16 | Local lock unavailable. |
| `offline_unavailable` | 17 | Command cannot run offline. |
| `ambiguous_context` | 2 | Company or location context is ambiguous. |

## 45. Currency, Timezone, and Locale Normalization Contract

Calendar, reporting, invoices, payments, opportunities, and revenue commands must
normalize money and time consistently.

### 45.1 Money rules

- Do not use floating point internally for money.
- Preserve upstream minor units when provided.
- JSON money values should use an object shape:

```json
{
  "amount": "15000.00",
  "currency": "USD",
  "minor_units": 1500000,
  "source": "upstream"
}
```

### 45.2 Time rules

- Timestamps normalize to RFC 3339.
- Date-only values remain date-only.
- Include timezone source when relevant: `profile`, `location`, `upstream`, or
  `explicit`.
- Calendar writes must validate local timezone assumptions before mutation.
- Reports must include the effective timezone used for date ranges.

### 45.3 Locale rules

- CLI JSON output should not localize numbers or dates.
- Human table output may localize only when explicitly requested in future work.

## 46. Data Export Privacy Contract

Export commands can move large amounts of PII. They need stricter defaults than
small direct reads.

### 46.1 Export commands

```bash
ghl contacts export --out <path> [--format jsonl|csv]
ghl opportunities export --out <path> [--format jsonl|csv]
ghl reports export --report <name> --out <path> [--format json|csv]
```

### 46.2 Export rules

- Exports containing PII require `--out`; default stdout export is refused unless
  `--allow-stdout` is passed.
- Export output path must not be inside committed fixture directories unless the
  command is explicitly a redacted fixture capture.
- Exports write an audit entry.
- Export dry-runs report expected fields, filters, and estimated page behavior
  without writing data.
- Export commands support `--fields` and redaction options where practical.

## 47. Retention and Maintenance Contract

Local state should not grow forever.

### 47.1 Maintenance commands

```bash
ghl maintenance status
ghl maintenance prune [--audit-before <datetime>] [--jobs-before <datetime>] [--idempotency-before <datetime>] [--dry-run] --yes
```

### 47.2 Default retention

| State | Default retention |
| --- | --- |
| Audit journal | Until pruned by user. |
| Idempotency records | 30 days. |
| Local jobs | 30 days after terminal state. |
| Local redacted error logs | 30 days. |
| Debug bundles | Not managed unless stored under CLI config dir. |

### 47.3 Prune rules

- Prune supports dry-run.
- Prune never deletes credentials.
- Prune never deletes fixtures outside configured generated-fixture paths.
- Prune writes an audit entry when it deletes state.


## 48. Contacts Feature Spec

### 48.1 Common contact object

```json
{
  "id": "contact-id",
  "location_id": "location-id",
  "first_name": "Sarah",
  "last_name": "Example",
  "email": "sarah@example.com",
  "phone": "+15551234567",
  "tags": ["lead"],
  "source": null,
  "created_at": null,
  "updated_at": null,
  "raw": {}
}
```

### 48.2 Commands

```bash
ghl contacts search [<query>] [--limit <n>] [--email <email>] [--phone <phone>]
ghl contacts get <contact-id>
ghl contacts create [...fields] [--dry-run]
ghl contacts update <contact-id> [...fields] [--dry-run]
ghl contacts upsert [...fields] [--dry-run]
ghl contacts delete <contact-id> [--dry-run] --yes
ghl contacts tags add <contact-id> <tag>... [--dry-run]
ghl contacts tags remove <contact-id> <tag>... [--dry-run]
ghl contacts tasks list <contact-id>
ghl contacts tasks create <contact-id> --title <text> [--due-at <datetime>] [--dry-run]
ghl contacts notes list <contact-id>
ghl contacts notes create <contact-id> --body <text>|--from-file <path> [--dry-run]
ghl contacts timeline <contact-id> [--limit <n>]
```

Requirements:

- Search by email must use exact email filtering when provided because fuzzy
  search can return loose matches.
- Search and get require resolved location context even when the upstream contact
  get endpoint only takes a contact id.
- Create and upsert must check for duplicate email when email is present.
- Delete requires `--yes` and `allow_destructive`.
- Note bodies may be redacted in dry-run output.

## 49. Conversations and Messaging Feature Spec

### 49.1 Commands

```bash
ghl conversations search [--contact <id>] [--query <query>] [--status all|read|unread|starred|recents] [--limit <n>]
ghl conversations get <conversation-id>
ghl conversations messages <conversation-id> [--limit <n>] [--last-message-id <id>] [--message-type <type>]
ghl messages send-sms --contact <id>|--conversation <id> --body <text>|--from-file <path> [--dry-run] [--yes]
ghl messages send-email --contact <id>|--conversation <id> --subject <text> --body <text>|--from-file <path> [--dry-run] [--yes]
ghl messages cancel <message-id> [--dry-run] --yes
```

Requirements:

- Message sends require `allow_messaging` and `--yes` unless `--dry-run` is used.
- Message bodies must not appear in verbose request logs.
- Dry-run output reports `body_redacted: true` and byte length.
- Email commands must preserve subject in dry-run output unless `--redact-subject`
  is passed.
- Conversation reads require resolved location context.
- Conversation message bodies and preview bodies are redacted from normal output.
- Conversation reads should support pagination.

## 50. Opportunities and Pipelines Feature Spec

### 50.1 Opportunity commands

```bash
ghl opportunities search [--query <q>] [--pipeline <id>] [--stage <id>] [--status open|won|lost|abandoned] [--limit <n>]
ghl opportunities get <opportunity-id>
ghl opportunities create --name <name> --contact <id> --pipeline <id> --stage <id> [--value <amount>] [--dry-run]
ghl opportunities update <opportunity-id> [...fields] [--dry-run]
ghl opportunities move <opportunity-id> --stage <stage-id> [--dry-run]
ghl opportunities delete <opportunity-id> [--dry-run] --yes
```

### 50.2 Pipeline commands

```bash
ghl pipelines list
ghl pipelines get <pipeline-id>
ghl pipelines create --name <name> --stages <csv|json|@file> [--dry-run]
ghl pipelines update <pipeline-id> [--name <name>] [--stages <csv|json|@file>] [--dry-run]
ghl pipelines delete <pipeline-id> [--dry-run] --yes
```

Pipeline rules from the reference:

- Do not include `locationId` in pipeline update bodies.
- GHL auto-creates Won and Lost stages. Commands must not add them manually.
- Existing stages require `id` during update. New stages omit `id`.
- Pipeline names must be unique within a location.
- Stage order is position-based and zero-indexed.

## 51. Calendar and Appointments Feature Spec

```bash
ghl calendars groups list
ghl calendars list [--group <id>]
ghl calendars get <calendar-id>
ghl calendars events --calendar <id> [--from <datetime>] [--to <datetime>] [--date <date>]
ghl calendars free-slots --calendar <id> --date <date>
ghl appointments create --calendar <id> --contact <id> --starts-at <datetime> --ends-at <datetime> [--dry-run]
ghl appointments update <appointment-id> [...fields] [--dry-run]
ghl appointments delete <appointment-id> [--dry-run] --yes
ghl appointments notes list <appointment-id>
ghl appointments notes create <appointment-id> --body <text>|--from-file <path> [--dry-run]
```

Requirements:

- Appointment creation should validate start/end ordering locally.
- Free-slot reads should be available before appointment creation.
- Delete requires `--yes` and `allow_destructive`.
- Timezone handling must preserve profile location timezone when known.

## 52. Workflows Feature Spec

### 52.1 Commands

```bash
ghl workflows list [--status active|inactive|draft]
ghl workflows get <workflow-id> [--full]
ghl workflows status <workflow-id> --status active|inactive [--dry-run]
ghl workflows trigger <workflow-id> --contact <contact-id> [--dry-run] [--yes]
ghl workflows validate --from-file <path>
ghl workflows create --name <name> [--from-file <path>] [--dry-run]
ghl workflows update <workflow-id> --from-file <path> [--dry-run]
ghl workflows publish <workflow-id> [--dry-run] --yes
ghl workflows delete <workflow-id> [--dry-run] --yes
```

### 52.2 Workflow update rules

Workflow builder commands use the internal API documented in
`references/ghl-internal-api-bible/02-workflows/endpoints.md`.

Requirements:

- Full update requires GET first to read the current version.
- PUT must include the current `version`.
- Branching `next` values must be arrays.
- `createdSteps`, `modifiedSteps`, `deletedSteps`, `oldTriggers`,
  `newTriggers`, and `triggersChanged` must be constructed according to the
  reference.
- Publish and trigger require `allow_workflow_publish`.
- Delete requires `allow_destructive`.
- `validate` performs schema checks locally and does not call the network.

## 53. Marketing, Content, and Assets Feature Spec

These commands are Phase 2 unless needed earlier by client work.

```bash
ghl funnels list
ghl funnels get <funnel-id>
ghl funnels pages <funnel-id>
ghl forms list
ghl forms get <form-id>
ghl forms submissions <form-id> [--from <date>] [--to <date>]
ghl surveys list
ghl surveys get <survey-id>
ghl surveys submissions <survey-id> [--from <date>] [--to <date>]
ghl media list [--folder <id>]
ghl media upload <local-path> [--folder <id>] [--dry-run]
ghl media delete <media-id> [--dry-run] --yes
ghl templates sms list
ghl templates sms create --name <name> --body <text>|--from-file <path> [--dry-run]
ghl templates email list
ghl templates email create --name <name> --subject <text> --body <html|@file> [--dry-run]
ghl snippets list
ghl snippets create --name <name> --body <text>|--from-file <path> [--dry-run]
ghl links list
ghl links create --name <name> --url <url> [--dry-run]
ghl social posts search [...filters]
ghl social posts create --from-file <path> [--dry-run]
```

Requirements:

- Public or trackable link creation requires `allow_public_links`.
- Media upload dry-run reports file metadata and target folder, not file bytes.
- Template body logging must be redacted by default.

## 54. Custom Fields, Values, Objects, and Smart Lists Feature Spec

```bash
ghl custom-fields list [--object contact|opportunity|company]
ghl custom-fields create --object <object> --name <name> --type <type> [--dry-run]
ghl custom-fields update <field-id> [...fields] [--dry-run]
ghl custom-fields delete <field-id> [--dry-run] --yes

ghl custom-values list
ghl custom-values create --name <name> --value <value> [--dry-run]
ghl custom-values update <value-id> --value <value> [--dry-run]
ghl custom-values delete <value-id> [--dry-run] --yes

ghl objects schemas list
ghl objects schemas create --from-file <path> [--dry-run]
ghl objects records search --object <key> [...filters]
ghl objects records create --object <key> --from-file <path> [--dry-run]
ghl objects records update --object <key> <record-id> --from-file <path> [--dry-run]
ghl objects records delete --object <key> <record-id> [--dry-run] --yes

ghl smart-lists list
ghl smart-lists get <smart-list-id>
ghl smart-lists contacts <smart-list-id> [--limit <n>]
ghl smart-lists duplicate <smart-list-id> --name <name> [--dry-run]
```

Requirements:

- Custom field and object schemas must be accepted from JSON files for
  reproducibility.
- Smart list contact reads must paginate.
- Destructive schema operations require `allow_destructive`.

## 55. Reporting, Reputation, and Revenue Feature Spec

```bash
ghl reports dashboard [--from <date>] [--to <date>]
ghl reports pipeline [--pipeline <id>] [--from <date>] [--to <date>]
ghl reports appointments [--from <date>] [--to <date>]
ghl reports calls [--from <date>] [--to <date>]
ghl reports attribution [--from <date>] [--to <date>]
ghl reports revenue [--from <date>] [--to <date>]

ghl reputation reviews list [--limit <n>]
ghl reputation reviews reply <review-id> --body <text>|--from-file <path> [--dry-run] [--yes]
ghl reputation requests send --contact <id> [--dry-run] [--yes]

ghl invoices list [--status <status>]
ghl invoices get <invoice-id>
ghl invoices create --from-file <path> [--dry-run]
ghl invoices send <invoice-id> [--dry-run] [--yes]
ghl estimates list
ghl estimates send <estimate-id> [--dry-run] [--yes]
ghl payments transactions list [--from <date>] [--to <date>]
ghl products list
ghl products create --from-file <path> [--dry-run]
```

Requirements:

- Payment and invoice send/create commands require `allow_payment_actions`.
- Review request sends require `allow_messaging`.
- Report commands should support date ranges and stable aggregation output.

## 56. Agency, Compliance, Phone, and AI Feature Spec

These commands are later-phase or client-triggered because they are higher-risk
or require specialized auth.

```bash
ghl snapshots list
ghl snapshots get <snapshot-id>
ghl snapshots push <snapshot-id> --locations <ids> [--dry-run] [--yes]
ghl snapshots share-link <snapshot-id> [--dry-run] [--yes]

ghl integrations pit list --scope company|location
ghl integrations pit create --name <name> --scopes <csv|@file> [--dry-run] [--yes]
ghl integrations pit delete <integration-id> [--dry-run] --yes

ghl domains list
ghl domains get <domain-id>
ghl domains verify <domain-id>

ghl a2p brand status
ghl a2p campaign status

ghl phone numbers list
ghl phone numbers search --area-code <code>
ghl phone numbers buy <number> [--dry-run] [--yes]
ghl phone numbers release <number-id> [--dry-run] --yes

ghl voice-ai agents list
ghl voice-ai agents get <agent-id>
ghl voice-ai call-logs list [--limit <n>]

ghl agent-studio agents list
ghl agent-studio agents get <agent-id>
ghl agent-studio agents deploy <agent-id> [--dry-run] [--yes]
```

Requirements:

- PIT create requires `allow_private_integration_token_create`.
- Snapshot share-link requires `allow_public_links`.
- Phone purchase/release requires `allow_phone_purchase`.
- A2P commands require Firebase auth when the endpoint requires it.
- Agent Studio deploy requires explicit confirmation and policy allowance.

## 57. Webhooks Feature Spec

Webhooks are high-risk because they connect GHL to external systems. They are in
scope for full coverage, but not MVP.

```bash
ghl webhooks list
ghl webhooks get <webhook-id>
ghl webhooks create --from-file <path> [--dry-run]
ghl webhooks update <webhook-id> --from-file <path> [--dry-run]
ghl webhooks delete <webhook-id> [--dry-run] --yes
ghl webhooks logs <webhook-id> [--limit <n>]
ghl webhooks retry <event-id> [--dry-run] --yes
```

Requirements:

- Create/update payloads require schema validation.
- Delete requires `allow_destructive`.
- Retry requires explicit confirmation and audit logging.
- Webhook URLs are treated as sensitive operational data in logs and fixtures.

## 58. OAuth, Marketplace, and Integration Feature Spec

Remote integrations are separate from local CLI credentials.

```bash
ghl integrations oauth apps list
ghl integrations oauth apps get <app-id>
ghl integrations oauth locations list <app-id>
ghl integrations marketplace installs list
ghl integrations marketplace installs delete <install-id> [--dry-run] --yes
ghl integrations api-keys list
ghl integrations api-keys create --name <name> [--dry-run] [--yes]
ghl integrations api-keys delete <key-id> [--dry-run] --yes
```

Requirements:

- API key and PIT creation require `allow_private_integration_token_create`.
- Deletes require `allow_destructive`.
- Created token values are secrets and may be shown only once when explicitly
  requested, then must be stored or discarded.
- Marketplace install deletion is high-risk and must include capability and
  dependency checks.

## 59. Users, Teams, and Roles Feature Spec

User and role reads support diagnostics and capability discovery. Mutating users
is later work.

```bash
ghl users list
ghl users get <user-id>
ghl users search --email <email>
ghl teams list
ghl roles list
ghl roles get <role-id>
```

Requirements:

- MVP may implement read-only user/role commands only.
- User emails and names are PII and follow export/privacy rules.
- Capability discovery should reuse users and roles data when available.


## 60. Raw Request Feature Spec

```bash
ghl raw request \
  --surface services|backend|firebase \
  --method GET|POST|PUT|PATCH|DELETE \
  --path <path> \
  [--query <json>] \
  [--body <json|@file>] \
  [--auth pit|session|firebase|cookie|oauth] \
  [--dry-run]
```

Requirements:

- Apply normal auth, redaction, retry, and rate-limit behavior.
- Refuse absolute URLs unless `--allow-absolute-url` is passed.
- Redact Authorization, Cookie, token fields, passwords, PITs, JWTs, OTPs, and
  message bodies from diagnostics.
- Dry-run prints method, surface, redacted path, selected auth class, and redacted
  body.
- Mutating raw requests require `--yes` unless `--dry-run` is used.
- Destructive raw requests require `allow_destructive`.

## 61. Smoke Test Feature Spec

```bash
ghl smoke run [--skip-writes] [--include-internal] [--include-messaging-dry-run]
```

Smoke output:

```json
{
  "ok": true,
  "profile": "default",
  "location_id": "location-id",
  "checks": [
    { "name": "auth.status", "status": "passed" },
    { "name": "locations.get", "status": "passed" },
    { "name": "contacts.search", "status": "passed", "count": 3 }
  ]
}
```

Requirements:

- Required checks are read-only.
- Optional write checks must use dry-run unless the user passes explicit flags.
- Output must not include contact names, phone numbers, emails, message text,
  invoice links, PIT tokens, or workflow bodies.
- Internal checks run only with `--include-internal`.

## 62. Command Metadata Feature Spec

```bash
ghl commands schema --format json
```

Metadata must include:

- Command group and command name.
- Description.
- Required auth class or classes.
- Required PIT scopes when known.
- Read/write/destructive classification.
- Policy flags required.
- Whether `--dry-run` is supported.
- Supported dry-run modes.
- Input schema keys.
- Output schema key and output schema version.
- Undo class.
- Endpoint key, surface, API confidence, and source references.
- Capability requirements.
- Offline availability.
- Required and optional arguments.

This mirrors the reference CLIs and gives agents a safe discovery surface.

## 63. Test Suite Spec

### 63.1 Unit tests

Unit coverage must include:

- Config path resolution.
- Profile schema serialization and migration.
- Credential reference creation and redaction.
- Auth response parsing.
- Refresh-token atomic rotation.
- Header construction per surface and auth class.
- Rate limiter behavior per location.
- Retry classification.
- Pagination normalization.
- Bulk input parsing and per-item result envelopes.
- Idempotency key reuse and request-hash mismatch behavior.
- Dependency preflight failures.
- Audit journal entry creation and redaction.
- Data classification and token-looking string redaction.
- Scope requirement metadata.
- API drift shape checks.
- Fixture capture deny rules.
- Signet credential reference parsing.
- Capability discovery result normalization.
- Messaging compliance pre-send blockers.
- Dry-run local versus validated behavior.
- Input schema validation and example generation.
- Undo plan generation for supported, partial, and unsupported operations.
- Job state persistence and wait behavior.
- Local lock acquisition, timeout, and stale-lock handling.
- Offline command allowlist enforcement.
- Confirmation phrase generation and non-interactive failure behavior.
- Endpoint manifest validation and endpoint coverage reports.
- Context precedence and ambiguous context failures.
- Query field selection, raw output, and unknown field behavior.
- Standard error registry completeness.
- Money and timezone normalization.
- Export privacy refusals and audit entries.
- Retention prune dry-runs and credential-preservation guarantees.
- Error envelope mapping.
- Dry-run redaction.
- Policy denial before network calls.
- Pipeline stage normalization rules.
- Workflow schema validation rules.

### 63.2 Mock HTTP integration tests

Mock tests must cover:

- PIT validation.
- Session login 3-step happy path.
- Session login failures for missing step-two token, invalid OTP, expired OTP,
  and rate-limited OTP email.
- Refresh success with rotated token.
- Refresh failure when refresh token is consumed.
- Contacts search/get/create/update/delete.
- Opportunity search/create/move/delete.
- Pipeline create/update/delete rules.
- Conversation reads and dry-run message send.
- Calendar events and free slots.
- Workflow get/update validation and version handling.
- Raw request redaction.
- Scope checks for available, missing, inferred, and unavailable scopes.
- API drift doctor checks for expected and changed shapes.
- Fixture capture with redacted and unsafe fixture cases.
- Audit journal writes for dry-run, success, and failure cases.
- Capability probes for available, blocked, and unknown feature states.
- Messaging compliance dry-runs and blocked real sends.
- Smoke seed and cleanup using marked resources only.
- Debug bundle redaction and unsafe bundle refusal.
- Webhook, integrations, and user/role read fixtures when those feature groups ship.

### 63.3 Fixture tests

Fixtures should live under `tests/fixtures/ghl/` and include redacted responses
from each implemented endpoint family.

Fixture rules:

- No real names, phone numbers, emails, tokens, links, or customer data.
- Preserve upstream shape, field naming, pagination metadata, and error bodies.
- Include at least one Cloudflare-style block response for PIT User-Agent tests.

### 63.4 Real-account smoke tests

Real smoke tests must be opt-in and read-only by default.

Requirements:

- Use environment variables or stored test profile.
- Print counts and statuses only.
- Never print private customer data.
- Skip unavailable auth classes rather than failing the entire smoke run unless
  the user requested that auth class.
- Do not send real SMS, email, invoices, review requests, workflow triggers, or
  phone purchases in default smoke.

## 64. Completion Gates

### 64.1 Command completion gate

A command is complete when:

- It has help text.
- It has a stable JSON output contract.
- It has unit tests.
- It has mock HTTP tests when it calls GHL.
- It maps errors into the shared error envelope.
- It declares coverage metadata, auth requirements, scope requirements, and
  policy flags in `commands schema`.
- It enforces auth class and policy before network mutation.
- It redacts secrets and sensitive bodies.
- It writes an audit entry for real mutations and sensitive dry-runs.
- It declares output schema version, endpoint manifest entries, standard error codes, and undo class where applicable.
- It supports local or validated dry-run according to the dry-run contract when it mutates state.
- It appears in `commands schema`.
- It is documented with at least one example.

### 64.2 Feature group completion gate

A feature group is complete when:

- All planned commands for that phase pass the command completion gate.
- Group-level docs explain auth requirements and safety behavior.
- At least one read-only smoke check exists.
- Pagination behavior is tested.
- Rate-limit behavior is tested where applicable.

### 64.3 MVP completion gate

The MVP is complete when:

- Auth, profiles, config, HTTP client, error envelopes, command metadata, and
  policy are implemented.
- Naming rules, API coverage metadata, pagination, idempotency, duplicate
  prevention, data classification, audit journal, scope verification, drift
  detection, fixture capture, Signet credential references, capability discovery,
  messaging compliance, dry-run modes, schema validation, undo metadata, jobs,
  locks, smoke cleanup, config lifecycle, offline mode, confirmation UX, network
  configuration, threat model, disclaimer requirements, endpoint manifest, context
  resolution, query shaping, support bundles, credential fallback warnings, token
  lifecycle rules, error registry, normalization, export privacy, and retention
  are implemented
  enough to support the MVP command groups.
- PIT and session auth both work.
- Contacts, conversations, opportunities, pipelines, calendars, workflow reads,
  raw requests, smoke run, and update check are implemented.
- `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and
  `cargo test --workspace` pass.
- Release build passes.
- README, SPEC, COMMANDS, CONFIG, INSTALL, NETWORK, SMOKE, ROADMAP, AUDIT,
  DATA-SAFETY, API-COVERAGE, SCHEMAS, CAPABILITIES, SECURITY, and agent setup
  skill docs exist.

## 65. Implementation Architecture

### 65.1 Workspace layout

```text
.
├── Cargo.toml
├── crates
│   ├── ghl
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── auth.rs
│   │       ├── client.rs
│   │       ├── config.rs
│   │       ├── credentials.rs
│   │       ├── error.rs
│   │       ├── models.rs
│   │       ├── policy.rs
│   │       ├── rate_limit.rs
│   │       ├── surfaces.rs
│   │       └── features
│   │           ├── contacts.rs
│   │           ├── conversations.rs
│   │           ├── opportunities.rs
│   │           ├── pipelines.rs
│   │           ├── calendars.rs
│   │           └── workflows.rs
│   └── ghl-cli
│       ├── Cargo.toml
│       └── src
│           ├── main.rs
│           ├── commands
│           └── output.rs
├── docs
│   ├── SPEC.md
│   ├── COMMANDS.md
│   ├── CONFIG.md
│   ├── INSTALL.md
│   ├── NETWORK.md
│   ├── SMOKE.md
│   └── ROADMAP.md
├── npm
│   ├── package.json
│   ├── install.js
│   ├── platform.js
│   └── run.js
├── skills
│   └── ghl-cli
│       └── SKILL.md
├── tests
│   └── fixtures
└── install.sh
```

### 65.2 Suggested Rust dependencies

- `clap` for CLI parsing.
- `serde`, `serde_json`, and `serde_yaml` for data handling.
- `tokio` for async runtime.
- `reqwest` with rustls for HTTP.
- `url` for safe path/query construction.
- `time` or `chrono` for timestamp parsing.
- `keyring` for OS credential storage.
- `directories` for config paths.
- `thiserror` for typed errors.
- `tracing` for redacted diagnostics.
- `wiremock` or equivalent for HTTP tests.

### 65.3 Client boundaries

The `ghl` library crate owns:

- Profile/config loading.
- Credential resolution.
- Auth refresh.
- API surface selection.
- Header construction.
- Rate limiting.
- Retry and cache behavior.
- Typed feature clients.
- Error mapping.

The `ghl-cli` crate owns:

- Argument parsing.
- Interactive prompts.
- Output formatting.
- Exit codes.
- Command metadata.
- CLI-specific validation.

## 66. Compatibility Spec

Target platforms:

| Platform | Target |
| --- | --- |
| Linux x64 glibc | `x86_64-unknown-linux-gnu` |
| Linux arm64 glibc | `aarch64-unknown-linux-gnu` |
| macOS arm64 | `aarch64-apple-darwin` |
| macOS x64 | `x86_64-apple-darwin` |
| Windows x64 | `x86_64-pc-windows-msvc` |

Compatibility requirements:

- Config schema migrations are additive and tested.
- Output contracts should version breaking shape changes.
- Commands may add fields without breaking compatibility.
- Renamed commands require aliases for at least one minor release once public.

## 67. Distribution and Installation Spec

Primary installation methods:

```bash
npm install -g ghl-cli
curl -fsSL https://raw.githubusercontent.com/<owner>/GHL-CLI/main/install.sh | sh
cargo install --git https://github.com/<owner>/GHL-CLI --locked
```

Release requirements:

- GitHub Release assets for supported targets.
- SHA-256 checksum per asset.
- npm package downloads the correct release binary and verifies checksum.
- npm package ships no vendored native binaries.
- Installer supports `GHL_CLI_INSTALL_VERSION`, `GHL_CLI_INSTALL_DIR`, and a
  release-base-url override for tests.
- Both `ghl-cli` and `ghl` binaries or shims work.
- `ghl update check` reports latest version and install method when detectable.

## 68. Supply-chain and Release Security Contract

The CLI stores CRM credentials and can mutate customer data. Release security is
part of the product.

### 68.1 Release artifact requirements

- GitHub Release archives include SHA-256 checksums.
- Installer refuses checksum mismatch.
- npm package never contains vendored native binaries.
- Release notes include version, git commit SHA, supported targets, and install
  commands.
- Release workflow should support signed artifacts with `cosign`, `minisign`, or
  an equivalent tool before a public v1.
- npm provenance should be enabled when practical.

### 68.2 Dependency checks

Release gates should include:

- Rust dependency audit, such as `cargo audit`.
- License and duplicate dependency policy, such as `cargo deny`.
- npm dependency audit for wrapper package.
- SBOM generation when practical.
- Dependency review for packages touching auth, crypto, HTTP, archive extraction,
  or installer behavior.

## 69. Shell Completion and Manpage Contract

Good CLI hygiene matters for humans and agents.

```bash
ghl completions bash
ghl completions zsh
ghl completions fish
ghl completions powershell
ghl man > ghl.1
```

Requirements:

- Completion generation must not require network access.
- Generated completions should include command aliases and global flags.
- Manpage generation should derive from the same command metadata as help text.

## 70. Terms and Acceptable-use Release Gate

Before public release, review current Go High Level, HighLevel, and LeadConnector
terms and platform rules.

Release requirements:

- Document which commands use undocumented APIs.
- Keep unofficial disclaimers visible.
- Do not position the CLI as official or endorsed.
- Do not ship commands whose primary purpose is bypassing access controls,
  evading platform limits, or hiding automation from GHL.
- Do not bypass user permissions. The CLI may automate allowed actions only.
- Revisit this gate before publishing package-manager releases or marketing the
  CLI publicly.


## 71. Documentation Requirements

Required docs before MVP:

- `README.md`: product summary, install, quickstart, safety model.
- `docs/SPEC.md`: this product contract.
- `docs/COMMANDS.md`: generated or maintained command reference.
- `docs/CONFIG.md`: config paths, profile schema, credential backend.
- `docs/INSTALL.md`: install methods and troubleshooting.
- `docs/NETWORK.md`: GHL surfaces, auth headers, rate limits, proxy/TLS notes.
- `docs/SMOKE.md`: safe real-account smoke instructions.
- `docs/ROADMAP.md`: phase map and future work.
- `docs/AUDIT.md`: audit journal format, retention, and export behavior.
- `docs/DATA-SAFETY.md`: data classes, redaction behavior, and fixture safety.
- `docs/API-COVERAGE.md`: generated or maintained coverage matrix.
- `docs/FEATURE-PARITY.md`: complete parity outline across all local GHL
  references and MCP-derived tool categories.
- `docs/SCHEMAS.md`: schema keys, examples, and file-input validation rules.
- `docs/CAPABILITIES.md`: permission, scope, feature, and command capability discovery.
- `docs/ENDPOINTS.md`: endpoint manifest shape, coverage process, and status values.
- `docs/ERRORS.md`: stable error-code registry.
- `docs/EXPORTS.md`: export privacy rules and examples.
- `docs/RELEASE-SECURITY.md`: checksums, signatures, audits, SBOMs, and provenance.
- `docs/SECURITY.md`: threat model, secret handling, redaction, and unofficial disclaimer.
- `skills/ghl-cli/SKILL.md`: agent setup and safe usage guide.

Docs must clearly state that internal/undocumented GHL endpoints can change and
that commands using those endpoints are implemented from local reverse-engineered
references.

## 72. Release Readiness Checklist

- [ ] `cargo fmt --all` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [ ] `cargo test --workspace` passes.
- [ ] `git diff --check` passes once the repo is initialized.
- [ ] Mock HTTP tests cover implemented endpoint families.
- [ ] Smoke run passes against a test profile without printing private data.
- [ ] Secrets do not appear in logs, snapshots, fixtures, or docs.
- [ ] Fixture scanner passes against all committed fixtures.
- [ ] Audit journal tests prove sensitive values are redacted.
- [ ] API coverage matrix covers every referenced endpoint family.
- [ ] API drift doctor can run safe-read checks without exposing customer data.
- [ ] Signet integration checks work when Signet is present and skip cleanly
      when it is absent.
- [ ] Capability checks distinguish auth failures from feature, plan, scope, and policy blocks.
- [ ] Messaging compliance checks refuse known DND/unsubscribed sends by default.
- [ ] Offline mode prevents network access before auth refresh.
- [ ] Config export/import/migrate round trips without leaking secrets.
- [ ] SECURITY and README include the unofficial product disclaimer.
- [ ] Endpoint manifest coverage passes for all MVP endpoints.
- [ ] Error registry documents every emitted stable error code.
- [ ] Debug bundle scanner refuses unsafe bundles.
- [ ] Release artifacts include checksums and installer checksum verification.
- [ ] Dependency audit, license policy, and npm wrapper audit pass.
- [ ] Public-release terms and acceptable-use gate has been reviewed.
- [ ] npm install smoke works from packed package.
- [ ] curl installer smoke works from local fixture.
- [ ] Release workflow creates archives and checksums.
- [ ] README quickstart works on a clean machine.
- [ ] Agent skill can install, authenticate, run status, and run read-only smoke.

## 73. Implementation Phasing

### 73.1 Phase 0: repository spine

- Initialize Rust workspace.
- Add `ghl` library crate and `ghl-cli` binary crate.
- Add config path resolution.
- Add JSON output helpers.
- Add error envelope and exit codes.
- Add command naming contract and initial command metadata registry.
- Add API coverage matrix scaffolding.
- Add output schema version metadata scaffolding.
- Add offline-mode command classification.
- Add unofficial disclaimer placeholders in README and package metadata.
- Add endpoint manifest and error registry scaffolding.
- Add shell completion and manpage generation hooks.
- Add CI for fmt, clippy, tests.
- Add README and docs skeleton.

### 73.2 Phase 1: auth, profiles, and HTTP client

- Implement profile schema.
- Implement credential backend.
- Implement PIT auth.
- Implement session login 3-step flow.
- Implement refresh rotation.
- Implement auth status.
- Implement client surfaces, headers, rate limiting, retry, and cache.
- Implement `raw request` dry-run and read-only GET.
- Implement data classification and redaction helpers.
- Implement Signet credential reference parsing and `ghl signet doctor`.
- Implement initial scope requirement metadata.
- Implement config export/import/migrate.
- Implement local locks for config, credentials, audit, and idempotency state.
- Implement schema registry and validation for MVP file inputs.
- Implement context resolution and ambiguous-context failures.
- Implement credential fallback warnings and token lifecycle safeguards.
- Implement standard error registry commands.

### 73.3 Phase 2: CRM core

- Implement locations.
- Implement contact search/get.
- Implement broader contact writes, tags, tasks, notes, timeline, and export.
- Implement conversation search/get/messages.
- Implement broader conversation message reads, recordings, transcriptions, and attachment operations.
- Implement messaging dry-run, then guarded real send.
- Implement opportunities.
- Implement pipelines.
- Implement calendars and appointments.
- Implement smoke run.
- Implement pagination normalization for all MVP list commands.
- Implement audit journal for MVP write commands and sensitive dry-runs.
- Implement idempotency and duplicate-prevention preflights for contacts,
  opportunities, appointments, and pipelines.
- Implement fixture capture and fixture scanning for MVP endpoint families.
- Implement `doctor api` and `doctor endpoint` for MVP safe-read endpoints.
- Implement capabilities commands for MVP command groups.
- Implement messaging compliance checks for SMS/email dry-runs and real sends.
- Implement dry-run local/validated modes.
- Implement undo metadata for MVP write commands and undo apply for simple reversible actions.
- Implement local jobs for bulk operations.
- Implement smoke seed and cleanup marker rules.
- Implement endpoint manifest entries for MVP endpoints.
- Implement common query, field selection, raw/include-raw output for MVP reads.
- Implement debug bundle generation and scanning.
- Implement money/timezone normalization for MVP opportunity, calendar, and report-shaped outputs.
- Implement contacts/opportunities export privacy rules if export commands ship in MVP.
- Implement retention status and dry-run prune for local jobs/idempotency records.

### 73.4 Phase 3: workflows and internal APIs

- Implement workflow list and get.
- Implement workflow validate.
- Implement guarded status update and trigger.
- Implement workflow create/update/publish/delete after fixture coverage.
- Implement custom fields and custom values.
- Implement media, forms, surveys, templates, and trigger links as needed.

### 73.5 Phase 4: revenue, reporting, and reputation

- Implement reporting commands.
- Implement reputation reads and guarded replies/requests.
- Implement invoices, estimates, payments, products, and orders with strict
  policy gates.

### 73.6 Phase 5: agency and advanced surfaces

- Implement snapshots.
- Implement PIT management.
- Implement domains and A2P.
- Implement phone and Voice AI.
- Implement Agent Studio.
- Implement webhooks.
- Implement OAuth, marketplace, and remote integration commands.
- Implement users, teams, and roles reads.
- Implement custom objects, associations, marketplace, and SaaS.

### 73.7 Phase 6: distribution and agent experience

- Add release workflow.
- Add npm wrapper.
- Add curl installer.
- Add checksums and installer tests.
- Add agent skill.
- Add final docs and smoke instructions.

## 74. Open Questions for Future Versions

1. Should `ghl` be the npm package name, or should the package publish as
   `ghl-cli` to avoid namespace ambiguity?
2. Should OAuth location-token auth be implemented in the MVP or deferred until
   PIT and session auth are stable?
3. Which endpoint families should receive generated command scaffolds from local
   reference tool definitions, and which should remain curated manually?
4. Should the CLI include NDJSON streaming for large contact and reporting reads
   in v0.1, or defer until pagination is stable?
5. Should real sends require both `allow_messaging` and a second per-command
   `--yes`, or should profile policy be enough for trusted automation profiles?
6. How should account-specific custom field schemas be represented in reusable
   fixtures without leaking business data?
7. Should workflow visual graph rendering be a future command that exports
   Mermaid or FigJam-compatible diagrams?
