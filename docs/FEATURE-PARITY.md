# GHL CLI Feature Parity Outline

Status: Draft parity plan  
Last updated: 2026-04-13  
Primary spec: `docs/SPEC.md`  
Machine-readable source extract: `data/reference-tools.json`

## Purpose

This document defines what is needed for `ghl-cli` to reach feature parity with the local Go High Level references. The spec now has the safety architecture. This file is the parity punch list: every referenced API family, every MCP tool module, and every artifact required to prove coverage.

Feature parity means every referenced API capability has a CLI story. It does not mean every capability is MVP. It means every capability is either implemented, explicitly planned with command names and contracts, intentionally deferred with a reason, or declared out of scope.

MCP serving remains out of scope. The MCP reference is used as an API catalog only.

## Parity Definition

A reference capability reaches parity when all of the following exist:

- Endpoint manifest entry with `endpoint_key`, surface, method, path template, auth classes, source refs, risk, status, confidence, and required response fields.
- CLI command mapping with canonical namespace and command key.
- Command metadata entry from `ghl commands schema`.
- Required auth classes and PIT scope metadata.
- Input schema for every file-driven or structured write command.
- Output schema key and output schema version.
- Pagination behavior when the reference endpoint lists resources.
- Bulk behavior when the operation can apply to multiple resources.
- Policy gates for destructive, messaging, payment, workflow, public-link, phone, token-creation, or compliance-sensitive actions.
- Audit journal behavior and undo class.
- Fixture coverage with redacted upstream response shapes.
- Mock HTTP tests for success, validation failure, auth failure, permission failure, rate limit, and upstream shape drift.
- Smoke test classification: safe read, dry-run write, dangerous write, or unavailable.
- Documentation examples and warnings.

## Required Parity Artifacts

| Artifact | Purpose | Required before parity |
| --- | --- | --- |
| `data/reference-tools.json` | Extracted MCP reference tool list. | Already created from 45 modules / 562 tools. |
| `data/endpoints.json` | Machine-readable endpoint manifest for every command-backed endpoint. | Yes. |
| `docs/API-COVERAGE.md` | Human-readable coverage table by reference family and phase. | Yes. |
| `docs/ENDPOINTS.md` | Endpoint manifest schema and coverage workflow. | Yes. |
| `docs/COMMANDS.md` | CLI command reference generated from command metadata. | Yes. |
| `docs/SCHEMAS.md` | Input and output schema registry. | Yes. |
| `docs/ERRORS.md` | Stable error-code registry. | Yes. |
| `tests/fixtures/ghl/**` | Redacted fixture corpus. | Yes. |
| `tests/mock/**` | Mock HTTP behavior tests. | Yes. |
| `skills/ghl-cli/SKILL.md` | Agent install/setup/safe-use skill. | Yes. |

## Coverage Summary

The MCP reference contains **45 tool modules** and **562 tool definitions**. The API bible contains **25 major sections**. Full parity requires both sources to be represented in the endpoint manifest and command metadata.

| Module | Tools | CLI namespace | Phase | Status | Risk |
| --- | ---: | --- | --- | --- | --- |
| `affiliates` | 17 | `affiliates` | 5 | planned | high |
| `agent-studio` | 8 | `agent-studio` | 5 | research | high |
| `association` | 10 | `associations` | 5 | planned | medium |
| `blog` | 7 | `blogs` | 5 | planned | medium |
| `businesses` | 5 | `businesses` | 5 | planned | medium |
| `calendar` | 39 | `calendars, appointments` | 2 | mvp | high |
| `campaigns` | 12 | `campaigns` | 5 | planned | high |
| `companies` | 5 | `companies` | 5 | planned | medium |
| `contact` | 31 | `contacts` | 2 | mvp | high |
| `conversation` | 20 | `conversations, messages` | 2 | mvp | high |
| `courses` | 32 | `courses` | 5 | planned | medium |
| `custom-field-v2` | 8 | `custom-fields` | 3 | planned | medium |
| `custom-menus` | 5 | `custom-menus` | 5 | planned | medium |
| `email-isv` | 9 | `email domains` | 5 | planned | high |
| `email` | 5 | `email, templates email` | 3 | planned | high |
| `forms` | 4 | `forms` | 3 | planned | medium |
| `funnels` | 8 | `funnels` | 3 | planned | medium |
| `invoices` | 18 | `invoices, estimates` | 4 | planned | high |
| `links` | 6 | `links` | 3 | planned | high |
| `location` | 28 | `locations` | 2 | mvp | high |
| `marketplace` | 7 | `integrations marketplace` | 5 | planned | high |
| `media` | 7 | `media` | 3 | planned | medium |
| `oauth` | 10 | `integrations oauth, auth oauth` | 5 | planned | high |
| `object` | 9 | `objects` | 5 | planned | medium |
| `opportunity` | 10 | `opportunities, pipelines` | 2 | mvp | high |
| `payments` | 22 | `payments, orders, coupons` | 4 | planned | high |
| `phone-system` | 15 | `phone` | 5 | planned | high |
| `phone` | 20 | `phone` | 5 | planned | high |
| `products` | 11 | `products` | 4 | planned | medium |
| `proposals` | 4 | `proposals` | 4 | planned | high |
| `reporting` | 12 | `reports` | 4 | planned | low |
| `reputation` | 15 | `reputation` | 4 | planned | high |
| `saas` | 12 | `saas` | 5 | planned | high |
| `smartlists` | 8 | `smart-lists` | 3 | planned | medium |
| `snapshots` | 7 | `snapshots` | 5 | planned | high |
| `social-media` | 19 | `social` | 5 | planned | high |
| `store` | 18 | `store, shipping` | 4 | planned | medium |
| `survey` | 9 | `surveys` | 3 | planned | medium |
| `templates` | 18 | `templates, snippets` | 3 | planned | medium |
| `triggers` | 11 | `triggers` | 5 | planned | high |
| `users` | 7 | `users, teams, roles` | 5 | planned | medium |
| `voice-ai` | 11 | `voice-ai` | 5 | planned | high |
| `webhooks` | 9 | `webhooks` | 5 | planned | high |
| `workflow-builder` | 7 | `workflows` | 3 | planned | high |
| `workflow` | 7 | `workflows` | 3 | planned | high |

