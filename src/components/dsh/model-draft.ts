import type { CustomModel, DshModel } from '@/lib/bindings'
import { findModelByIdOrAlias } from '@/lib/model-registry'

/** One editable model row in the DSH provider dialog. */
export interface DshModelDraft {
  id: string
  name: string
  contextWindow: string
  maxTokens: string
  /** Original model, kept so reasoningEfforts/extra fields survive edits. */
  base: DshModel | null
}

/** Convert a stored DSH model into an editable draft. */
export function modelToDraft(model: DshModel): DshModelDraft {
  return {
    id: model.id,
    name: model.name ?? '',
    contextWindow: model.contextWindow?.toString() ?? '',
    maxTokens: model.maxTokens?.toString() ?? '',
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
    base: null,
  }
}
