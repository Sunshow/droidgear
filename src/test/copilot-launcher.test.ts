import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { spawnSync } from 'node:child_process'
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { delimiter, join, resolve } from 'node:path'

// Exercise the real launcher with a fake CLI; no network or user config is used.
describe('Copilot local launcher', () => {
  let directory: string
  let configPath: string
  let entryPath: string
  const launcher = resolve('scripts/copilot-local.js')

  beforeEach(() => {
    directory = mkdtempSync(join(tmpdir(), 'copilot launcher '))
    configPath = join(directory, 'config.env')
    entryPath = join(directory, 'copilot entry.cjs')
    writeFileSync(
      entryPath,
      `console.log(JSON.stringify({
        args: process.argv.slice(2),
        env: Object.fromEntries(Object.entries(process.env).filter(([key]) =>
          key.startsWith('COPILOT_') && key !== 'COPILOT_ENTRY' && key !== 'COPILOT_CONFIG_FILE'))
      })); process.exitCode = Number(process.env.COPILOT_TEST_EXIT || 0);`
    )
  })

  afterEach(() => rmSync(directory, { recursive: true, force: true }))

  function run(args: string[] = [], extraEnv: Record<string, string> = {}) {
    return spawnSync(process.execPath, [launcher, ...args], {
      encoding: 'utf8',
      env: {
        ...process.env,
        NODE_EXE: process.execPath,
        COPILOT_CONFIG_FILE: configPath,
        COPILOT_ENTRY: entryPath,
        COPILOT_MODEL: 'stale-model',
        COPILOT_PROVIDER_BASE_URL: 'https://stale.invalid',
        COPILOT_PROVIDER_TYPE: 'stale-type',
        COPILOT_PROVIDER_API_KEY: 'stale-key',
        COPILOT_PROVIDER_MAX_PROMPT_TOKENS: '99999',
        COPILOT_PROVIDER_MAX_OUTPUT_TOKENS: '88888',
        COPILOT_AUTH_TOKEN: 'test-official-token',
        COPILOT_PROVIDER_MODEL_ID: 'stale-model-id',
        COPILOT_PROVIDER_WIRE_MODEL: 'stale-wire-model',
        COPILOT_PROVIDER_BEARER_TOKEN: 'stale-bearer-token',
        COPILOT_PROVIDER_API_KEY_COMMAND: 'stale-command',
        COPILOT_PROVIDER_HEADERS: 'X-Stale: stale',
        COPILOT_PROVIDER_WIRE_API: 'responses',
        COPILOT_PROVIDER_TRANSPORT: 'websockets',
        ...extraEnv,
      },
    })
  }

  function byokConfig() {
    const text = [
      '# Comments and CRLF are accepted',
      'COPILOT_OFFLINE=true',
      'COPILOT_PROVIDER_BASE_URL=https://api.example.test/v1',
      'COPILOT_PROVIDER_API_KEY=literal=!#$value\'"&=tail',
      'COPILOT_MODEL=first-model',
      'COPILOT_MODEL=last-model',
      '',
    ].join('\r\n')
    writeFileSync(configPath, text)
    return text
  }

  it('isolates BYOK values, preserves literals and forwards argument boundaries', () => {
    const original = byokConfig()
    const result = run([
      '-m',
      'override/model',
      '--',
      '--prompt',
      'two words',
      'a&b',
    ])
    expect(result.stderr).toBe('')
    expect(result.status).toBe(0)
    expect(JSON.parse(result.stdout)).toEqual({
      args: ['--model', 'override/model', '--prompt', 'two words', 'a&b'],
      env: {
        COPILOT_OFFLINE: 'true',
        COPILOT_PROVIDER_BASE_URL: 'https://api.example.test/v1',
        COPILOT_PROVIDER_TYPE: 'openai',
        COPILOT_PROVIDER_API_KEY: 'literal=!#$value\'"&=tail',
        COPILOT_MODEL: 'override/model',
      },
    })
    expect(readFileSync(configPath, 'utf8')).toBe(original)
  })

  it('uses the last setting and does not reveal the API key in configuration output', () => {
    byokConfig()
    const child = JSON.parse(run().stdout)
    expect(child.env.COPILOT_MODEL).toBe('last-model')
    expect(child.args).toEqual(['--model', 'last-model'])
    const result = run(['--config'])
    expect(result.status).toBe(0)
    expect(result.stdout).toContain('COPILOT_PROVIDER_API_KEY=(set; hidden)')
    expect(result.stdout).not.toContain('literal=')
    expect(result.stdout).not.toContain('stale-key')
  })

  it('keeps official authentication while removing inherited BYOK settings', () => {
    writeFileSync(configPath, 'COPILOT_OFFLINE=false\n')
    const result = run(['--model', 'official-model'])
    expect(result.status).toBe(0)
    expect(JSON.parse(result.stdout)).toEqual({
      args: ['--model', 'official-model'],
      env: { COPILOT_AUTH_TOKEN: 'test-official-token' },
    })
  })

  it.each([
    ['openai', 'https://channel.test', 'https://channel.test/v1'],
    ['openai', 'https://channel.test/v1/', 'https://channel.test/v1'],
    ['anthropic', 'https://channel.test/v1', 'https://channel.test'],
    [
      'anthropic',
      'https://channel.test/gateway/v1/',
      'https://channel.test/gateway',
    ],
    [
      'openai',
      'https://channel.test/gateway?route=/',
      'https://channel.test/gateway/v1?route=/',
    ],
  ])(
    'normalizes %s endpoint %s to %s at launch',
    (provider, input, expected) => {
      const config =
        byokConfig() +
        `\nCOPILOT_PROVIDER_TYPE=${provider}\nCOPILOT_PROVIDER_BASE_URL=${input}\n`
      writeFileSync(configPath, config)
      const result = run()
      expect(result.status).toBe(0)
      expect(JSON.parse(result.stdout).env.COPILOT_PROVIDER_BASE_URL).toBe(
        expected
      )
      expect(readFileSync(configPath, 'utf8')).toBe(config)
    }
  )

  it.each([['--model', 'forwarded-model'], ['--model=forwarded-model']])(
    'honors a forwarded model flag %s without supplying a conflicting flag',
    (...args) => {
      byokConfig()
      const result = run(['-m', 'wrapper-model', '--', ...args])
      expect(result.status).toBe(0)
      const child = JSON.parse(result.stdout)
      expect(child.args).toEqual(args)
      expect(child.env.COPILOT_MODEL).toBe('forwarded-model')
    }
  )

  it('rejects unsupported providers and empty forwarded model flags', () => {
    const config = byokConfig()
    expect(run(['--', '--model']).status).toBe(1)
    expect(run(['--', '--model=']).status).toBe(1)
    expect(run(['--', '--model', '--version']).status).toBe(1)
    writeFileSync(configPath, config + '\nCOPILOT_PROVIDER_TYPE=azure\n')
    expect(run().stderr).toContain('OpenAI and Anthropic')
  })

  it('rejects incomplete profiles even when old credentials are inherited', () => {
    writeFileSync(configPath, 'COPILOT_OFFLINE=true\nCOPILOT_MODEL=test\n')
    const result = run()
    expect(result.status).toBe(1)
    expect(result.stdout).toBe('')
    expect(result.stderr).toContain('COPILOT_PROVIDER_API_KEY')
    expect(run(['--model']).status).toBe(1)
  })

  it('propagates the CLI exit code and offers help without a configuration', () => {
    expect(run(['--help']).status).toBe(0)
    expect(run().stderr).toContain('Configuration not found')
    byokConfig()
    expect(run([], { COPILOT_TEST_EXIT: '7' }).status).toBe(7)
  })

  it.skipIf(process.platform === 'win32')(
    'finds native Copilot installations on PATH without npm',
    () => {
      byokConfig()
      writeFileSync(
        join(directory, 'copilot'),
        `#!/usr/bin/env node\n${readFileSync(entryPath, 'utf8')}`,
        { mode: 0o700 }
      )
      const result = run(['--', '--version'], {
        COPILOT_ENTRY: '',
        PATH: `${directory}${delimiter}${process.env.PATH || ''}`,
      })
      expect(result.status).toBe(0)
      expect(JSON.parse(result.stdout).args).toEqual([
        '--model',
        'last-model',
        '--version',
      ])
      expect(JSON.parse(result.stdout).env.COPILOT_MODEL).toBe('last-model')
    }
  )
})