## Work Needed for Full Parity

### 1. Generate the endpoint manifest

Build `data/endpoints.json` from the API bible and MCP tool definitions. Each record needs:

- `endpoint_key`
- `source tool name(s)`
- `surface` (`services`, `backend`, `firebase`, or external)
- `HTTP method`
- `path template`
- `required auth class(es)`
- `required scopes`
- `CLI command key(s)`
- `request schema key`
- `response schema key`
- `pagination metadata`
- `risk class`
- `phase/status`
- `fixture path`
- `drift-check requirements`

### 2. Normalize command namespaces

Use `docs/SPEC.md` section 8 as the canonical naming contract. The biggest rule is that local credential commands stay under `auth`, while remote integration resources stay under `integrations`. No top-level `ghl pit` namespace.

### 3. Add command metadata for every mapped capability

Every command must appear in `ghl commands schema` with auth, scope, policy, schema, output, undo, endpoint, capability, and offline metadata.

### 4. Build schemas before risky writes

For create/update/send/publish/push/buy/retry commands, define input schemas before implementation. For large payload areas like workflows, webhooks, snapshots, objects, courses, invoices, products, and social posts, schema validation is a blocker.

### 5. Create fixture coverage

Every parity command needs redacted fixtures for success and common failure shapes. Internal/undocumented endpoints need drift-check fixtures too.

### 6. Fill feature specs for broad grouped surfaces

The main spec now covers every family, but some are broad future specs. To reach full command parity, expand these into command-by-command contracts as implementation approaches: affiliates, blogs, businesses, campaigns, companies, courses, custom menus, email domains, store/shipping, products, payments, proposals, SaaS, associations, custom objects, phone, triggers, snapshots, social, users/roles, webhooks, and integrations.

## `affiliates` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/affiliates-tools.ts`
- Source tool count: 17
- CLI namespace: `affiliates`
- Phase: 5
- Status: planned
- Risk: high
- Scope: Affiliate campaigns, affiliates, commissions, payouts, referrals

### Source tools

- `get_affiliate_campaigns`
- `get_affiliate_campaign`
- `create_affiliate_campaign`
- `update_affiliate_campaign`
- `delete_affiliate_campaign`
- `get_affiliates`
- `get_affiliate`
- `create_affiliate`
- `update_affiliate`
- `approve_affiliate`
- `reject_affiliate`
- `delete_affiliate`
- `get_affiliate_commissions`
- `get_affiliate_stats`
- `create_payout`
- `get_payouts`
- `get_referrals`

### Required CLI command families

- `ghl affiliates campaigns list|get|create|update|delete`
- `ghl affiliates list|get|create|update|approve|reject|delete`
- `ghl affiliates commissions list`
- `ghl affiliates stats`
- `ghl affiliates payouts create|list`
- `ghl affiliates referrals list`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.
- Explicit profile policy gates for high-risk writes.
- Human confirmation or `--yes` behavior for real execution.
- Undo or compensation class for every write.

## `agent-studio` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/agent-studio-tools.ts`
- Source tool count: 8
- CLI namespace: `agent-studio`
- Phase: 5
- Status: research
- Risk: high
- Scope: GHL AI agent lifecycle, versions, deploy

### Source tools

- `ghl_create_agent`
- `ghl_list_agents`
- `ghl_get_agent`
- `ghl_update_agent`
- `ghl_delete_agent`
- `ghl_list_agent_versions`
- `ghl_update_agent_version`
- `ghl_deploy_agent`

### Required CLI command families

- `ghl agent-studio agents create|list|get|update|delete|deploy`
- `ghl agent-studio versions list|update`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.
- Explicit profile policy gates for high-risk writes.
- Human confirmation or `--yes` behavior for real execution.
- Undo or compensation class for every write.

## `association` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/association-tools.ts`
- Source tool count: 10
- CLI namespace: `associations`
- Phase: 5
- Status: planned
- Risk: medium
- Scope: Custom object association schemas and relations

### Source tools

- `ghl_get_all_associations`
- `ghl_create_association`
- `ghl_get_association_by_id`
- `ghl_update_association`
- `ghl_delete_association`
- `ghl_get_association_by_key`
- `ghl_get_association_by_object_key`
- `ghl_create_relation`
- `ghl_get_relations_by_record`
- `ghl_delete_relation`

### Required CLI command families

- `ghl associations list|get|create|update|delete`
- `ghl associations by-key`
- `ghl associations relations create|list|delete`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.

## `blog` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/blog-tools.ts`
- Source tool count: 7
- CLI namespace: `blogs`
- Phase: 5
- Status: planned
- Risk: medium
- Scope: Blog posts, sites, authors, categories, slug checks

### Source tools

- `create_blog_post`
- `update_blog_post`
- `get_blog_posts`
- `get_blog_sites`
- `get_blog_authors`
- `get_blog_categories`
- `check_url_slug`

### Required CLI command families

- `ghl blogs posts list|get|create|update`
- `ghl blogs sites list`
- `ghl blogs authors list`
- `ghl blogs categories list`
- `ghl blogs slug check`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.

## `businesses` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/businesses-tools.ts`
- Source tool count: 5
- CLI namespace: `businesses`
- Phase: 5
- Status: planned
- Risk: medium
- Scope: Business records

### Source tools

- `get_businesses`
- `get_business`
- `create_business`
- `update_business`
- `delete_business`

### Required CLI command families

- `ghl businesses list|get|create|update|delete`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.

## `calendar` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/calendar-tools.ts`
- Source tool count: 39
- CLI namespace: `calendars, appointments`
- Phase: 2
- Status: mvp
- Risk: high
- Scope: Calendar groups, calendars, events, slots, appointments, resources, notifications

### Source tools

- `get_calendar_groups`
- `get_calendars`
- `create_calendar`
- `get_calendar`
- `update_calendar`
- `delete_calendar`
- `get_calendar_events`
- `get_free_slots`
- `create_appointment`
- `get_appointment`
- `update_appointment`
- `delete_appointment`
- `create_block_slot`
- `update_block_slot`
- `create_calendar_group`
- `validate_group_slug`
- `update_calendar_group`
- `delete_calendar_group`
- `disable_calendar_group`
- `get_appointment_notes`
- `create_appointment_note`
- `update_appointment_note`
- `delete_appointment_note`
- `get_calendar_resources_equipments`
- `create_calendar_resource_equipment`
- `get_calendar_resource_equipment`
- `update_calendar_resource_equipment`
- `delete_calendar_resource_equipment`
- `get_calendar_resources_rooms`
- `create_calendar_resource_room`
- `get_calendar_resource_room`
- `update_calendar_resource_room`
- `delete_calendar_resource_room`
- `get_calendar_notifications`
- `create_calendar_notifications`
- `get_calendar_notification`
- `update_calendar_notification`
- `delete_calendar_notification`
- `get_blocked_slots`

