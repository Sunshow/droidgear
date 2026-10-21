import type {
  DshConfigEntryView,
  DshConfigFieldView,
  DshConfigSnapshot,
  JsonValue,
} from '@/lib/bindings'

/** A JSON object as exposed by the Tauri bindings. */
export type JsonObject = Record<string, JsonValue>

/** Narrow a JSON value to an object (excluding arrays and null). */
export function isJsonObject(
  value: JsonValue | null | undefined
): value is JsonObject {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

/** Whether a field value is a `{ "$js": "<expression>" }` marker. */
export function isJsExpression(value: JsonValue | null | undefined): boolean {
  if (!isJsonObject(value)) return false
  const keys = Object.keys(value)
  return keys.length === 1 && typeof value['$js'] === 'string'
}

/** Whether a field value is a `{ "$secret": true }` redaction marker. */
export function isSecretMarker(value: JsonValue | null | undefined): boolean {
  return isJsonObject(value) && value['$secret'] === true
}

/** Structural equality for small JSON values (config fields). */
export function jsonEquals(
  a: JsonValue | null | undefined,
  b: JsonValue | null | undefined
): boolean {
  return JSON.stringify(a ?? null) === JSON.stringify(b ?? null)
}

/** Human-readable text for a read-only JSON value. */
export function jsonText(value: JsonValue | null | undefined): string {
  if (value === null || value === undefined) return ''
  if (typeof value === 'string') return value
  return JSON.stringify(value, null, 2)
}

/** Find one composed entry in a snapshot. */
export function findEntry(
  snapshot: DshConfigSnapshot | null,
  entryId: string
): DshConfigEntryView | undefined {
  return snapshot?.entries.find(entry => entry.id === entryId)
}

/** Find one field on an entry view. */
export function findField(
  entry: DshConfigEntryView,
  fieldName: string
): DshConfigFieldView | undefined {
  return entry.fields.find(field => field.name === fieldName)
}

/** Translation key for a composed entry's layer. */
export function layerLabelKey(
  layer: string
):
  | 'dsh.entry.layer.profile'
  | 'dsh.entry.layer.home'
  | 'dsh.entry.layer.bundle' {
  if (layer === 'profile') return 'dsh.entry.layer.profile'
  if (layer === 'home') return 'dsh.entry.layer.home'
  return 'dsh.entry.layer.bundle'
}

/** Translation key for an entry's tier. */
export function tierLabelKey(
  tier: string
): 'dsh.entry.tier.A' | 'dsh.entry.tier.B' | 'dsh.entry.tier.C' {
  if (tier === 'A') return 'dsh.entry.tier.A'
  if (tier === 'B') return 'dsh.entry.tier.B'
  return 'dsh.entry.tier.C'
}

/** Literal translation keys for the entries with a curated surface. */
export type EntryTitleKey =
  | 'dsh.entry.title.locale'
  | 'dsh.entry.title.ui-theme'
  | 'dsh.entry.title.ui-chat'
  | 'dsh.entry.title.ui-conversation'
  | 'dsh.entry.title.ui-settings'
  | 'dsh.entry.title.session-log-deepseek'
  | 'dsh.entry.title.agent-default-model'
  | 'dsh.entry.title.agent-loop'
  | 'dsh.entry.title.subagent'
  | 'dsh.entry.title.subagent-model-selection-settings'
  | 'dsh.entry.title.permission'
  | 'dsh.entry.title.sandbox-policy'
  | 'dsh.entry.title.approval'
  | 'dsh.entry.title.agent-preset-registry'
  | 'dsh.entry.title.web'
  | 'dsh.entry.title.web-search-deepseek'
  | 'dsh.entry.title.web-deepseek-api'
  | 'dsh.entry.title.web-fetch-http'

/** Translation key for a curated entry's title, or `null` for generic rows. */
export function entryTitleKey(entryId: string): EntryTitleKey | null {
  switch (entryId) {
    case 'locale':
      return 'dsh.entry.title.locale'
    case 'ui-theme':
      return 'dsh.entry.title.ui-theme'
    case 'ui-chat':
      return 'dsh.entry.title.ui-chat'
    case 'ui-conversation':
      return 'dsh.entry.title.ui-conversation'
    case 'ui-settings':
      return 'dsh.entry.title.ui-settings'
    case 'session-log-deepseek':
      return 'dsh.entry.title.session-log-deepseek'
    case 'agent-default-model':
      return 'dsh.entry.title.agent-default-model'
    case 'agent-loop':
      return 'dsh.entry.title.agent-loop'
    case 'subagent':
      return 'dsh.entry.title.subagent'
    case 'subagent-model-selection-settings':
      return 'dsh.entry.title.subagent-model-selection-settings'
    case 'permission':
      return 'dsh.entry.title.permission'
    case 'sandbox-policy':
      return 'dsh.entry.title.sandbox-policy'
    case 'approval':
      return 'dsh.entry.title.approval'
    case 'agent-preset-registry':
      return 'dsh.entry.title.agent-preset-registry'
    case 'web':
      return 'dsh.entry.title.web'
    case 'web-search-deepseek':
      return 'dsh.entry.title.web-search-deepseek'
    case 'web-deepseek-api':
      return 'dsh.entry.title.web-deepseek-api'
    case 'web-fetch-http':
      return 'dsh.entry.title.web-fetch-http'
    default:
      return null
  }
}
