import { describe, expect, it } from 'vitest'
import type { CustomModel } from '@/lib/bindings'
import {
  draftFromChannelModel,
  draftToModel,
  modelToDraft,
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
      expect(draft.maxTokens).toBe('384000')
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
        base: null,
      })
    ).toEqual({
      id: 'deepseek-v4-pro',
      name: 'DeepSeek V4 Pro',
      contextWindow: 1000000,
      maxTokens: null,
    })
  })

  it('round-trips a stored model through a draft', () => {
    const model = {
      id: 'deepseek-v4.1-flash',
      name: 'DeepSeek V4.1 Flash',
      contextWindow: 1000000,
      maxTokens: 384000,
    }
    expect(draftToModel(modelToDraft(model))).toEqual(model)
  })
})