### Required CLI command families

- `ghl calendars groups list|create|update|delete|disable|validate-slug`
- `ghl calendars list|get|create|update|delete`
- `ghl calendars events list`
- `ghl calendars free-slots`
- `ghl appointments create|get|update|delete`
- `ghl appointments notes list|create|update|delete`
- `ghl calendars resources equipment list|create|get|update|delete`
- `ghl calendars resources rooms list|create|get|update|delete`
- `ghl calendars notifications list|create|get|update|delete`
- `ghl calendars blocked-slots list|create|update`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.
- Explicit profile policy gates for high-risk writes.
- Human confirmation or `--yes` behavior for real execution.
- Undo or compensation class for every write.
- MVP smoke classification and safe read smoke check.

## `campaigns` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/campaigns-tools.ts`
- Source tool count: 12
- CLI namespace: `campaigns`
- Phase: 5
- Status: planned
- Risk: high
- Scope: Campaign lifecycle, recipients, stats, scheduled messages

### Source tools

- `get_campaigns`
- `get_campaign`
- `create_campaign`
- `update_campaign`
- `delete_campaign`
- `start_campaign`
- `pause_campaign`
- `resume_campaign`
- `get_campaign_stats`
- `get_campaign_recipients`
- `get_scheduled_messages`
- `cancel_scheduled_campaign_message`

### Required CLI command families

- `ghl campaigns list|get|create|update|delete|start|pause|resume`
- `ghl campaigns stats`
- `ghl campaigns recipients list`
- `ghl campaigns scheduled-messages list|cancel`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.
- Explicit profile policy gates for high-risk writes.
- Human confirmation or `--yes` behavior for real execution.
- Undo or compensation class for every write.

## `companies` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/companies-tools.ts`
- Source tool count: 5
- CLI namespace: `companies`
- Phase: 5
- Status: planned
- Risk: medium
- Scope: Company records

### Source tools

- `get_companies`
- `get_company`
- `create_company`
- `update_company`
- `delete_company`

### Required CLI command families

- `ghl companies list|get|create|update|delete`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.

## `contact` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/contact-tools.ts`
- Source tool count: 31
- CLI namespace: `contacts`
- Phase: 2
- Status: mvp
- Risk: high
- Scope: Contacts, tasks, notes, tags, followers, campaigns/workflows association

### Source tools

- `create_contact`
- `search_contacts`
- `get_contact`
- `update_contact`
- `delete_contact`
- `add_contact_tags`
- `remove_contact_tags`
- `get_contact_tasks`
- `create_contact_task`
- `get_contact_task`
- `update_contact_task`
- `delete_contact_task`
- `update_task_completion`
- `get_contact_notes`
- `create_contact_note`
- `get_contact_note`
- `update_contact_note`
- `delete_contact_note`
- `upsert_contact`
- `get_duplicate_contact`
- `get_contacts_by_business`
- `get_contact_appointments`
- `bulk_update_contact_tags`
- `bulk_update_contact_business`
- `add_contact_followers`
- `remove_contact_followers`
- `add_contact_to_campaign`
- `remove_contact_from_campaign`
- `remove_contact_from_all_campaigns`
- `add_contact_to_workflow`
- `remove_contact_from_workflow`

### Required CLI command families

- `ghl contacts create|search|get|update|delete|upsert|duplicates`
- `ghl contacts tags add|remove|bulk-update`
- `ghl contacts tasks list|get|create|update|delete|complete`
- `ghl contacts notes list|get|create|update|delete`
- `ghl contacts business list|bulk-update`
- `ghl contacts followers add|remove`
- `ghl contacts campaigns add|remove|remove-all`
- `ghl contacts workflows add|remove`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.
- Explicit profile policy gates for high-risk writes.
- Human confirmation or `--yes` behavior for real execution.
- Undo or compensation class for every write.
- MVP smoke classification and safe read smoke check.

## `conversation` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/conversation-tools.ts`
- Source tool count: 20
- CLI namespace: `conversations, messages`
- Phase: 2
- Status: mvp
- Risk: high
- Scope: Threads, messages, SMS/email send, recordings, transcriptions

### Source tools

- `send_sms`
- `send_email`
- `search_conversations`
- `get_conversation`
- `create_conversation`
- `update_conversation`
- `get_recent_messages`
- `delete_conversation`
- `get_email_message`
- `get_message`
- `upload_message_attachments`
- `update_message_status`
- `add_inbound_message`
- `add_outbound_call`
- `get_message_recording`
- `get_message_transcription`
- `download_transcription`
- `cancel_scheduled_message`
- `cancel_scheduled_email`
- `live_chat_typing`

### Required CLI command families

- `ghl conversations search|get|create|update|delete`
- `ghl conversations messages recent|get|email|get-message|status-update`
- `ghl messages send-sms|send-email|cancel-scheduled-sms|cancel-scheduled-email`
- `ghl messages inbound add`
- `ghl calls outbound add`
- `ghl messages attachments upload`
- `ghl messages recordings get`
- `ghl messages transcriptions get|download`
- `ghl live-chat typing`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.
- Explicit profile policy gates for high-risk writes.
- Human confirmation or `--yes` behavior for real execution.
- Undo or compensation class for every write.
- MVP smoke classification and safe read smoke check.

## `courses` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/courses-tools.ts`
- Source tool count: 32
- CLI namespace: `courses`
- Phase: 5
- Status: planned
- Risk: medium
- Scope: Courses, products, categories, posts, offers, enrollments, progress

### Source tools

