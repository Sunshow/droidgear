import type { JsonValue } from '@/lib/bindings'
import type { ClaudeSettingsDoc } from '@/store/claude-settings-store'

export const CLAUDE_BASE_URL_ENV = 'ANTHROPIC_BASE_URL'
export const CLAUDE_AUTH_TOKEN_ENV = 'ANTHROPIC_AUTH_TOKEN'
export const CLAUDE_MODEL_ENV = 'ANTHROPIC_MODEL'
export const CLAUDE_SMALL_MODEL_ENV = 'ANTHROPIC_DEFAULT_HAIKU_MODEL'
export const CLAUDE_EFFORT_ENV = 'CLAUDE_CODE_EFFORT_LEVEL'
export const CLAUDE_DISABLE_ADAPTIVE_ENV =
  'CLAUDE_CODE_DISABLE_ADAPTIVE_THINKING'
export const CLAUDE_DISABLE_THINKING_ENV = 'CLAUDE_CODE_DISABLE_THINKING'
export const CLAUDE_MAX_THINKING_TOKENS_ENV = 'MAX_THINKING_TOKENS'

/** Suffix that Claude Code uses to enable 1M context windows. */
export const MODEL_1M_SUFFIX = '[1m]'

export type ClaudeReasoningEffort =
  | 'inherit'
  | 'low'
  | 'medium'
  | 'high'
  | 'max'

export type ClaudeThinkingMode = 'inherit' | 'on' | 'off'

const REASONING_VALUES: ReadonlySet<string> = new Set([
  'low',
  'medium',
  'high',
  'max',
])

function getEnvObject(
  doc: ClaudeSettingsDoc | null | undefined
): Record<string, JsonValue> | null {
  if (!doc) return null
  const env = doc.env
  if (!env || typeof env !== 'object' || Array.isArray(env)) return null
  return env as Record<string, JsonValue>
}

function getOrCreateEnv(draft: ClaudeSettingsDoc): Record<string, JsonValue> {
  const existing = draft.env
  if (existing && typeof existing === 'object' && !Array.isArray(existing)) {
    return existing as Record<string, JsonValue>
  }
  const next: Record<string, JsonValue> = {}
  draft.env = next
  return next
}

function pruneEnvIfEmpty(draft: ClaudeSettingsDoc): void {
  const env = draft.env
  if (
    env &&
    typeof env === 'object' &&
    !Array.isArray(env) &&
    Object.keys(env as Record<string, JsonValue>).length === 0
  ) {
    Reflect.deleteProperty(draft, 'env')
  }
}

export function getEnvString(
  doc: ClaudeSettingsDoc | null | undefined,
  key: string
): string | null {
  const env = getEnvObject(doc)
  if (!env) return null
  const value = env[key]
  if (typeof value !== 'string') return null
  const trimmed = value.trim()
  return trimmed.length > 0 ? trimmed : null
}

export function setEnvString(
  draft: ClaudeSettingsDoc,
  key: string,
  value: string | null
): void {
  const trimmed = value?.trim() ?? ''
  if (trimmed.length === 0) {
    const env = getEnvObject(draft)
    if (!env) return
    if (key in env) {
      Reflect.deleteProperty(env, key)
      pruneEnvIfEmpty(draft)
    }
    return
  }
  const env = getOrCreateEnv(draft)
  env[key] = trimmed
}

function deleteEnvKey(draft: ClaudeSettingsDoc, key: string): void {
  const env = getEnvObject(draft)
  if (!env) return
  if (key in env) {
    Reflect.deleteProperty(env, key)
    pruneEnvIfEmpty(draft)
  }
}

export function getReasoningEffort(
  doc: ClaudeSettingsDoc | null | undefined
): ClaudeReasoningEffort {
  const raw = getEnvString(doc, CLAUDE_EFFORT_ENV)
  if (!raw) return 'inherit'
  const normalized = raw.toLowerCase()
  return REASONING_VALUES.has(normalized)
    ? (normalized as ClaudeReasoningEffort)
    : 'inherit'
}

export function setReasoningEffort(
  draft: ClaudeSettingsDoc,
  effort: ClaudeReasoningEffort
): void {
  if (effort === 'inherit') {
    deleteEnvKey(draft, CLAUDE_EFFORT_ENV)
    deleteEnvKey(draft, CLAUDE_DISABLE_ADAPTIVE_ENV)
    return
  }
  const env = getOrCreateEnv(draft)
  env[CLAUDE_EFFORT_ENV] = effort
  if (effort === 'high' || effort === 'max') {
    env[CLAUDE_DISABLE_ADAPTIVE_ENV] = '1'
  } else {
    Reflect.deleteProperty(env, CLAUDE_DISABLE_ADAPTIVE_ENV)
    pruneEnvIfEmpty(draft)
  }
}

function getRootBool(
  doc: ClaudeSettingsDoc | null | undefined,
  key: string
): boolean | null {
  if (!doc) return null
  const value = doc[key]
  return typeof value === 'boolean' ? value : null
}

