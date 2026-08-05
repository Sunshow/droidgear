---
name: register-model
description: Register a new AI model in DroidGear's model registry by fetching specs from models.dev. Use when the user asks to register, add, or support a new model (e.g., "register claude-fable-5") or to sync the registry with models.dev.
---

# Register New Model

The model registry (`src/lib/model-registry-data.json`) is the single source of
truth for model behavior. All behavioral lookups (`isStrictSamplingModel`,
`isAnthropicAdaptiveThinkingModel`, `supportsXhighEffort`, `supportsMaxEffort`,
`getDefaultMaxOutputTokens`) read from it. Registering a model is a data-only
change in the common case — do NOT touch `src/lib/utils.ts`.

## Sync with models.dev

For a full registry refresh ("sync the registry with models.dev"), do NOT
follow the single-model workflow below. Instead:

### A. Fetch the full dataset

```bash
curl -sSL --max-time 60 "https://models.dev/api.json" -o /tmp/models-dev-api.json
```

### B. Compare with the registry using OFFICIAL-lab priority

models.dev contains 180+ labs; third-party mirrors (snowflake-cortex,
nano-gpt, qiniu-ai, llmgateway, helicone, vivgrid, opencode, opencode-go,
greenpt, alibaba-cn, etc.) often report **different** context/output/temperature
values than the vendor. Always compare against the vendor's own lab first:

`anthropic`, `openai`, `google`, `deepseek`, `xai`, `mistral`, `moonshotai`,
`alibaba`, `zhipuai`, `minimax`, `xiaomi`, `kimi-for-coding`

Only fall back to a third-party lab when the vendor lab no longer lists the
model (e.g. deprecated models like `claude-opus-4`, `gpt-5-codex`, `o1-mini`).

Note: labs are case-sensitive — the MiniMax vendor lab uses `MiniMax-M2` while
the registry uses lowercase `minimax-m2`. Normalize IDs when matching.

### C. Apply updates

1. **Spec updates**: set `contextWindow` / `maxOutputTokens` from the vendor
   lab's `limit.context` / `limit.output`. Ignore third-party differences.
   When a vendor drops a model, use the best available source or leave it.
   Ambiguity: the registry may alias a newer model under an older id (e.g.
   `qwen-max` has alias `qwen3-max`) — align to the spec the alias points to.
2. **strictSampling**: set `"strictSampling": true` for EVERY entry whose main
   provider shows `temperature: false` — this includes GPT-5.x (except
   `gpt-5.3-chat-latest`, which is `temperature: true`), the o1/o3/o4 series,
   Kimi K2.5/K2.7/K3, and new-gen Claude (Sonnet 5, Opus 5, Fable 5).
3. **New models**: confirm scope with the user first (add all new chat/code
   models vs. only flagship models). Apply the single-model workflow below per
   model (official-lab spec, standard reasoning profile, alphabetical insert).
4. **Removals**: entries no longer present in models.dev at all (not even via
   alias) are deprecated — confirm with the user before removing, since user
   configs may still reference them. Removing requires cleaning up test
   assertions that reference those ids (e.g. `claude-jupiter-v1-p`).
5. **Efforts**: `reasoning_options` may be a dict OR a list of
   `{type: toggle|effort|budget_tokens}` objects. `toggle` means reasoning is
   optional → include `none`. Use the `effort` values when present (e.g.
   `kimi-k3` → `["none", "low", "high", "max"]`).

### D. Update tests + verify

- Add/remove model ids in `src/lib/utils.test.ts` describe blocks
  (`isStrictSamplingModel`, `isAnthropicAdaptiveThinkingModel`,
  `supportsMaxEffort`, `supportsXhighEffort`, `getDefaultMaxOutputTokens`) —
  flip assertions like `isStrictSamplingModel('gpt-5.2')` when strictSampling
  changes. Watch for spec-value assertions (e.g. gemini output 64000 → 65536).
- `src/lib/model-registry.test.ts`: add non-standard effort coverage (e.g.
  `kimi-k3`).
- `src/components/models/ModelDialog.test.tsx`: the strict-sampling
  extraArgs test uses a strict model id — repoint it if that model is removed.
- Run `npm run check:all` and fix any failures.

## Workflow

### 1. Fetch the model spec from models.dev

Fetch the full dataset (more reliable than the model page):

```bash
curl -sSL --max-time 30 "https://models.dev/api.json" -o /tmp/models-dev-api.json
```

Then extract the entry for the target model. The Anthropic lab uses a nested
`models` map; other labs use a flat `id -> spec` map:

```bash
python3 -c "
import json
data = json.load(open('/tmp/models-dev-api.json'))
# Search all labs for the model ID
for lab, models in data.items():
    models = models.get('models', models) if isinstance(models, dict) else {}
    for mid, m in models.items():
        if '<model-id>' in mid:
            print(lab, mid)
            print(json.dumps(m, indent=2))
"
```