- `get_course_importers`
- `create_course_importer`
- `get_course_products`
- `get_course_product`
- `create_course_product`
- `update_course_product`
- `delete_course_product`
- `get_course_categories`
- `create_course_category`
- `update_course_category`
- `delete_course_category`
- `get_courses`
- `get_course`
- `create_course`
- `update_course`
- `delete_course`
- `get_course_instructors`
- `add_course_instructor`
- `get_course_posts`
- `get_course_post`
- `create_course_post`
- `update_course_post`
- `delete_course_post`
- `get_course_offers`
- `create_course_offer`
- `update_course_offer`
- `delete_course_offer`
- `get_course_enrollments`
- `enroll_contact_in_course`
- `remove_course_enrollment`
- `get_student_progress`
- `update_lesson_completion`

### Required CLI command families

- `ghl courses importers list|create`
- `ghl courses products list|get|create|update|delete`
- `ghl courses categories list|create|update|delete`
- `ghl courses list|get|create|update|delete`
- `ghl courses instructors list|add`
- `ghl courses posts list|get|create|update|delete`
- `ghl courses offers list|create|update|delete`
- `ghl courses enrollments list|add|remove`
- `ghl courses progress get|update-lesson`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.

## `custom-field-v2` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/custom-field-v2-tools.ts`
- Source tool count: 8
- CLI namespace: `custom-fields`
- Phase: 3
- Status: planned
- Risk: medium
- Scope: Custom fields and folders by object key

### Source tools

- `ghl_get_custom_field_by_id`
- `ghl_create_custom_field`
- `ghl_update_custom_field`
- `ghl_delete_custom_field`
- `ghl_get_custom_fields_by_object_key`
- `ghl_create_custom_field_folder`
- `ghl_update_custom_field_folder`
- `ghl_delete_custom_field_folder`

### Required CLI command families

- `ghl custom-fields get|create|update|delete`
- `ghl custom-fields by-object`
- `ghl custom-fields folders create|update|delete`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.

## `custom-menus` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/custom-menus-tools.ts`
- Source tool count: 5
- CLI namespace: `custom-menus`
- Phase: 5
- Status: planned
- Risk: medium
- Scope: Custom menu CRUD

### Source tools

- `list_custom_menus`
- `create_custom_menu`
- `get_custom_menu`
- `update_custom_menu`
- `delete_custom_menu`

### Required CLI command families

- `ghl custom-menus list|get|create|update|delete`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.

## `email-isv` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/email-isv-tools.ts`
- Source tool count: 9
- CLI namespace: `email domains`
- Phase: 5
- Status: planned
- Risk: high
- Scope: Email domains, DNS, stats, providers, verification

### Source tools

- `verify_email`
- `ghl_list_email_domains`
- `ghl_add_email_domain`
- `ghl_verify_email_domain`
- `ghl_delete_email_domain`
- `ghl_get_domain_dns_records`
- `ghl_get_email_stats`
- `ghl_list_email_providers`
- `ghl_set_default_email_provider`

### Required CLI command families

- `ghl email verify`
- `ghl email domains list|add|verify|delete|dns-records`
- `ghl email stats`
- `ghl email providers list|set-default`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.
- Explicit profile policy gates for high-risk writes.
- Human confirmation or `--yes` behavior for real execution.
- Undo or compensation class for every write.

## `email` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/email-tools.ts`
- Source tool count: 5
- CLI namespace: `email, templates email`
- Phase: 3
- Status: planned
- Risk: high
- Scope: Email campaigns and templates

### Source tools

- `get_email_campaigns`
- `create_email_template`
- `get_email_templates`
- `update_email_template`
- `delete_email_template`

### Required CLI command families

- `ghl email campaigns list`
- `ghl templates email create|list|update|delete`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.
- Explicit profile policy gates for high-risk writes.
- Human confirmation or `--yes` behavior for real execution.
- Undo or compensation class for every write.

## `forms` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/forms-tools.ts`
- Source tool count: 4
- CLI namespace: `forms`
- Phase: 3
- Status: planned
- Risk: medium
- Scope: Forms and submissions

### Source tools

- `get_forms`
- `get_form_submissions`
- `get_form_by_id`
- `upload_form_custom_files`

### Required CLI command families

- `ghl forms list|get|submissions`
- `ghl forms files upload`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.

## `funnels` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/funnels-tools.ts`
- Source tool count: 8
- CLI namespace: `funnels`
- Phase: 3
- Status: planned
- Risk: medium
- Scope: Funnels, pages, redirects

### Source tools

- `get_funnels`
- `get_funnel`
- `get_funnel_pages`
- `count_funnel_pages`
- `create_funnel_redirect`
- `update_funnel_redirect`
- `delete_funnel_redirect`
- `get_funnel_redirects`

### Required CLI command families

- `ghl funnels list|get|pages|pages-count`
- `ghl funnels redirects list|create|update|delete`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.

## `invoices` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/invoices-tools.ts`
- Source tool count: 18
- CLI namespace: `invoices, estimates`
- Phase: 4
- Status: planned
- Risk: high
- Scope: Invoice/estimate templates, schedules, send, generation

### Source tools

- `create_invoice_template`
- `list_invoice_templates`
- `get_invoice_template`
- `update_invoice_template`
- `delete_invoice_template`
- `create_invoice_schedule`
- `list_invoice_schedules`
- `get_invoice_schedule`
- `create_invoice`
- `list_invoices`
- `get_invoice`
- `send_invoice`
- `create_estimate`
- `list_estimates`
- `send_estimate`
- `create_invoice_from_estimate`
- `generate_invoice_number`
- `generate_estimate_number`

### Required CLI command families

- `ghl invoices templates create|list|get|update|delete`
- `ghl invoices schedules create|list|get`
- `ghl invoices create|list|get|send|generate-number`
- `ghl estimates create|list|send|generate-number`
- `ghl estimates convert-to-invoice`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.
- Explicit profile policy gates for high-risk writes.
- Human confirmation or `--yes` behavior for real execution.
- Undo or compensation class for every write.

## `links` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/links-tools.ts`
- Source tool count: 6
- CLI namespace: `links`
- Phase: 3
- Status: planned
- Risk: high
- Scope: Trigger links / tracking links

### Source tools

- `get_links`
- `get_link`
- `create_link`
- `update_link`
- `delete_link`
- `search_links`

### Required CLI command families

- `ghl links list|get|create|update|delete|search`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.
- Explicit profile policy gates for high-risk writes.
- Human confirmation or `--yes` behavior for real execution.
- Undo or compensation class for every write.