export function getThinkingMode(
  doc: ClaudeSettingsDoc | null | undefined
): ClaudeThinkingMode {
  const always = getRootBool(doc, 'alwaysThinkingEnabled')
  if (always === true) return 'on'
  if (always === false) return 'off'
  const disable = getEnvString(doc, CLAUDE_DISABLE_THINKING_ENV)
  if (disable === '1' || disable?.toLowerCase() === 'true') return 'off'
  return 'inherit'
}

export function setThinkingMode(
  draft: ClaudeSettingsDoc,
  mode: ClaudeThinkingMode
): void {
  switch (mode) {
    case 'inherit':
      Reflect.deleteProperty(draft, 'alwaysThinkingEnabled')
      deleteEnvKey(draft, CLAUDE_DISABLE_THINKING_ENV)
      deleteEnvKey(draft, CLAUDE_MAX_THINKING_TOKENS_ENV)
      return
    case 'on':
      draft.alwaysThinkingEnabled = true
      deleteEnvKey(draft, CLAUDE_DISABLE_THINKING_ENV)
      deleteEnvKey(draft, CLAUDE_MAX_THINKING_TOKENS_ENV)
      return
    case 'off': {
      draft.alwaysThinkingEnabled = false
      const env = getOrCreateEnv(draft)
      env[CLAUDE_DISABLE_THINKING_ENV] = '1'
      Reflect.deleteProperty(env, CLAUDE_MAX_THINKING_TOKENS_ENV)
      pruneEnvIfEmpty(draft)
      return
    }
  }
}

/**
 * Mirroring is ON only when the small model is explicitly set to the same
 * value as the main model. This aligns with the Rust apply logic where
 * `small_model_uses_main_model = true` writes
 * `ANTHROPIC_DEFAULT_HAIKU_MODEL = model`.
 *
 * When nothing is configured or the small model differs from the main
 * model, mirroring is OFF.
 */
export function isSmallModelMirroringMain(
  doc: ClaudeSettingsDoc | null | undefined
): boolean {
  const main = getEnvString(doc, CLAUDE_MODEL_ENV)
  const small = getEnvString(doc, CLAUDE_SMALL_MODEL_ENV)
  if (!main || !small) return false
  return small === main
}

export function setSmallModelMirroring(
  draft: ClaudeSettingsDoc,
  mirror: boolean,
  mainModel: string | null
): void {
  if (mirror) {
    // Enable mirroring: set small model to main model value.
    // This aligns with Rust `apply_profile_to_settings_path` which writes
    // `ANTHROPIC_DEFAULT_HAIKU_MODEL = model` when
    // `small_model_uses_main_model = true`.
    if (mainModel) {
      setEnvString(draft, CLAUDE_SMALL_MODEL_ENV, mainModel)
    }
  } else {
    // Disable mirroring: clear the small model so the user can input a different one.
    setEnvString(draft, CLAUDE_SMALL_MODEL_ENV, null)
  }
}

// ============================================================================
// 1M Context helpers
// ============================================================================

/**
 * Read the resolved model value, preferring env.ANTHROPIC_MODEL over the
 * top-level `model` field (matches Rust `build_current_config_from_settings`).
 */
function getResolvedModel(
  doc: ClaudeSettingsDoc | null | undefined
): string | null {
  const envModel = getEnvString(doc, CLAUDE_MODEL_ENV)
  if (envModel) return envModel
  if (!doc) return null
  const topLevel = doc.model
  return typeof topLevel === 'string' && topLevel.trim().length > 0
    ? topLevel.trim()
    : null
}

/**
 * Returns true when the current model name includes the `[1m]` suffix.
 * Checks env.ANTHROPIC_MODEL first, then falls back to top-level `model`.
 */
export function hasModel1MContext(
  doc: ClaudeSettingsDoc | null | undefined
): boolean {
  const model = getResolvedModel(doc)
  return model?.includes(MODEL_1M_SUFFIX) ?? false
}

/**
 * Toggle the `[1m]` suffix on the model name.
 * Manages env.ANTHROPIC_MODEL; also cleans up a stale top-level `model` field.
 */
export function toggleModel1MContext(
  draft: ClaudeSettingsDoc,
  enabled: boolean
): void {
  const current = getResolvedModel(draft)
  if (!current) return

  const stripped = current.replaceAll(MODEL_1M_SUFFIX, '').trim()
  if (!stripped) return

  const next = enabled ? stripped + MODEL_1M_SUFFIX : stripped
  setEnvString(draft, CLAUDE_MODEL_ENV, next)
  syncTopLevelModel(draft)
}

/**
 * When `env.ANTHROPIC_MODEL` is set, remove the top-level `model` field
 * so the two never conflict. When env is absent but top-level `model` exists,
 * leave it alone (the user may rely on it).
 */
export function syncTopLevelModel(draft: ClaudeSettingsDoc): void {
  const envModel = getEnvString(draft, CLAUDE_MODEL_ENV)
  if (envModel) {
    Reflect.deleteProperty(draft, 'model')
  }
}

/**
 * Clean up the document before saving. Removes stale top-level `model` when
 * `ANTHROPIC_MODEL` env is present, keeping the config unambiguous.
 */
export function cleanupDocument(draft: ClaudeSettingsDoc): void {
  syncTopLevelModel(draft)
}
