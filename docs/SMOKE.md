# Smoke Tests

Local smoke is network-free and safe to run without credentials:

```bash
ghl commands schema
ghl config path
ghl errors list
ghl endpoints coverage
ghl completions bash >/tmp/ghl.bash
printf 'pit-test-token\n' | ghl --config-dir /tmp/ghl-cli-smoke auth pit add --token-stdin --location loc_test --company company_test
ghl --config-dir /tmp/ghl-cli-smoke auth status
ghl --config-dir /tmp/ghl-cli-smoke auth pit list-local
ghl --config-dir /tmp/ghl-cli-smoke raw request --surface services --method get --path /locations/loc_test --dry-run=local
ghl --config-dir /tmp/ghl-cli-smoke locations get loc_test --dry-run=local
ghl --config-dir /tmp/ghl-cli-smoke locations list --dry-run=local
ghl --config-dir /tmp/ghl-cli-smoke locations search test@example.com --dry-run=local
ghl --config-dir /tmp/ghl-cli-smoke contacts list --limit 5 --dry-run=local
ghl --config-dir /tmp/ghl-cli-smoke contacts search "Test" --dry-run=local
ghl --config-dir /tmp/ghl-cli-smoke contacts search --email test@example.com --dry-run=local
ghl --config-dir /tmp/ghl-cli-smoke contacts get contact_test --dry-run=local
ghl --config-dir /tmp/ghl-cli-smoke conversations search --dry-run=local
ghl --config-dir /tmp/ghl-cli-smoke conversations search --contact contact_test --status unread --dry-run=local
ghl --config-dir /tmp/ghl-cli-smoke conversations get conv_test --dry-run=local
ghl --config-dir /tmp/ghl-cli-smoke conversations messages conv_test --dry-run=local
ghl --config-dir /tmp/ghl-cli-smoke pipelines list --dry-run=local
ghl --config-dir /tmp/ghl-cli-smoke pipelines get pipe_test --dry-run=local
ghl --config-dir /tmp/ghl-cli-smoke opportunities search --dry-run=local
ghl --config-dir /tmp/ghl-cli-smoke opportunities search --pipeline pipe_test --status open --dry-run=local
ghl --config-dir /tmp/ghl-cli-smoke opportunities get opp_test --dry-run=local
ghl --config-dir /tmp/ghl-cli-smoke --location loc_test calendars list --dry-run=local
ghl --config-dir /tmp/ghl-cli-smoke --location loc_test calendars events --calendar cal_test --date 2026-04-17 --dry-run=local
ghl --config-dir /tmp/ghl-cli-smoke --location loc_test calendars free-slots --calendar cal_test --date 2026-04-17 --dry-run=local
ghl --config-dir /tmp/ghl-cli-smoke --location loc_test users list --dry-run=local
ghl --config-dir /tmp/ghl-cli-smoke --location loc_test users search --email test@example.com --dry-run=local
ghl --config-dir /tmp/ghl-cli-smoke --location loc_test --company company_test smoke run --dry-run=local --pretty
```

`ghl smoke run` is the preferred first real-account validation command after
`ghl auth pit validate`. It performs only read-only checks by default and prints
status, HTTP status, counts, and error codes. It does not print contact names,
phone numbers, emails, message text, opportunity notes, tokens, or response
bodies.

Run the default live smoke against a dedicated test location:

```bash
ghl --profile default auth pit validate --pretty
ghl --profile default smoke run --pretty
```

The required live checks are:

- `auth.status`, local PIT availability
- `context.location`, resolved location context
- `locations.get`, selected location is readable
- `contacts.list`, contact search endpoint is readable without printing contacts
- `pipelines.list`, sales pipelines are readable
- `conversations.search`, conversation search is readable
- `opportunities.search`, opportunity search is readable
- `calendars.list`, calendar list is readable
- `users.list`, team-member list is readable without printing user bodies

`locations.list` runs when company context is available. Pass `--company` or set
a profile company id with `ghl profiles set-default-company <profile> <company-id>`.

Optional read checks run only when you pass known test IDs or search filters:

```bash
ghl --profile default smoke run \
  --contact-email test@example.com \
  --contact-id <known-test-contact-id> \
  --conversation-id <known-test-conversation-id> \
  --pipeline-id <known-test-pipeline-id> \
  --opportunity-id <known-test-opportunity-id> \
  --calendar-id <known-test-calendar-id> \
  --calendar-date 2026-04-17 \
  --user-id <known-test-user-id> \
  --user-email test@example.com \
  --pretty
```

Use `--skip-optional` to run only the required checks plus company-aware
`locations.list`.