## `location` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/location-tools.ts`
- Source tool count: 28
- CLI namespace: `locations`
- Phase: 2
- Status: mvp
- Risk: high
- Scope: Locations, tags, tasks, custom fields/values, templates, timezones

### Source tools

- `search_locations`
- `get_location`
- `create_location`
- `update_location`
- `delete_location`
- `get_location_tags`
- `create_location_tag`
- `get_location_tag`
- `update_location_tag`
- `delete_location_tag`
- `search_location_tasks`
- `get_location_custom_fields`
- `create_location_custom_field`
- `get_location_custom_field`
- `update_location_custom_field`
- `delete_location_custom_field`
- `get_location_custom_values`
- `create_location_custom_value`
- `get_location_custom_value`
- `update_location_custom_value`
- `delete_location_custom_value`
- `get_location_templates`
- `delete_location_template`
- `get_timezones`
- `create_recurring_task`
- `get_recurring_task`
- `update_recurring_task`
- `delete_recurring_task`

### Required CLI command families

- `ghl locations search|get|create|update|delete`
- `ghl locations tags list|get|create|update|delete`
- `ghl locations tasks search|recurring-create|get|update|delete`
- `ghl locations custom-fields list|get|create|update|delete`
- `ghl locations custom-values list|get|create|update|delete`
- `ghl locations templates list|delete`
- `ghl locations timezones list`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.
- Explicit profile policy gates for high-risk writes.
- Human confirmation or `--yes` behavior for real execution.
- Undo or compensation class for every write.
- MVP smoke classification and safe read smoke check.

## `marketplace` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/marketplace-tools.ts`
- Source tool count: 7
- CLI namespace: `integrations marketplace`
- Phase: 5
- Status: planned
- Risk: high
- Scope: Marketplace installs and billing charges

### Source tools

- `list_marketplace_installations`
- `delete_marketplace_installation`
- `list_billing_charges`
- `create_billing_charge`
- `check_billing_funds`
- `get_billing_charge`
- `delete_billing_charge`

### Required CLI command families

- `ghl integrations marketplace installs list|delete`
- `ghl integrations marketplace billing charges list|create|get|delete`
- `ghl integrations marketplace billing funds check`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.
- Explicit profile policy gates for high-risk writes.
- Human confirmation or `--yes` behavior for real execution.
- Undo or compensation class for every write.

## `media` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/media-tools.ts`
- Source tool count: 7
- CLI namespace: `media`
- Phase: 3
- Status: planned
- Risk: medium
- Scope: Media files/folders upload/update/delete

### Source tools

- `get_media_files`
- `upload_media_file`
- `delete_media_file`
- `create_media_folder`
- `bulk_update_media_files`
- `bulk_delete_media_files`
- `update_media_file`

### Required CLI command families

- `ghl media list|upload|update|delete|bulk-update|bulk-delete`
- `ghl media folders create`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.

## `oauth` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/oauth-tools.ts`
- Source tool count: 10
- CLI namespace: `integrations oauth, auth oauth`
- Phase: 5
- Status: planned
- Risk: high
- Scope: OAuth apps, installed locations, tokens, API keys

### Source tools

- `get_oauth_apps`
- `get_oauth_app`
- `get_installed_locations`
- `get_access_token_info`
- `get_location_access_token`
- `get_connected_integrations`
- `disconnect_integration`
- `get_api_keys`
- `create_api_key`
- `delete_api_key`

### Required CLI command families

- `ghl integrations oauth apps list|get`
- `ghl integrations oauth installed-locations`
- `ghl integrations oauth token-info`
- `ghl integrations oauth location-token`
- `ghl integrations connected list|disconnect`
- `ghl integrations api-keys list|create|delete`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.
- Explicit profile policy gates for high-risk writes.
- Human confirmation or `--yes` behavior for real execution.
- Undo or compensation class for every write.

## `object` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/object-tools.ts`
- Source tool count: 9
- CLI namespace: `objects`
- Phase: 5
- Status: planned
- Risk: medium
- Scope: Custom object schemas and records

### Source tools

- `get_all_objects`
- `create_object_schema`
- `get_object_schema`
- `update_object_schema`
- `create_object_record`
- `get_object_record`
- `update_object_record`
- `delete_object_record`
- `search_object_records`

### Required CLI command families

- `ghl objects schemas list|get|create|update`
- `ghl objects records search|get|create|update|delete`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.

## `opportunity` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/opportunity-tools.ts`
- Source tool count: 10
- CLI namespace: `opportunities, pipelines`
- Phase: 2
- Status: mvp
- Risk: high
- Scope: Opportunities, pipelines, statuses, followers

### Source tools

- `search_opportunities`
- `get_pipelines`
- `get_opportunity`
- `create_opportunity`
- `update_opportunity_status`
- `delete_opportunity`
- `update_opportunity`
- `upsert_opportunity`
- `add_opportunity_followers`
- `remove_opportunity_followers`

### Required CLI command families

- `ghl opportunities search|get|create|update|upsert|delete|status`
- `ghl opportunities followers add|remove`
- `ghl pipelines list`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.
- Explicit profile policy gates for high-risk writes.
- Human confirmation or `--yes` behavior for real execution.
- Undo or compensation class for every write.
- MVP smoke classification and safe read smoke check.

## `payments` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/payments-tools.ts`
- Source tool count: 22
- CLI namespace: `payments, orders, coupons`
- Phase: 4
- Status: planned
- Risk: high
- Scope: Orders, transactions, subscriptions, coupons, providers, order payment

### Source tools

- `create_whitelabel_integration_provider`
- `list_whitelabel_integration_providers`
- `list_orders`
- `get_order_by_id`
- `create_order_fulfillment`
- `list_order_fulfillments`
- `list_transactions`
- `get_transaction_by_id`
- `list_subscriptions`
- `get_subscription_by_id`
- `list_coupons`
- `create_coupon`
- `update_coupon`
- `delete_coupon`
- `get_coupon`
- `create_custom_provider_integration`
- `delete_custom_provider_integration`
- `get_custom_provider_config`
- `create_custom_provider_config`
- `disconnect_custom_provider_config`
- `get_order_notes`
- `record_order_payment`

### Required CLI command families

