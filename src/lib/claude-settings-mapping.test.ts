import { describe, it, expect } from 'vitest'
import {
  CLAUDE_AUTH_TOKEN_ENV,
  CLAUDE_BASE_URL_ENV,
  CLAUDE_DISABLE_ADAPTIVE_ENV,
  CLAUDE_DISABLE_THINKING_ENV,
  CLAUDE_EFFORT_ENV,
  CLAUDE_MAX_THINKING_TOKENS_ENV,
  CLAUDE_MODEL_ENV,
  CLAUDE_SMALL_MODEL_ENV,
  getEnvString,
  getReasoningEffort,
  getThinkingMode,
  isSmallModelMirroringMain,
  setEnvString,
  setReasoningEffort,
  setSmallModelMirroring,
  setThinkingMode,
} from './claude-settings-mapping'
import type { ClaudeSettingsDoc } from '@/store/claude-settings-store'

function clone(doc: ClaudeSettingsDoc): ClaudeSettingsDoc {
  return JSON.parse(JSON.stringify(doc)) as ClaudeSettingsDoc
}

describe('setEnvString', () => {
  it('writes a value into env', () => {
    const draft: ClaudeSettingsDoc = {}
    setEnvString(draft, CLAUDE_BASE_URL_ENV, 'https://proxy.example.com')
    expect(draft.env).toEqual({
      [CLAUDE_BASE_URL_ENV]: 'https://proxy.example.com',
    })
  })

  it('trims whitespace before writing', () => {
    const draft: ClaudeSettingsDoc = {}
    setEnvString(draft, CLAUDE_AUTH_TOKEN_ENV, '  token  ')
    expect(getEnvString(draft, CLAUDE_AUTH_TOKEN_ENV)).toBe('token')
  })

  it('removes the key when value is empty and prunes env when last key', () => {
    const draft: ClaudeSettingsDoc = { env: { [CLAUDE_BASE_URL_ENV]: 'x' } }
    setEnvString(draft, CLAUDE_BASE_URL_ENV, '')
    expect(draft.env).toBeUndefined()
  })

  it('keeps env object when other keys remain', () => {
    const draft: ClaudeSettingsDoc = {
      env: { [CLAUDE_BASE_URL_ENV]: 'x', OTHER: 'keep' },
    }
    setEnvString(draft, CLAUDE_BASE_URL_ENV, null)
    expect(draft.env).toEqual({ OTHER: 'keep' })
  })
})

describe('reasoning effort', () => {
  it('returns inherit when no effort env is set', () => {
    expect(getReasoningEffort({})).toBe('inherit')
  })

  it('round-trips low without setting adaptive flag', () => {
    const draft: ClaudeSettingsDoc = {}
    setReasoningEffort(draft, 'low')
    expect(getReasoningEffort(draft)).toBe('low')
    const env = draft.env as Record<string, unknown>
    expect(env[CLAUDE_EFFORT_ENV]).toBe('low')
    expect(env[CLAUDE_DISABLE_ADAPTIVE_ENV]).toBeUndefined()
  })

  it('sets adaptive flag for high', () => {
    const draft: ClaudeSettingsDoc = {}
    setReasoningEffort(draft, 'high')
    const env = draft.env as Record<string, unknown>
    expect(env[CLAUDE_EFFORT_ENV]).toBe('high')
    expect(env[CLAUDE_DISABLE_ADAPTIVE_ENV]).toBe('1')
  })

  it('clears adaptive flag when downgrading from max to medium', () => {
    const draft: ClaudeSettingsDoc = {
      env: {
        [CLAUDE_EFFORT_ENV]: 'max',
        [CLAUDE_DISABLE_ADAPTIVE_ENV]: '1',
      },
    }
    setReasoningEffort(draft, 'medium')
    const env = draft.env as Record<string, unknown>
    expect(env[CLAUDE_EFFORT_ENV]).toBe('medium')
    expect(env[CLAUDE_DISABLE_ADAPTIVE_ENV]).toBeUndefined()
  })

  it('removes effort and adaptive keys when set to inherit', () => {
    const draft: ClaudeSettingsDoc = {
      env: {
        [CLAUDE_EFFORT_ENV]: 'high',
        [CLAUDE_DISABLE_ADAPTIVE_ENV]: '1',
      },
    }
    setReasoningEffort(draft, 'inherit')
    expect(draft.env).toBeUndefined()
  })
})

