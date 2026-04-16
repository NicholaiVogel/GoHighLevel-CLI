# Smoke Tests

Local smoke is still network-free:

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
```

Real-account smoke should start with `ghl auth pit validate` against a dedicated test location. It performs `GET /locations/{location_id}` and does not print the response body. After that passes, use `ghl locations get <location-id>`, `ghl locations list`, `ghl locations search <email>`, `ghl contacts search --email <known-test-email>`, `ghl contacts get <known-test-contact-id>`, `ghl conversations search --contact <known-test-contact-id>`, `ghl conversations messages <known-test-conversation-id>`, `ghl pipelines list`, and `ghl opportunities search --contact <known-test-contact-id>` as the first typed read-only CRM smokes.
