# Command Reference

Status: Phase 1 auth/profile and HTTP spine, read-only CRM/team commands, guarded appointment create/update/cancel/notes, smoke diagnostics, and local audit/idempotency commands.

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
- `ghl doctor`
- `ghl doctor api [--limit <n>]`
- `ghl doctor endpoint <endpoint-key>`
- `ghl doctor bundle --out <path> --redacted`
- `ghl capabilities`
- `ghl capabilities list`
- `ghl capabilities check <capability>`
- `ghl capabilities command <command-key>`
- `ghl audit list [--from <datetime>] [--to <datetime>] [--action <name>] [--resource <id>] [--limit <n>]`
- `ghl audit show <entry-id>`
- `ghl audit export [--from <datetime>] [--to <datetime>] [--action <name>] [--resource <id>] [--out <path>]`
- `ghl idempotency list`
- `ghl idempotency show <key>`
- `ghl idempotency clear <key> --yes`
- `ghl raw request --surface services|backend --method get --path <path> [--include-body]`
- `ghl locations get <location-id>`
- `ghl locations list [--company <company-id>] [--skip <n>] [--limit <n>] [--order asc|desc]`
- `ghl locations search <email> [--company <company-id>] [--skip <n>] [--limit <n>] [--order asc|desc]`
- `ghl contacts list [--limit <n>] [--start-after-id <id>] [--start-after <cursor>]`
- `ghl contacts search [<query>] [--email <email>] [--phone <phone>] [--limit <n>] [--start-after-id <id>] [--start-after <cursor>]`
- `ghl contacts get <contact-id>`
- `ghl conversations search [--contact <contact-id>] [--query <query>] [--status all|read|unread|starred|recents] [--limit <n>] [--assigned-to <user-id>] [--last-message-type <type>] [--start-after-date <epoch-ms>]`
- `ghl conversations get <conversation-id>`
- `ghl conversations messages <conversation-id> [--limit <n>] [--last-message-id <id>] [--message-type <type>]`
- `ghl pipelines list`
- `ghl pipelines get <pipeline-id>`
- `ghl opportunities search [--query <query>] [--pipeline <pipeline-id>] [--stage <stage-id>] [--contact <contact-id>] [--status open|won|lost|abandoned|all] [--assigned-to <user-id>] [--limit <n>] [--page <n>] [--start-after-id <id>] [--start-after <cursor>]`
- `ghl opportunities get <opportunity-id>`
- `ghl calendars list [--group <id>] [--show-drafted true|false]`
- `ghl calendars get <calendar-id>`
- `ghl calendars events [--calendar <id>] [--date <date>] [--from <datetime>] [--to <datetime>]`
- `ghl calendars free-slots --calendar <id> --date <date> [--timezone <timezone>]`
- `ghl appointments create --calendar <id> --contact <id> --starts-at <datetime> --ends-at <datetime> [--title <text>] [--status new|confirmed] [--assigned-user <id>] [--meeting-location-type <type>] [--timezone <tz>] [--idempotency-key <key>] [--skip-free-slot-check]`
- `ghl appointments update <appointment-id> [--title <text>] [--status new|confirmed|cancelled|showed|no-show|invalid] [--starts-at <datetime>] [--ends-at <datetime>] [--notify|--no-notify] [--idempotency-key <key>]`
- `ghl appointments cancel <appointment-id> [--idempotency-key <key>]`
- `ghl appointments notes list <appointment-id> [--limit <n>] [--offset <n>]`
- `ghl appointments notes create <appointment-id> --body <text>|--from-file <path> [--user <id>] [--idempotency-key <key>]`
- `ghl appointments notes update <appointment-id> <note-id> --body <text>|--from-file <path> [--user <id>] [--idempotency-key <key>]`
- `ghl appointments notes delete <appointment-id> <note-id> [--idempotency-key <key>]``
- `ghl users list [--skip <n>] [--limit <n>]`
- `ghl users get <user-id>`
- `ghl users search --email <email>`
- `ghl users search --query <query> [--company <company-id>] [--skip <n>] [--limit <n>]`
- `ghl teams list [--skip <n>] [--limit <n>]`
- `ghl smoke run [--limit <n>] [--skip-optional] [--contact-query <query>] [--contact-email <email>] [--contact-phone <phone>] [--contact-id <id>] [--conversation-id <id>] [--pipeline-id <id>] [--opportunity-id <id>] [--calendar-id <id>] [--calendar-date <date>] [--calendar-timezone <timezone>] [--user-id <id>] [--user-email <email>] [--user-query <query>]`
- `ghl completions bash|zsh|fish|powershell`
- `ghl man`

Audit and idempotency commands are local-only. `audit list/show/export` reads the redacted JSONL journal under the resolved audit data directory. `idempotency list/show/clear` manages the local idempotency cache that future write commands will use to prevent accidental duplicate creates. `idempotency clear` requires `--yes` or global `--yes`.

Network support is deliberately narrow: PIT validation, raw GET, read-only location get/list/search, contact list/search/get, conversation search/get/messages, pipeline list/get, opportunity search/get, calendar list/get/events/free-slots, guarded appointment create/update/cancel/notes, user/team-member list/get/search, `doctor api`, and the read-only smoke runner only. Use `--dry-run=local` to preview network commands without credentials or network access. CRM commands require resolved location context from `--location` or the active profile. PIT tokens, message bodies, calendar event bodies, user/team-member bodies, opportunity notes, and smoke-run customer data are redacted from normal output.


## Diagnostics

`ghl doctor` is local-only and reports config path health, auth availability, command count, and endpoint coverage. `ghl doctor api` runs the same safe required read checks as smoke mode and prints only statuses, counts, and HTTP codes. `ghl doctor endpoint <endpoint-key>` explains the bundled endpoint record and mapped commands. `ghl doctor bundle --out <path> --redacted` writes a JSON support bundle that excludes credential stores and raw customer bodies. The redaction flag is mandatory.

`ghl capabilities` lists inferred command availability for the selected profile. `ghl capabilities check <capability>` accepts either an implemented command key, such as `contacts.list`, or a planned capability name, such as `contacts.write`, and explains whether local auth, context, or profile policy blocks it.

## Audit and idempotency

`ghl audit list` accepts RFC3339 datetimes or Unix milliseconds for `--from` and `--to`, plus action/resource filters. `ghl audit export` writes owner-only JSON when `--out` is provided or returns matching entries in the normal JSON envelope when omitted.

`ghl idempotency list` and `ghl idempotency show` inspect the local duplicate-prevention cache. `ghl idempotency clear <key> --yes` removes one key by raw key or scoped key. Reusing the same idempotency key with a different redacted request hash is rejected by the core library.

## Appointments

`ghl appointments create`, `ghl appointments update`, `ghl appointments cancel`, and appointment note create/update/delete are guarded write commands. With `--dry-run=local`, they validate the local request shape, write a redacted sensitive dry-run audit entry, and perform no network mutation. Real writes require global `--yes`, profile policy `allow_destructive=true`, and `--idempotency-key <key>`. Create also requires a successful free-slot preflight unless `--skip-free-slot-check` is passed. Real writes record audit and idempotency entries. `appointments notes list` is read-only and redacts note bodies in normal output.