describe('thinking mode', () => {
  it('reads on from alwaysThinkingEnabled=true', () => {
    expect(getThinkingMode({ alwaysThinkingEnabled: true })).toBe('on')
  })

  it('reads off from alwaysThinkingEnabled=false', () => {
    expect(getThinkingMode({ alwaysThinkingEnabled: false })).toBe('off')
  })

  it('reads off from CLAUDE_CODE_DISABLE_THINKING=1 alone', () => {
    expect(
      getThinkingMode({
        env: { [CLAUDE_DISABLE_THINKING_ENV]: '1' },
      })
    ).toBe('off')
  })

  it('reads inherit when nothing is set', () => {
    expect(getThinkingMode({})).toBe('inherit')
  })

  it('writes on by deleting disable flag and setting alwaysThinkingEnabled', () => {
    const draft: ClaudeSettingsDoc = {
      env: {
        [CLAUDE_DISABLE_THINKING_ENV]: '1',
        [CLAUDE_MAX_THINKING_TOKENS_ENV]: '2048',
      },
    }
    setThinkingMode(draft, 'on')
    expect(draft.alwaysThinkingEnabled).toBe(true)
    expect(draft.env).toBeUndefined()
  })

  it('writes off by setting disable flag and root false', () => {
    const draft: ClaudeSettingsDoc = {}
    setThinkingMode(draft, 'off')
    expect(draft.alwaysThinkingEnabled).toBe(false)
    const env = draft.env as Record<string, unknown>
    expect(env[CLAUDE_DISABLE_THINKING_ENV]).toBe('1')
  })

  it('writes inherit by removing all thinking artifacts', () => {
    const draft: ClaudeSettingsDoc = {
      alwaysThinkingEnabled: true,
      env: { [CLAUDE_DISABLE_THINKING_ENV]: '1' },
    }
    setThinkingMode(draft, 'inherit')
    expect(draft.alwaysThinkingEnabled).toBeUndefined()
    expect(draft.env).toBeUndefined()
  })
})

describe('small model mirroring', () => {
  it('returns true when no model is set', () => {
    expect(isSmallModelMirroringMain({})).toBe(true)
  })

  it('returns true when small model equals main', () => {
    const doc: ClaudeSettingsDoc = {
      env: {
        [CLAUDE_MODEL_ENV]: 'claude-sonnet-4-5',
        [CLAUDE_SMALL_MODEL_ENV]: 'claude-sonnet-4-5',
      },
    }
    expect(isSmallModelMirroringMain(doc)).toBe(true)
  })

  it('returns false when small model differs', () => {
    const doc: ClaudeSettingsDoc = {
      env: {
        [CLAUDE_MODEL_ENV]: 'claude-sonnet-4-5',
        [CLAUDE_SMALL_MODEL_ENV]: 'claude-haiku-4',
      },
    }
    expect(isSmallModelMirroringMain(doc)).toBe(false)
  })

  it('mirroring on copies main into small', () => {
    const draft: ClaudeSettingsDoc = {
      env: { [CLAUDE_MODEL_ENV]: 'claude-sonnet-4-5' },
    }
    setSmallModelMirroring(draft, true, 'claude-sonnet-4-5')
    const env = draft.env as Record<string, unknown>
    expect(env[CLAUDE_SMALL_MODEL_ENV]).toBe('claude-sonnet-4-5')
  })

  it('mirroring off does not touch existing small model', () => {
    const draft: ClaudeSettingsDoc = {
      env: {
        [CLAUDE_MODEL_ENV]: 'claude-sonnet-4-5',
        [CLAUDE_SMALL_MODEL_ENV]: 'claude-haiku-4',
      },
    }
    const before = clone(draft)
    setSmallModelMirroring(draft, false, 'claude-sonnet-4-5')
    expect(draft).toEqual(before)
  })
})
