# Roadmap

## Phase 0

Repository spine, local command metadata, stable JSON envelopes, config path discovery, error registry, endpoint manifest scaffold, completions, docs, and CI.

## Phase 1

Profiles, local credential backend, local PIT storage, live PIT validation, HTTP surfaces, redaction helpers, and guarded raw GET are started. Remaining Phase 1 work: session login, refresh rotation, Signet credential references, schema registry, Signet credential references, schema registry, rate limiting, retries, and caching.

## Phase 2

CRM core: location get/list/search, contact list/search/get plus guarded contact create/update, conversation search/get/messages, pipeline list/get, opportunity search/get plus guarded opportunity create/update, calendar list/get/events/free-slots, user/team-member list/get/search, profile company context, required location context, and read-only smoke run are started. Remaining Phase 2 work: guarded messaging, fixtures, broader opportunity subcommands, and broader contact subresources such as tags, tasks, notes, and timeline. Capability checks, local/API doctor reports, endpoint diagnostics, redacted JSON support bundles, audit journal commands, idempotency cache commands, and guarded contact, opportunity, and appointment writes/notes are started.

Later phases follow `docs/SPEC.md`.
