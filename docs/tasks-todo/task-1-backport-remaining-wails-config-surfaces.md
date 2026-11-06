# Backport the remaining wails-branch config surfaces (codex / hermes)

Source branch: `spike/wails-v3-minimal` (Wails v3 port). The Pi and OMP half of
that branch's 2026-11-06 upstream pass is already backported to the Tauri line
(`pi_merge.rs`, `pi_settings.rs`, `pi_startup.rs`, `omp_roles.rs`,
`omp_import.rs`, `channel_import.rs`). This task covers what was deliberately
left out.

## Codex (`src-tauri/crates/droidgear-core/src/codex.rs`)

Upstream `[model_providers.<id>]` grew these keys; the Wails branch models and
edits them, the Tauri line only preserves unknown ones:

- `model_catalog_url`
- `capabilities.external_web_access`, `capabilities.remote_compaction`
- `env_http_headers` (KEY → environment-variable name)
- `request_max_retries`, `stream_max_retries`, `stream_idle_timeout_ms`
- `supports_websockets`, `supports_standalone_web_search`
- command-backed `auth` (`argv`, `args`, `timeout_ms`, `refresh_interval_ms`,
  `cwd`)
- top-level `model_auto_compact_token_limit_scope` and
  `model_post_turn_compact_threshold_percent` (written from the active
  provider, backfilled by `ReadCurrentConfig`)

Work:

1. Extend `CodexProviderConfig` (+ the profile) with the fields above and keep
   them out of the "unknown keys" bucket on apply.
2. Add a `ProviderDialog` "Advanced (upstream config keys)" section with the
   inputs; reuse the `codex.provider.*` locale keys that already exist for the
   `wire_api` hint.
3. TUI parity: `keys_codex.rs` fields + `modal.rs` handlers.
4. Tests: apply keeps built-ins (gcp / amazon-bedrock / ollama / lmstudio),
   does not resurrect a cleared `env_key`, and round-trips the new keys.

## Hermes (`src-tauri/crates/droidgear-core/src/hermes.rs`)

Config v12 per-provider request knobs and per-model options:

- provider entry: `extra_headers`, `session_affinity_header`,
  `discover_models`, `transport`, `capabilities`
- model entry: `context_length`, `prompt_caching`, `answer_in_reasoning`,
  `api_mode`, `streaming`
- deleting the active provider must be refused

Work:

1. Model the fields in `HermesModelConfig` (they currently survive only because
   the apply is a read-modify-write over the YAML).
2. `HermesConfigPage`: an "Advanced provider options" `<details>` block per
   model entry, plus the extra-headers KEY=VALUE editor
   (`src/lib/key-value.ts` in the Wails branch is the helper to port).
3. TUI parity: `keys_hermes.rs` + `modal.rs`.
4. Refuse deleting the active provider in core (GUI + TUI surface the error).

## Pi channel-import unification (rule 9)

The Wails branch moved Pi's channel-import inference into
`service/pi/pi_import.go`. On the Tauri line the GUI dialog
(`src/components/pi/PiImportFromChannelDialog.tsx`) still infers in TypeScript
while the TUI (`keys_pi.rs` / `modal.rs`) does its own mapping, so the two can
drift. Port the inference to `droidgear-core` (mirroring
`omp_import::prepare_omp_channel_import`, which is already shared) and call it
from both consumers.

## Notes

- Do not port `dg`-specific pieces: the single-file self-update, the Go TUI and
  the module batch table are Wails-shell features.
- Keep `docs/agents/tool-config-sources.md` accurate as each row lands.