- `ghl payments providers whitelabel create|list`
- `ghl orders list|get|fulfillments list|create|notes list|record-payment`
- `ghl payments transactions list|get`
- `ghl subscriptions list|get`
- `ghl coupons list|get|create|update|delete`
- `ghl payments custom-provider create|delete|config-get|config-create|config-disconnect`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.
- Explicit profile policy gates for high-risk writes.
- Human confirmation or `--yes` behavior for real execution.
- Undo or compensation class for every write.

## `phone-system` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/phone-system-tools.ts`
- Source tool count: 15
- CLI namespace: `phone`
- Phase: 5
- Status: planned
- Risk: high
- Scope: Number pools, active numbers, buy/release, recordings, voicemail, BYOC

### Source tools

- `list_number_pools`
- `list_active_numbers_by_location`
- `ghl_search_available_numbers`
- `ghl_buy_phone_number`
- `ghl_release_phone_number`
- `ghl_get_phone_number`
- `ghl_list_phone_numbers`
- `ghl_update_phone_number`
- `ghl_get_call_recording`
- `ghl_list_call_recordings`
- `ghl_get_voicemail`
- `ghl_configure_call_forwarding`
- `ghl_get_byoc_trunk`
- `ghl_create_byoc_trunk`
- `ghl_list_byoc_trunks`

### Required CLI command families

- `ghl phone pools list`
- `ghl phone numbers active|search|buy|release|get|list|update`
- `ghl phone recordings get|list`
- `ghl phone voicemail get`
- `ghl phone forwarding configure`
- `ghl phone byoc get|create|list`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.
- Explicit profile policy gates for high-risk writes.
- Human confirmation or `--yes` behavior for real execution.
- Undo or compensation class for every write.

## `phone` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/phone-tools.ts`
- Source tool count: 20
- CLI namespace: `phone`
- Phase: 5
- Status: planned
- Risk: high
- Scope: Phone numbers, forwarding, IVR, voicemail, caller IDs

### Source tools

- `get_phone_numbers`
- `get_phone_number`
- `search_available_numbers`
- `purchase_phone_number`
- `update_phone_number`
- `release_phone_number`
- `get_call_forwarding_settings`
- `update_call_forwarding`
- `get_ivr_menus`
- `create_ivr_menu`
- `update_ivr_menu`
- `delete_ivr_menu`
- `get_voicemail_settings`
- `update_voicemail_settings`
- `get_voicemails`
- `delete_voicemail`
- `get_caller_ids`
- `add_caller_id`
- `verify_caller_id`
- `delete_caller_id`

### Required CLI command families

- `ghl phone numbers list|get|search|purchase|update|release`
- `ghl phone forwarding get|update`
- `ghl phone ivr list|create|update|delete`
- `ghl phone voicemail settings-get|settings-update|list|delete`
- `ghl phone caller-ids list|add|verify|delete`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.
- Explicit profile policy gates for high-risk writes.
- Human confirmation or `--yes` behavior for real execution.
- Undo or compensation class for every write.

## `products` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/products-tools.ts`
- Source tool count: 11
- CLI namespace: `products`
- Phase: 4
- Status: planned
- Risk: medium
- Scope: Products, prices, inventory, collections

### Source tools

- `ghl_create_product`
- `ghl_list_products`
- `ghl_get_product`
- `ghl_update_product`
- `ghl_delete_product`
- `ghl_create_price`
- `ghl_list_prices`
- `ghl_list_inventory`
- `ghl_create_product_collection`
- `ghl_list_product_collections`
- `ghl_bulk_edit_products`

### Required CLI command families

- `ghl products create|list|get|update|delete|bulk-edit`
- `ghl products prices create|list`
- `ghl products inventory list`
- `ghl products collections create|list`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.

## `proposals` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/proposals-tools.ts`
- Source tool count: 4
- CLI namespace: `proposals`
- Phase: 4
- Status: planned
- Risk: high
- Scope: Proposal/document listing and sending

### Source tools

- `list_proposals_documents`
- `send_proposal_document`
- `list_proposal_templates`
- `send_proposal_template`

### Required CLI command families

- `ghl proposals documents list|send`
- `ghl proposals templates list|send`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.
- Explicit profile policy gates for high-risk writes.
- Human confirmation or `--yes` behavior for real execution.
- Undo or compensation class for every write.

## `reporting` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/reporting-tools.ts`
- Source tool count: 12
- CLI namespace: `reports`
- Phase: 4
- Status: planned
- Risk: low
- Scope: Attribution, calls, appointments, pipeline, email, SMS, funnel, ads, agents, dashboards, conversions, revenue

### Source tools

- `get_attribution_report`
- `get_call_reports`
- `get_appointment_reports`
- `get_pipeline_reports`
- `get_email_reports`
- `get_sms_reports`
- `get_funnel_reports`
- `get_ad_reports`
- `get_agent_reports`
- `get_dashboard_stats`
- `get_conversion_reports`
- `get_revenue_reports`

### Required CLI command families

- `ghl reports attribution|calls|appointments|pipeline|email|sms|funnels|ads|agents|dashboard|conversions|revenue`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.

## `reputation` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/reputation-tools.ts`
- Source tool count: 15
- CLI namespace: `reputation`
- Phase: 4
- Status: planned
- Risk: high
- Scope: Reviews, replies, stats, requests, platforms, links, widgets

### Source tools

- `get_reviews`
- `get_review`
- `reply_to_review`
- `update_review_reply`
- `delete_review_reply`
- `get_review_stats`
- `send_review_request`
- `get_review_requests`
- `get_connected_review_platforms`
- `connect_google_business`
- `disconnect_review_platform`
- `get_review_links`
- `update_review_links`
- `get_review_widget_settings`
- `update_review_widget_settings`

### Required CLI command families

- `ghl reputation reviews list|get|reply|reply-update|reply-delete`
- `ghl reputation stats`
- `ghl reputation requests send|list`
- `ghl reputation platforms list|connect-google|disconnect`
- `ghl reputation links list|update`
- `ghl reputation widget get|update`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.
- Explicit profile policy gates for high-risk writes.
- Human confirmation or `--yes` behavior for real execution.
- Undo or compensation class for every write.

