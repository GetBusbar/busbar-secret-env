# Secret env ordinary assertion migration

Source freeze: plugin `80e13fcd3ac788f20d74abc46a231abed69c981c`, SDK and kit `a35493a5e74cc935ff7dcc15c1af869290da91cb`. No SDK or lock revision changes. No builds run: native execution belongs to the coordinated build lane.

The published macro and kit dependency edges have been removed after preserving the product/origin assertions locally and obtaining common-loader source ownership ACK. Native qualification remains pending.

## Product assertions

| Original secret script assertion | Ordinary test/helper |
| --- | --- |
| Facts, equal linked/dropped Statement, identity and lifecycle transcript | `the_linked_and_the_dropped_in_env_plugin_are_one_plugin` |
| Bad validate/open name reasons; successful open after refusal | transcript and `invalid_instance_settings_refuse_open_without_installing_an_instance` |
| Before-open resolve refuses with no material; duplicate open refuses | transcript plus `resolve_into` |
| Known resolve and known again are Ready, correct material, no lease, secret flag, caller buffer | transcript, `repeated_resolution_and_refresh_keep_material_until_close`, `resolve_into` |
| Short buffer Failed, exact needed length, no material or secret flag | transcript plus `resolve_into` |
| Unknown NOT_FOUND, malformed `[1]` INVALID, wrong-shape `{name:5}` INVALID | transcript plus repeated-resolution test |
| Release without lease refuses | transcript |
| Refused refresh retains material, successful refresh retains material, close then no resolve | repeated-resolution test |
| No configured credential in error text | `spelled` checks the configured credential (`VALUE`) at every ordinary call |
| Pre-open pure CHECK and invalid references | `reference_check_is_pure_and_refuses_bad_settings_through_both_doors` (additional existing coverage) |
| Empty value INVALID, fail closed and no host writes on refusal | transcript and `resolve_into` (additional existing coverage) |

Original optional `secret.empty`, `secret.check` and `bad_references` were absent from the former `conformance.json` (no longer shipped: plugin-ci reads it only for C plugins); their script branches contributed no configured cases. `ticketed_resolves_preserve_material_flags_buffers_and_refusals` now uses actual dispatcher tickets and Lent ownership for known, short, absent and two malformed settings over both doors, checks Ready, host buffers, secret flags, exact needed, zero leases and no lost wake; synchronous transcript coverage also remains.

## Generated test disposition (exact pinned macro)

| Generated test | Disposition |
| --- | --- |
| `no_configured_credential_crosses_back` | `spelled` checks errors; all ordinary binds now use `AuditSink`, which records diagnostic/log names/text and metric labels and immediately refuses either configured credential |
| `the_linked_and_the_dropped_in_door_are_one_plugin` | Product mapping above; ordinary helpers and ticketed tests now pin actual per-instance crossings and zero resumes |
| `red_a_moved_crossing_count_is_refused` | Retired kit comparator selfcheck; actual physical crossing checks are ordinary |
| `red_a_declared_class_no_case_reports_is_refused` | Secret kind has no plane billing class; no active secret assertion |
| `red_a_door_at_another_kind_abi_is_refused` | Local metadata ordinary tests listed in the removal mapping below |
| `red_a_door_built_against_another_kind_layout_is_refused` | Local metadata ordinary tests listed in the removal mapping below |
| `red_a_transport_built_against_another_frame_piece_is_refused` | Secret kind early return; no active assertion |
| `red_a_plane_built_against_another_layout_is_refused` | Secret kind early return; no active assertion |
| `red_a_statement_other_than_the_stated_one_is_refused` | Local complete Statement test below covers row/manifest byte mismatch and restated version |
| `red_a_door_asked_for_as_another_kind_is_refused` | Local metadata ordinary tests listed in the removal mapping below |
| `red_a_connection_over_an_undeclared_need_is_refused` | Acknowledged actual GitHub ordinary custody test below |
| `red_another_instances_connection_is_refused` | Acknowledged actual GitHub ordinary custody test below |
| `red_a_failing_ready_refuses_the_boot` | Acknowledged core actual-transport failed-ready test below |
| `red_an_outbound_field_byte_off_is_refused` | Secret kind has no outbound auth family; no active assertion |
| `red_an_outbound_door_writing_no_field_is_refused` | Secret kind has no outbound auth family; no active assertion |
| `red_a_second_token_fetch_is_refused` | Secret kind has no outbound auth family; no active assertion |
| `red_a_networked_door_with_no_connection_table_is_refused` | Acknowledged actual GitHub NoConnectionTable/Probe test below; no synthetic network capability retained |
| `a_store_refuses_a_schema_newer_than_its_build` | Secret kind early return; no active assertion |
| `the_declared_needs_are_the_statements` | `both_statements_agree_with_the_empty_declared_needs` parses declares and both actual Statements and asserts no needs |
| `the_suite_runs_in_the_profile_it_was_asked_to` | `ordinary_checks_run_in_the_requested_profile` |

## Core fakeproduct boundary

Core `secret_door_conformance_tests` product resolution/refusal/caller-buffer assertions map to the ordinary tests above. Core `secret_calls_tests` SecretRows registration/shared-instance/replacement/locking/logging/seal assertions, `secret_disk_tests` host disk custody, and `member_secret_tests` member handoff assertions require core tests using the real product. An ABI transcript does not replace those host assertions. Core owner source ACK retargets those unchanged host tests to the actual env product; native qualification remains independently required.

## Artifact requirement

Ordinary dynamic tests require an absolute `BUSBAR_PLUGIN_CDYLIB` pointing to the freshly built actual `busbar-secret-env-plugin` with `dropped-in`. Missing path or artifact hard fails. The build lane must use a clean isolated target, build the exact source, then pass the exact artifact path. Freshness cannot be established by a file-existence check alone; this incremental change does not claim CI freshness until wiring is complete.

