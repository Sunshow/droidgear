import { describe, it, expect } from 'vitest'
import type { ClaudeTemporaryRunDebugPreview } from '@/lib/bindings'
import { formatClaudePreview } from './claude-preview-format'

function makePreview(
  overrides: Partial<ClaudeTemporaryRunDebugPreview> = {}
): ClaudeTemporaryRunDebugPreview {
  return {
    profileId: 'p1',
    profileName: 'work',
    program: '/usr/local/bin/droidgear',
    args: ['__droidgear_internal', 'claude-settings-launcher'],
    childProgram: 'claude',
    childArgs: ['--settings', '/tmp/runtime/claude-settings.json'],
    liveConfigDir: '/tmp/runtime',
    inheritedEnvFileSource: null,
    env: [['ANTHROPIC_MODEL', 'claude-sonnet-4-5']],
    unsetEnv: ['CLAUDE_CONFIG_DIR'],
    secretEnvKeys: ['DROIDGEAR_INTERNAL_CLAUDE_SETTINGS_JSON'],
    warnings: ['old runtime dirs cleaned'],
    settingsOverlayJson: '{\n  "env": {}\n}',
    ...overrides,
  }
}

describe('formatClaudePreview', () => {
  it('shows launcher, command and config dir', () => {
    const text = formatClaudePreview(makePreview())
    expect(text).toContain('/usr/local/bin/droidgear __droidgear_internal')
    expect(text).toContain(
      'claude --settings /tmp/runtime/claude-settings.json'
    )
    expect(text).toContain('Config dir: /tmp/runtime')
  })

  it('lists visible env values', () => {
    const text = formatClaudePreview(makePreview())
    expect(text).toContain('ANTHROPIC_MODEL=claude-sonnet-4-5')
  })

  it('never prints secret env values, only key names', () => {
    const text = formatClaudePreview(makePreview())
    expect(text).toContain('DROIDGEAR_INTERNAL_CLAUDE_SETTINGS_JSON')
    expect(text).toContain('(values hidden)')
    expect(text).not.toContain('secret-value')
  })

  it('lists unset env keys', () => {
    const text = formatClaudePreview(makePreview())
    expect(text).toContain('Unset environment: CLAUDE_CONFIG_DIR')
  })

  it('shows warnings and the settings overlay', () => {
    const text = formatClaudePreview(makePreview())
    expect(text).toContain('old runtime dirs cleaned')
    expect(text).toContain('Settings overlay:')
    expect(text).toContain('"env": {}')
  })

  it('omits optional sections when empty', () => {
    const text = formatClaudePreview(
      makePreview({
        env: [],
        unsetEnv: [],
        secretEnvKeys: [],
        warnings: [],
        inheritedEnvFileSource: null,
      })
    )
    expect(text).not.toContain('Environment:')
    expect(text).not.toContain('Unset environment:')
    expect(text).not.toContain('Secret environment')
    expect(text).not.toContain('Warnings:')
  })

  it('shows inherited env file source when present', () => {
    const text = formatClaudePreview(
      makePreview({ inheritedEnvFileSource: '/home/u/.claude/.env' })
    )
    expect(text).toContain('Inherited env file: /home/u/.claude/.env')
  })
})
