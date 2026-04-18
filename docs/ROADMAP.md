# Roadmap

## Phase 0

Repository spine, local command metadata, stable JSON envelopes, config path discovery, error registry, endpoint manifest scaffold, completions, docs, and CI.

## Phase 1

Profiles, local credential backend, local PIT storage, live PIT validation, HTTP surfaces, redaction helpers, and guarded raw GET are started. Remaining Phase 1 work: session login, refresh rotation, Signet credential references, schema registry, Signet credential references, schema registry, rate limiting, retries, and caching.

## Phase 2

CRM core: location get/list/search, contact list/search/get, conversation search/get/messages, pipeline list/get, opportunity search/get, calendar list/get/events/free-slots, user/team-member list/get/search, profile company context, required location context, and read-only smoke run are started. Remaining Phase 2 work: guarded messaging, fixtures, and extending the implemented audit/idempotency write pattern beyond appointments. Capability checks, local/API doctor reports, endpoint diagnostics, redacted JSON support bundles, audit journal commands, idempotency cache commands, and guarded appointment writes and notes are started.

Later phases follow `docs/SPEC.md`.