## Exact crossing counters

Exact SDK Git-object inspection confirms `Plugin.inner` and `Instance.crossings/resumes` are public at the pinned revision. (An earlier intermediate note incorrectly inferred privacy from another checkout.) Ordinary lifecycle, CHECK and synchronous resolve helpers now read the actual per-instance counters before and after each call, asserting one first-entry crossing for plugin calls and zero for host-refused unopened/closed resolves and duplicate open. Actual outcomes are independently asserted by the transcript. Every env case asserts zero resumes. The ticketed resolve test also pins actual ready crossings to `has_ready` and exactly one actual crossing per submitted resolve, with zero resumes. No call/submission counter is substituted for actual entry counts, no ABI shim is introduced and no SDK pin is changed.

## Monday-audit composition decision

Both exact parents are preserved: ordinary migration `8a4620b437bde374f756605b746df7e65f2455f3` and audit `3ba1256715ea4f549df7eab89438087d6bc55fb7`. Root explicitly chose unknown-setting refusal before missing/blank key for combined malformed input; plain missing-key words stay unchanged. This is authorized audit composition, not a primary-1.5.5 proof or signed oracle difference.

Source conflict dispositions: preserve separate `reference_key` validation and `resolve_variable` byte/zeroizing custody; put donor unknown-setting check at the start of `reference_key`, so both CHECK and RESOLVE share precedence; retain donor invalid environment-name `=`/NUL checks before lookup. Keep ordinary actual-image/ticket/count/envelope assertions; redundant generated-suite prose is omitted while ordinary actual-product checks replace the macro. Unit conflicts retain independent donor equals/NUL/stray-setting tests and replace the old contradictory precedence expectation with exact combined-malformed and plain-missing controls. README audit updates retained. No SDK/public-pin/registry change. Rustfmt/diff checks only; native unrun.

## Framework removal and loader-negative ownership

The framework macro and busbar-test-kit manifest edges are removed. Locked package records were reconciled by dependency reachability from the real workspace crates; retained versions and source URLs are unchanged. All three Busbar kit packages are absent from these manifests and locks. Native validation is pending the exclusive coordinated lane; source migration is not a GREEN/native/release claim.

| Original generated loader assertion | Ordinary destination |
| --- | --- |
| ABI +/-1, pack error, linked admission, dropped manifest exact version refusal | Local `the_actual_kind_abi_is_checked_by_pack_linked_and_dropped_origins` |
| Kind layout xor1/zero, pack error and rebuild text, linked under honest and own rows | Local `the_actual_kind_layout_is_checked_under_honest_and_own_rows` |
| Changed row/manifest byte and restated actual version under honest row | Local `the_actual_statement_matches_each_row_manifest_and_restated_version` |
| Actual image requested as every other kind, exact linked/dropped refusal and honest twins | Local `the_actual_image_is_refused_as_every_other_kind_through_both_origins` and metadata tests' honest actual loads |
| Distinct physical linked/dropped door and operation-table addresses | Local `the_actual_dropped_image_is_distinct_from_the_linked_image` |
| Failed ready returns its reason and is entered once | Core `8227f0138ef2e44d4cefd25ce8fd3d51a9604b6a`, `crates/plugin-loader/src/tests/ordinary_load_negatives.rs::actual_transport_lifecycle_surrounds_one_failed_ready_without_retry`; common-loader SOURCE ownership ACK by doc_macro_bindings |
| Undeclared need and another instance's read/write/close, owner connection remains live | Actual GitHub `abd7663be59b670c8cceabaf30475334e771c566`, `auth-github-plugin/tests/support/remaining.rs::custody::github_connections_refuse_undeclared_needs_and_other_instances` |
| Named NoConnectionTable refusal and Probe success | Same actual GitHub source, `auth-github-plugin/tests/conformance.rs::github_serving_requires_its_connection_table_through_both_doors` |

The three connection predicates have exact SOURCE_EQUIVALENCE_ACK in `A/diet-loader-d306/transfer-packets/generic-connection-negative-transfer-ack.json`, SHA256 `ef00f652e3a7fa0f3f762e59f98e958fc586b225b3b7d1c1f351beaa75773573`; current composed GitHub `b6c823d90ef092920b74339dbe9a68f17b766a20` retains them. Their native qualification is separately pending the auth owner, not claimed here. These common mechanics are tested on an actual declared-needs product instead of grafting a synthetic network capability onto this no-needs product. The local metadata tests retain all per-product pack/origin oracles excluded from that ACK: immutable actual product tables are unchanged, only rejected numeric declarations/version differ. No shared replacement suite or fake product exists.

The moved-count RED comparator test is retired with its deleted comparator implementation; ordinary tests assert actual physical crossing/resume counts directly. Plane-only unreported classes/layout, transport-only frame-piece, outbound-auth-only three arms and store-only newer-schema branches executed no product assertions for this kind and are retired as such. No behavior failure or security oracle is silently retired.

The pinned reusable CI workflow still contains historical kit-name/comparator requirements; root renderer owns replacement and canonical fresh artifact wiring before eventual SDK qualification. No public pin is changed here.

Native diagnostic refinement: the historical last-byte Statement mutation corrupts the actual rendering's trailing claims count and gets Rendering refusal before equality comparison. The ordinary mismatch tests now change one same-length actual version-string byte, first proving the mutated rendering still parses, then retaining the exact linked/dropped StatementMismatch and separately restated-version refusal. This repairs the negative input to reach its original equality oracle; it does not weaken the expected refusal.