## `saas` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/saas-tools.ts`
- Source tool count: 12
- CLI namespace: `saas`
- Phase: 5
- Status: planned
- Risk: high
- Scope: SaaS locations/subscriptions/plans/rebilling/bulk enable-disable

### Source tools

- `get_saas_locations`
- `get_saas_location`
- `update_saas_subscription`
- `pause_saas_location`
- `enable_saas_location`
- `rebilling_update`
- `get_saas_agency_plans`
- `bulk_disable_saas`
- `bulk_enable_saas`
- `get_saas_subscription`
- `list_saas_locations_by_company`
- `get_saas_plan`

### Required CLI command families

- `ghl saas locations list|get|bulk-disable|bulk-enable|by-company`
- `ghl saas subscriptions get|update|pause|enable`
- `ghl saas rebilling update`
- `ghl saas plans list|get`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.
- Explicit profile policy gates for high-risk writes.
- Human confirmation or `--yes` behavior for real execution.
- Undo or compensation class for every write.

## `smartlists` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/smartlists-tools.ts`
- Source tool count: 8
- CLI namespace: `smart-lists`
- Phase: 3
- Status: planned
- Risk: medium
- Scope: Smart list CRUD, contacts, counts, duplicate

### Source tools

- `get_smart_lists`
- `get_smart_list`
- `create_smart_list`
- `update_smart_list`
- `delete_smart_list`
- `get_smart_list_contacts`
- `get_smart_list_count`
- `duplicate_smart_list`

### Required CLI command families

- `ghl smart-lists list|get|create|update|delete|duplicate`
- `ghl smart-lists contacts list`
- `ghl smart-lists count`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.

## `snapshots` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/snapshots-tools.ts`
- Source tool count: 7
- CLI namespace: `snapshots`
- Phase: 5
- Status: planned
- Risk: high
- Scope: Snapshots, push status, push, share links

### Source tools

- `get_snapshots`
- `get_snapshot`
- `create_snapshot`
- `get_snapshot_push_status`
- `get_latest_snapshot_push`
- `push_snapshot_to_subaccounts`
- `create_snapshot_share_link`

### Required CLI command families

- `ghl snapshots list|get|create|push-status|latest-push|push|share-link`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.
- Explicit profile policy gates for high-risk writes.
- Human confirmation or `--yes` behavior for real execution.
- Undo or compensation class for every write.

## `social-media` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/social-media-tools.ts`
- Source tool count: 19
- CLI namespace: `social`
- Phase: 5
- Status: planned
- Risk: high
- Scope: Social posts, accounts, CSV import, OAuth, stats

### Source tools

- `search_social_posts`
- `create_social_post`
- `get_social_post`
- `update_social_post`
- `delete_social_post`
- `bulk_delete_social_posts`
- `get_social_accounts`
- `delete_social_account`
- `upload_social_csv`
- `get_csv_upload_status`
- `set_csv_accounts`
- `get_social_categories`
- `get_social_category`
- `get_social_tags`
- `get_social_tags_by_ids`
- `start_social_oauth`
- `get_platform_accounts`
- `get_social_media_statistics`
- `set_social_media_accounts`

### Required CLI command families

- `ghl social posts search|create|get|update|delete|bulk-delete`
- `ghl social accounts list|delete|set`
- `ghl social csv upload|status|finalize`
- `ghl social categories list|get`
- `ghl social tags list|get-by-ids`
- `ghl social oauth start|platform-accounts`
- `ghl social stats`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.
- Explicit profile policy gates for high-risk writes.
- Human confirmation or `--yes` behavior for real execution.
- Undo or compensation class for every write.

## `store` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/store-tools.ts`
- Source tool count: 18
- CLI namespace: `store, shipping`
- Phase: 4
- Status: planned
- Risk: medium
- Scope: Shipping zones/rates/carriers, store settings

### Source tools

- `ghl_create_shipping_zone`
- `ghl_list_shipping_zones`
- `ghl_get_shipping_zone`
- `ghl_update_shipping_zone`
- `ghl_delete_shipping_zone`
- `ghl_get_available_shipping_rates`
- `ghl_create_shipping_rate`
- `ghl_list_shipping_rates`
- `ghl_get_shipping_rate`
- `ghl_update_shipping_rate`
- `ghl_delete_shipping_rate`
- `ghl_create_shipping_carrier`
- `ghl_list_shipping_carriers`
- `ghl_get_shipping_carrier`
- `ghl_update_shipping_carrier`
- `ghl_delete_shipping_carrier`
- `ghl_create_store_setting`
- `ghl_get_store_setting`

### Required CLI command families

- `ghl store shipping-zones create|list|get|update|delete`
- `ghl store shipping-rates available|create|list|get|update|delete`
- `ghl store shipping-carriers create|list|get|update|delete`
- `ghl store settings create|get`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.

## `survey` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/survey-tools.ts`
- Source tool count: 9
- CLI namespace: `surveys`
- Phase: 3
- Status: planned
- Risk: medium
- Scope: Surveys, submissions, stats

### Source tools

- `ghl_get_surveys`
- `ghl_get_survey_submissions`
- `ghl_create_survey`
- `ghl_get_survey`
- `ghl_update_survey`
- `ghl_delete_survey`
- `ghl_list_survey_submissions`
- `ghl_get_survey_submission`
- `ghl_get_survey_stats`

### Required CLI command families

- `ghl surveys list|get|create|update|delete`
- `ghl surveys submissions list|get`
- `ghl surveys stats`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.

## `templates` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/templates-tools.ts`
- Source tool count: 18
- CLI namespace: `templates, snippets`
- Phase: 3
- Status: planned
- Risk: medium
- Scope: SMS/voicemail/social/WhatsApp templates and snippets

### Source tools

- `get_sms_templates`
- `get_sms_template`
- `create_sms_template`
- `update_sms_template`
- `delete_sms_template`
- `get_voicemail_templates`
- `create_voicemail_template`
- `delete_voicemail_template`
- `get_social_templates`
- `create_social_template`
- `delete_social_template`
- `get_whatsapp_templates`
- `create_whatsapp_template`
- `delete_whatsapp_template`
- `get_snippets`
- `create_snippet`
- `update_snippet`
- `delete_snippet`

### Required CLI command families

