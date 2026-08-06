import { describe, expect, it } from 'vitest'
import {
  createPiModelFromRegistry,
  enrichPiModelFromRegistry,
} from './pi-model-metadata'
import { findModelByIdOrAlias } from './model-registry'

const registryModel = findModelByIdOrAlias('gpt-5.2')

if (!registryModel) {
  throw new Error('Expected gpt-5.2 in the model registry')
}

describe('pi model metadata', () => {
  it('creates a Pi model without imposing the upstream API', () => {
    const model = createPiModelFromRegistry(registryModel)

    expect(model).toMatchObject({
      id: registryModel.id,
      name: registryModel.name,
      api: null,
      reasoning: registryModel.reasoning,
      input: registryModel.input,
      thinkingLevelMap: registryModel.thinkingLevelMap,
      contextWindow: registryModel.contextWindow,
      maxTokens: registryModel.maxOutputTokens,
    })
  })

  it('enriches aliases while preserving provider-specific fields', () => {
    const model = enrichPiModelFromRegistry({
      id: registryModel.aliases[0] ?? registryModel.id,
      name: 'Old name',
      api: 'openai-completions',
      reasoning: true,
      input: ['text', 'image'],
      contextWindow: 1,
      maxTokens: 2,
      cost: null,
      compat: { supportsDeveloperRole: false },
    })

    expect(model.name).toBe(registryModel.name)
    expect(model.contextWindow).toBe(registryModel.contextWindow)
    expect(model.maxTokens).toBe(registryModel.maxOutputTokens)
    expect(model.api).toBe('openai-completions')
    expect(model.reasoning).toBe(registryModel.reasoning)
    expect(model.input).toEqual(registryModel.input)
    expect(model.thinkingLevelMap).toEqual(registryModel.thinkingLevelMap)
    expect(model.compat).toEqual({ supportsDeveloperRole: false })
  })

  it('leaves unknown model IDs unchanged', () => {
    const model = {
      id: 'private-model',
      name: 'Private model',
    }

    expect(enrichPiModelFromRegistry(model)).toBe(model)
  })
})
