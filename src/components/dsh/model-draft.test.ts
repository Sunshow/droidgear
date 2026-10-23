import { describe, expect, it } from 'vitest'
import type { CustomModel } from '@/lib/bindings'
import {
  draftFromChannelModel,
  draftToModel,
  modelToDraft,
  normalizeInputModalities,
  resolveInputModalities,
} from './model-draft'

function channelModel(overrides: Partial<CustomModel>): CustomModel {
  return {
    model: 'model-id',
    baseUrl: 'https://sub.wududu.com/v1',
    apiKey: 'sk-test',
    provider: 'generic-chat-completion-api',
    ...overrides,
  }
}

describe('draftFromChannelModel', () => {
  it('fills the context window from the registry for deepseek models', () => {
    // 渠道模型不携带上下文窗口，必须由内置注册表补齐（与保存时的后端填充一致）
    for (const id of [
      'deepseek-flash',
      'deepseek-v4-pro',
      'deepseek-v4.1-flash',
    ]) {
      const draft = draftFromChannelModel(channelModel({ model: id }))
      expect(draft.contextWindow).toBe('1000000')
      expect(draft.maxTokens).toBe('393216')
    }
  })

  it('keeps channel name and max output tokens when present', () => {
    const draft = draftFromChannelModel(
      channelModel({
        model: 'deepseek-v4-pro',
        displayName: 'My DeepSeek',
        maxOutputTokens: 8192,
      })
    )
    expect(draft.name).toBe('My DeepSeek')
    expect(draft.maxTokens).toBe('8192')
    expect(draft.contextWindow).toBe('1000000')
  })

  it('leaves unknown models without registry metadata', () => {
    const draft = draftFromChannelModel(
      channelModel({ model: 'totally-unknown-model' })
    )
    expect(draft.contextWindow).toBe('')
    expect(draft.maxTokens).toBe('')
    expect(draft.name).toBe('')
    expect(draft.input).toEqual(['text'])
  })

  it('fills image support from the registry aliases', () => {
    expect(
      draftFromChannelModel(channelModel({ model: 'deepseek-flash' })).input
    ).toEqual(['text', 'image'])
    expect(
      draftFromChannelModel(channelModel({ model: 'deepseek-v4-pro' })).input
    ).toEqual(['text'])
  })
})

describe('resolveInputModalities', () => {
  it('always keeps text and orders text before image', () => {
    expect(normalizeInputModalities(['image'])).toEqual(['text', 'image'])
    expect(normalizeInputModalities(['image', 'text', 'image'])).toEqual([
      'text',
      'image',
    ])
    expect(normalizeInputModalities([])).toEqual(['text'])
    expect(normalizeInputModalities(null)).toEqual(['text'])
  })

  it('drops values DSH does not accept', () => {
    expect(normalizeInputModalities(['text', 'audio'])).toEqual(['text'])
  })

  it('prefers an explicit declaration over the registry', () => {
    // A stored text-only declaration (e.g. a gateway without vision) wins.
    expect(resolveInputModalities('deepseek-flash', ['text'])).toEqual(['text'])
    expect(
      resolveInputModalities('deepseek-v4-pro', ['text', 'image'])
    ).toEqual(['text', 'image'])
  })

  it('falls back to the registry alias metadata, then to text', () => {
    expect(resolveInputModalities('deepseek-flash')).toEqual(['text', 'image'])
    expect(resolveInputModalities('deepseek-v4-pro')).toEqual(['text'])
    expect(resolveInputModalities('totally-unknown-model')).toEqual(['text'])
  })
})

describe('draftToModel / modelToDraft', () => {
  it('converts drafts back to the stored model shape', () => {
    expect(
      draftToModel({
        id: ' deepseek-v4-pro ',
        name: ' DeepSeek V4 Pro ',
        contextWindow: '1000000',
        maxTokens: '',
        input: ['text', 'image'],
        inputTouched: true,
        base: null,
      })
    ).toEqual({
      id: 'deepseek-v4-pro',
      name: 'DeepSeek V4 Pro',
      contextWindow: 1000000,
      maxTokens: null,
      input: ['text', 'image'],
    })
  })

  it('never writes an unsupported or text-less modality list', () => {
    const model = draftToModel({
      id: 'custom',
      name: '',
      contextWindow: '',
      maxTokens: '',
      input: ['image', 'audio'],
      inputTouched: true,
      base: null,
    })
    expect(model.input).toEqual(['text', 'image'])
  })

  it('round-trips a stored model through a draft', () => {
    const model = {
      id: 'deepseek-v4.1-flash',
      name: 'DeepSeek V4.1 Flash',
      contextWindow: 1000000,
      maxTokens: 384000,
      input: ['text', 'image'],
    }
    expect(draftToModel(modelToDraft(model))).toEqual(model)
  })

  it('fills a missing input declaration from the registry', () => {
    const draft = modelToDraft({
      id: 'deepseek-flash',
      name: 'DeepSeek Flash',
      contextWindow: 1000000,
      maxTokens: 384000,
    })
    expect(draft.input).toEqual(['text', 'image'])
    expect(draft.inputTouched).toBe(false)
  })

  it('keeps an explicit stored declaration even when the registry differs', () => {
    const draft = modelToDraft({
      id: 'deepseek-flash',
      input: ['text'],
    })
    expect(draft.input).toEqual(['text'])
    expect(draft.inputTouched).toBe(true)
  })
})
