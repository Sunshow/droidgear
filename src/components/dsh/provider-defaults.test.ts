import { describe, expect, it } from 'vitest'
import type { DshConfigSnapshot } from '@/lib/bindings'
import { makeEntry, makeField, makeSnapshot } from './dsh-test-fixtures'
import {
  modelsRejectingLevel,
  readDefaultModel,
  suggestDefaultReasoning,
  supportedReasoningLevels,
} from './provider-defaults'

function reasoningModel(
  id: string,
  levels: string[] | null
): {
  id: string
  base: { id: string; reasoningEfforts: Record<string, string | null> } | null
} {
  return {
    id,
    base: levels
      ? {
          id,
          reasoningEfforts: Object.fromEntries(
            levels.map(level => [level, level === 'off' ? null : level])
          ),
        }
      : null,
  }
}

describe('supportedReasoningLevels', () => {
  it('prefers the provider-fetched reasoningEfforts map', () => {
    expect(
      supportedReasoningLevels(reasoningModel('custom-model', ['off', 'high']))
    ).toEqual(['off', 'high'])
  })

  it('falls back to the built-in registry and normalizes none to off', () => {
    const levels = supportedReasoningLevels({ id: 'gpt-5' })
    expect(levels).not.toBeNull()
    expect(levels).toContain('off')
    expect(levels).toContain('high')
    expect(levels).not.toContain('none')
  })

  it('returns null for models with unknown capabilities', () => {
    expect(supportedReasoningLevels({ id: 'totally-unknown-model' })).toBeNull()
    expect(supportedReasoningLevels({ id: '' })).toBeNull()
  })
})

describe('suggestDefaultReasoning', () => {
  it('suggests high when every model supports it', () => {
    expect(
      suggestDefaultReasoning([
        reasoningModel('model-a', ['off', 'low', 'high']),
        reasoningModel('model-b', ['high', 'max']),
      ])
    ).toBe('high')
  })

  it('uses registry capabilities for known model ids', () => {
    expect(suggestDefaultReasoning([{ id: 'gpt-5' }])).toBe('high')
  })

  it('does not suggest when a model lacks high', () => {
    expect(
      suggestDefaultReasoning([
        reasoningModel('model-a', ['high']),
        reasoningModel('model-b', ['low', 'medium']),
      ])
    ).toBeNull()
  })

  it('does not suggest when a model capability is unknown', () => {
    expect(
      suggestDefaultReasoning([
        reasoningModel('model-a', ['high']),
        { id: 'totally-unknown-model' },
      ])
    ).toBeNull()
  })

  it('does not suggest without models', () => {
    expect(suggestDefaultReasoning([])).toBeNull()
  })
})

describe('modelsRejectingLevel', () => {
  it('lists models known to reject the level', () => {
    expect(
      modelsRejectingLevel(
        [
          reasoningModel('supports-high', ['high']),
          reasoningModel('no-high', ['low', 'medium']),
        ],
        'high'
      )
    ).toEqual(['no-high'])
  })

  it('ignores unknown capabilities and an empty level', () => {
    expect(
      modelsRejectingLevel([{ id: 'totally-unknown-model' }], 'high')
    ).toEqual([])
    expect(modelsRejectingLevel([reasoningModel('a', ['low'])], '')).toEqual([])
  })
})

describe('readDefaultModel', () => {
  function snapshotWithEntry(config: unknown): DshConfigSnapshot {
    return makeSnapshot({
      entries: [
        makeEntry({
          id: 'agent-default-model',
          config: config as DshConfigSnapshot['entries'][number]['config'],
          fields: [
            makeField({ name: 'provider', value: 'p' }),
            makeField({ name: 'model', value: 'm' }),
          ],
        }),
      ],
    })
  }

  it('reads provider, model, and effort from the entry config', () => {
    expect(
      readDefaultModel(
        snapshotWithEntry({
          provider: 'deepseek-official',
          model: 'deepseek-flash',
          reasoningEffort: 'high',
        })
      )
    ).toEqual({
      provider: 'deepseek-official',
      model: 'deepseek-flash',
      reasoningEffort: 'high',
    })
  })

  it('falls back to field values when the config object is absent', () => {
    expect(readDefaultModel(snapshotWithEntry(null))).toEqual({
      provider: 'p',
      model: 'm',
      reasoningEffort: null,
    })
  })

  it('returns null without an entry or a complete selection', () => {
    expect(readDefaultModel(null)).toBeNull()
    expect(readDefaultModel(makeSnapshot())).toBeNull()
    const incomplete = makeSnapshot({
      entries: [
        makeEntry({
          id: 'agent-default-model',
          config: { provider: 'p' },
          fields: [makeField({ name: 'provider', value: 'p' })],
        }),
      ],
    })
    expect(readDefaultModel(incomplete)).toBeNull()
  })
})