If the model is not in the API data, it may be a hypothetical/requested model —
confirm with the user before proceeding.

### 2. Map spec fields to the registry entry

From the models.dev entry, extract:

| models.dev field     | Registry field            | Notes                                                           |
| -------------------- | ------------------------- | --------------------------------------------------------------- |
| `name`               | `name`                    | Display name, e.g. "Claude Opus 5"                              |
| `limit.context`      | `contextWindow`           | In tokens                                                       |
| `limit.output`       | `maxOutputTokens`         | In tokens                                                       |
| `temperature: false` | `strictSampling: true`    | Only set when the model rejects sampling params; omit otherwise |
| `reasoning_options`  | `reasoningConfig.efforts` | Map effort values, add `none` if reasoning is optional          |
| provider/lab         | `platform`                | See mapping below                                               |

Determine `platform` from the lab:

| Lab       | Platform                                                                |
| --------- | ----------------------------------------------------------------------- |
| Anthropic | `anthropic-messages`                                                    |
| OpenAI    | `openai-responses` (or `openai-completions` for chat-style)             |
| Google    | `gemini`                                                                |
| DeepSeek  | `openai-completions`                                                    |
| Others    | Check existing entries in `model-registry-data.json` for similar models |

### 3. Add the registry entry

Insert into `src/lib/model-registry-data.json` in **alphabetical order** by
`id`. Generate aliases by replacing hyphens with dots and dropping date-version
suffixes (e.g., `claude-opus-4-8` → alias `claude-opus-4.8`; no dot form exists
when the version has no separator, e.g. `claude-opus-5` → alias `claude-opus.5`).

**Standard reasoning profile** (models.dev `reasoning_options` lists effort
values; use the profile matching the lab's native encoding):

```json
{
  "id": "<model-id>",
  "name": "<Display Name>",
  "aliases": ["<dotted-alias>"],
  "platform": "<platform>",
  "contextWindow": 1000000,
  "maxOutputTokens": 128000,
  "reasoningConfig": {
    "efforts": ["none", "low", "medium", "high", "xhigh", "max"],
    "profiles": {
      "anthropic": "anthropic-adaptive",
      "openai": "openai-reasoning",
      "generic-chat-completion-api": "openai-reasoning"
    }
  }
}
```

Profile selection for `reasoningConfig.profiles.anthropic`:

| Profile                   | When to use                                                                |
| ------------------------- | -------------------------------------------------------------------------- |
| `anthropic-adaptive`      | New-generation Anthropic models (adaptive thinking + output_config effort) |
| `anthropic-budget`        | Older Anthropic models (thinking budget_tokens)                            |
| `anthropic-output-config` | Non-Anthropic models routed through the Anthropic-compatible endpoint      |

**Non-standard effort encoding** (rare): if the model needs custom
`extraArgsFragment` per effort (e.g., `deepseek-v4-pro` uses
`thinking: {type: enabled}` + `reasoning_effort`), copy the `encoding` block
from an existing entry with the same shape instead of `profiles`.

**Strict sampling**: if models.dev shows `temperature: false` for the main
provider, add `"strictSampling": true` to the entry.

### 4. Check for special cases

- **Official display name**: if the model is one of DroidGear's official
  models (e.g., "Opus 5", "Sonnet 4.6"), add its display name to
  `DROID_OFFICIAL_MODEL_NAMES` in `src/lib/utils.ts`. This is the ONLY
  supported edit to `utils.ts`.
- **New ID prefix**: if the model introduces a brand-new prefix (not
  `claude-`/`gpt-`/`o1-`/`o3-`/`o4-`/`gemini-`), update protocol inference:
  - `src/lib/model-protocol/global-inference.ts`
  - `src/lib/model-protocol/channel-inferrers/`
  - `src/lib/sub2api-platform.ts`
  - `src/lib/newapi-platform.ts`

### 5. Update tests

- `src/lib/utils.test.ts`: add the model to the applicable describe blocks
  (`isStrictSamplingModel`, `isAnthropicAdaptiveThinkingModel`,
  `supportsXhighEffort`, `getDefaultMaxOutputTokens`) — the lookups are
  registry-driven, so these assertions should pass without code changes.
- `src/lib/model-registry.test.ts`: add registry coverage if the model has a
  non-standard `reasoningConfig` shape.

### 6. Verify

```bash
npm run check:all
```

Fix any lint or type errors before declaring the task complete.

## Verification

- `npm run check:all` passes with zero errors
- New entry is in `model-registry-data.json` in alphabetical order
- `strictSampling` set only when models.dev shows `temperature: false`
- `utils.ts` untouched except `DROID_OFFICIAL_MODEL_NAMES` (if applicable)
- No unexpected changes to protocol inference or Rust code
- For full syncs: verify the diff contains ONLY intended entries (removed /
  added / spec-updated), with no reformatting of untouched entries; confirm
  removed ids are also gone from test assertions