- `ghl templates sms list|get|create|update|delete`
- `ghl templates voicemail create|delete|list`
- `ghl templates social create|delete|list`
- `ghl templates whatsapp create|delete|list`
- `ghl snippets list|create|update|delete`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.

## `triggers` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/triggers-tools.ts`
- Source tool count: 11
- CLI namespace: `triggers`
- Phase: 5
- Status: planned
- Risk: high
- Scope: Trigger CRUD, enable/disable, logs, test, duplicate

### Source tools

- `get_triggers`
- `get_trigger`
- `create_trigger`
- `update_trigger`
- `delete_trigger`
- `enable_trigger`
- `disable_trigger`
- `get_trigger_types`
- `get_trigger_logs`
- `test_trigger`
- `duplicate_trigger`

### Required CLI command families

- `ghl triggers list|get|create|update|delete|enable|disable|types|logs|test|duplicate`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.
- Explicit profile policy gates for high-risk writes.
- Human confirmation or `--yes` behavior for real execution.
- Undo or compensation class for every write.

## `users` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/users-tools.ts`
- Source tool count: 7
- CLI namespace: `users, teams, roles`
- Phase: 5
- Status: planned
- Risk: medium
- Scope: Users search/filter/CRUD; role/team reads added by spec

### Source tools

- `get_users`
- `get_user`
- `create_user`
- `update_user`
- `delete_user`
- `search_users`
- `filter_users_by_email`

### Required CLI command families

- `ghl users list|get|create|update|delete|search|filter-by-email`
- `ghl teams list`
- `ghl roles list|get`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.

## `voice-ai` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/voice-ai-tools.ts`
- Source tool count: 11
- CLI namespace: `voice-ai`
- Phase: 5
- Status: planned
- Risk: high
- Scope: Voice AI agents, actions, call logs

### Source tools

- `list_voice_ai_agents`
- `create_voice_ai_agent`
- `get_voice_ai_agent`
- `update_voice_ai_agent`
- `delete_voice_ai_agent`
- `create_voice_ai_action`
- `get_voice_ai_action`
- `update_voice_ai_action`
- `delete_voice_ai_action`
- `list_voice_ai_call_logs`
- `get_voice_ai_call_log`

### Required CLI command families

- `ghl voice-ai agents list|create|get|update|delete`
- `ghl voice-ai actions create|get|update|delete`
- `ghl voice-ai call-logs list|get`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.
- Explicit profile policy gates for high-risk writes.
- Human confirmation or `--yes` behavior for real execution.
- Undo or compensation class for every write.

## `webhooks` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/webhooks-tools.ts`
- Source tool count: 9
- CLI namespace: `webhooks`
- Phase: 5
- Status: planned
- Risk: high
- Scope: Webhook CRUD, events, logs, retry, test

### Source tools

- `get_webhooks`
- `get_webhook`
- `create_webhook`
- `update_webhook`
- `delete_webhook`
- `get_webhook_events`
- `get_webhook_logs`
- `retry_webhook`
- `test_webhook`

### Required CLI command families

- `ghl webhooks list|get|create|update|delete`
- `ghl webhooks events|logs|retry|test`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.
- Explicit profile policy gates for high-risk writes.
- Human confirmation or `--yes` behavior for real execution.
- Undo or compensation class for every write.

## `workflow-builder` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/workflow-builder-tools.ts`
- Source tool count: 7
- CLI namespace: `workflows`
- Phase: 3
- Status: planned
- Risk: high
- Scope: Full workflow builder CRUD/actions/publish/clone

### Source tools

- `ghl_create_workflow`
- `ghl_list_workflows_full`
- `ghl_get_workflow_full`
- `ghl_update_workflow_actions`
- `ghl_delete_workflow`
- `ghl_publish_workflow`
- `ghl_clone_workflow`

### Required CLI command families

- `ghl workflows create|list-full|get-full|update-actions|delete|publish|clone`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.
- Explicit profile policy gates for high-risk writes.
- Human confirmation or `--yes` behavior for real execution.
- Undo or compensation class for every write.

## `workflow` parity outline

- Source module: `references/Go-High-Level-MCP-2026-Complete/src/tools/workflow-tools.ts`
- Source tool count: 7
- CLI namespace: `workflows`
- Phase: 3
- Status: planned
- Risk: high
- Scope: Workflow list/get/status/delete/trigger/executions

### Source tools

- `ghl_get_workflows`
- `ghl_list_workflows`
- `ghl_get_workflow`
- `ghl_update_workflow_status`
- `ghl_delete_workflow`
- `ghl_trigger_workflow`
- `ghl_get_workflow_executions`

### Required CLI command families

- `ghl workflows list|get|status|delete|trigger|executions`

### Parity requirements

- Endpoint manifest entries for every backed operation.
- Command metadata with auth classes, scopes, policy flags, output schema version, and undo class.
- Redacted fixtures for list/get and at least one write path when writes exist.
- Mock tests for auth failure, permission/scope failure, validation failure, and upstream shape drift.
- Audit entries for every mutation and sensitive dry-run.
- Explicit profile policy gates for high-risk writes.
- Human confirmation or `--yes` behavior for real execution.
- Undo or compensation class for every write.

## Parity Gates

Feature parity is complete when:

- `data/reference-tools.json` still reports 45 modules and the expected tool count, or documented reference changes are reviewed.
- Every source tool maps to at least one endpoint key or a documented no-op/deferred reason.
- Every endpoint key maps to at least one CLI command or a documented raw-only/deferred reason.
- Every MVP and planned command has command metadata.
- Every implemented endpoint has fixture coverage.
- Every undocumented/internal endpoint has drift metadata.
- `ghl endpoints coverage` reports zero unclassified reference capabilities.
- `ghl commands schema` includes every implemented command and every planned command stub intended for parity tracking.
- Docs clearly separate implemented, planned, research, deferred, and out-of-scope capabilities.

## Immediate Next Steps

1. Expand `data/endpoints.json` beyond the seeded `locations.get` and `locations.search` records.
2. Add source-ref parsing for `ghl-internal-api-bible/*/endpoints.md`.
3. Generate richer `docs/API-COVERAGE.md` tables from `data/reference-tools.json` and `data/endpoints.json`.
4. Add planned command stubs to command metadata for all modules, even before implementation.
5. Pick the MVP subset and mark all non-MVP parity work as planned/research/deferred with reasons.
