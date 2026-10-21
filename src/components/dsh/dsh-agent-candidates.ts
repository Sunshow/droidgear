import { isJsonObject } from './dsh-config-utils'
import type { DshConfigEntryView } from '@/lib/bindings'

/** One candidate for a select control built from other entries' config. */
export interface CandidateOption {
  value: string
  label: string
}

/** Provider ids declared by `llm-pi-ai`, plus the DeepSeek official route. */
export function collectProviderOptions(
  entries: DshConfigEntryView[],
  label: (value: string) => string
): CandidateOption[] {
  const options = new Map<string, string>()

  for (const entry of entries) {
    if (entry.id !== 'llm-pi-ai' || !isJsonObject(entry.config)) continue
    const providers = entry.config['providers']
    if (!isJsonObject(providers)) continue
    for (const [id, value] of Object.entries(providers)) {
      const displayName =
        isJsonObject(value) && typeof value['displayName'] === 'string'
          ? value['displayName']
          : label(id)
      options.set(id, displayName)
    }
  }

  if (
    entries.some(
      entry =>
        entry.id === 'llm-deepseek' || entry.id === 'llm-deepseek-account'
    )
  ) {
    options.set('deepseek-official', label('deepseek-official'))
  }

  return [...options].map(([value, optionLabel]) => ({
    value,
    label: optionLabel,
  }))
}

/** Model ids declared by `llm-pi-ai` providers and the DeepSeek entries. */
export function collectModelOptions(
  entries: DshConfigEntryView[]
): CandidateOption[] {
  const options = new Map<string, string>()

  const addModels = (models: unknown) => {
    if (!Array.isArray(models)) return
    for (const model of models) {
      if (!isJsonObject(model) || typeof model['id'] !== 'string') continue
      const id = model['id']
      const name =
        typeof model['name'] === 'string' && model['name'].length > 0
          ? model['name']
          : id
      options.set(id, name)
    }
  }

  for (const entry of entries) {
    if (!isJsonObject(entry.config)) continue
    if (entry.id === 'llm-pi-ai') {
      const providers = entry.config['providers']
      if (isJsonObject(providers)) {
        for (const provider of Object.values(providers)) {
          if (isJsonObject(provider)) addModels(provider['models'])
        }
      }
    } else if (
      entry.id === 'llm-deepseek' ||
      entry.id === 'llm-deepseek-account'
    ) {
      addModels(entry.config['models'])
    }
  }

  return [...options].map(([value, label]) => ({ value, label }))
}
