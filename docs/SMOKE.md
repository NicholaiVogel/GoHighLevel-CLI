# Smoke Tests

Local smoke is still network-free:

```bash
ghl commands schema
ghl config path
ghl errors list
ghl endpoints coverage
ghl completions bash >/tmp/ghl.bash
printf 'pit-test-token\n' | ghl --config-dir /tmp/ghl-cli-smoke auth pit add --token-stdin --location loc_test
ghl --config-dir /tmp/ghl-cli-smoke auth status
ghl --config-dir /tmp/ghl-cli-smoke auth pit list-local
ghl --config-dir /tmp/ghl-cli-smoke raw request --surface services --method get --path /locations/loc_test --dry-run=local
ghl --config-dir /tmp/ghl-cli-smoke locations get loc_test --dry-run=local
```

Real-account smoke should start with `ghl auth pit validate` against a dedicated test location. It performs `GET /locations/{location_id}` and does not print the response body. After that passes, `ghl locations get <location-id>` is the first typed read-only CRM smoke.
