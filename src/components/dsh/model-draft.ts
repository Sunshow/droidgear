import type { CustomModel, DshModel } from '@/lib/bindings'
import { findModelByIdOrAlias } from '@/lib/model-registry'

/** Modalities DSH accepts in a model's `input` declaration. */
const INPUT_MODALITIES = new Set(['text', 'image'])

/** One editable model row in the DSH provider dialog. */
export interface DshModelDraft {
  id: string
  name: string
  contextWindow: string
  maxTokens: string
  /** Request modalities, always starting with `text`. */
  input: string[]
  /**
   * True once the user (or a stored explicit value) pins the modalities, so
   * editing the model id no longer re-derives them from the registry.
   */
  inputTouched: boolean
  /** Original model, kept so reasoningEfforts/extra fields survive edits. */
  base: DshModel | null
}

/**
 * Normalize modalities: drop unknown values, de-duplicate, and always keep
 * `text` because chat models must accept text.
 */
export function normalizeInputModalities(
  input: readonly string[] | null | undefined
): string[] {
  const modalities = new Set<string>()
  for (const value of input ?? []) {
    if (INPUT_MODALITIES.has(value)) modalities.add(value)
  }
  modalities.add('text')
  return [
    ...(modalities.has('text') ? ['text'] : []),
    ...(modalities.has('image') ? ['image'] : []),
  ]
}

/**
 * Modalities to declare for a model: an explicit declaration wins, otherwise
 * the built-in registry is consulted, then DSH's `text`-only default.
 */
export function resolveInputModalities(
  id: string,
  provided?: readonly string[] | null
): string[] {
  if (provided && provided.length > 0) {
    return normalizeInputModalities(provided)
  }
  return normalizeInputModalities(
    findModelByIdOrAlias(id.trim())?.input ?? null
  )
}

/** Convert a stored DSH model into an editable draft. */
export function modelToDraft(model: DshModel): DshModelDraft {
  return {
    id: model.id,
    name: model.name ?? '',
    contextWindow: model.contextWindow?.toString() ?? '',
    maxTokens: model.maxTokens?.toString() ?? '',
    input: resolveInputModalities(model.id, model.input),
    inputTouched: Boolean(model.input && model.input.length > 0),
    base: model,
  }
}

/** Convert an editable draft back into the stored DSH model shape. */
export function draftToModel(draft: DshModelDraft): DshModel {
  const contextWindow = draft.contextWindow.trim()
  const maxTokens = draft.maxTokens.trim()
  return {
    ...(draft.base ?? {}),
    id: draft.id.trim(),
    name: draft.name.trim() || null,
    contextWindow: contextWindow ? Number(contextWindow) : null,
    maxTokens: maxTokens ? Number(maxTokens) : null,
    input: normalizeInputModalities(draft.input),
  }
}

/**
 * Build a draft for a model imported from a channel.
 *
 * Metadata the channel does not carry (context window, and max tokens/name
 * for its favorites) is filled from the built-in registry, mirroring the
 * backend enrichment performed when the provider is saved.
 */
export function draftFromChannelModel(model: CustomModel): DshModelDraft {
  const entry = findModelByIdOrAlias(model.model)
  return {
    id: model.model,
    name: model.displayName || entry?.name || '',
    contextWindow: entry?.contextWindow?.toString() ?? '',
    maxTokens:
      (model.maxOutputTokens ?? entry?.maxOutputTokens)?.toString() ?? '',
    input: resolveInputModalities(model.model, null),
    inputTouched: false,
    base: null,
  }
}
