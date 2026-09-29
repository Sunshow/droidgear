#!/usr/bin/env node
// Shared config parsing keeps the Windows and Unix launchers consistent.
import {
  accessSync,
  constants,
  existsSync,
  readFileSync,
  statSync,
} from 'node:fs'
import { homedir } from 'node:os'
import { delimiter, dirname, join, resolve } from 'node:path'
import { fileURLToPath, URL } from 'node:url'
import { execFileSync, spawn } from 'node:child_process'

const providerKeys = [
  'COPILOT_PROVIDER_BASE_URL',
  'COPILOT_PROVIDER_TYPE',
  'COPILOT_PROVIDER_API_KEY',
  'COPILOT_MODEL',
  'COPILOT_PROVIDER_MAX_PROMPT_TOKENS',
  'COPILOT_PROVIDER_MAX_OUTPUT_TOKENS',
]
const configKeys = new Set([
  ...providerKeys,
  'COPILOT_OFFLINE',
  'NODE_EXE',
  'COPILOT_ENTRY',
])
const providerOverrides = [
  'COPILOT_PROVIDER_MODEL_ID',
  'COPILOT_PROVIDER_WIRE_MODEL',
  'COPILOT_PROVIDER_BEARER_TOKEN',
  'COPILOT_PROVIDER_API_KEY_COMMAND',
  'COPILOT_PROVIDER_HEADERS',
  'COPILOT_PROVIDER_WIRE_API',
  'COPILOT_PROVIDER_TRANSPORT',
]

function normalizeBaseUrl(baseUrl, provider) {
  if (!['openai', 'anthropic'].includes(provider)) {
    throw new Error('Copilot supports OpenAI and Anthropic provider types')
  }
  const url = new URL(baseUrl)
  if (!['http:', 'https:'].includes(url.protocol))
    throw new Error('Base URL must use HTTP(S)')
  const path = url.pathname.replace(/\/+$/, '')
  url.pathname =
    provider === 'anthropic'
      ? path.replace(/\/v1$/, '')
      : path.endsWith('/v1')
        ? path
        : `${path}/v1`
  return url.pathname === '/' && !url.search && !url.hash
    ? url.toString().replace(/\/$/, '')
    : url.toString()
}

function readConfig(path) {
  const values = {}
  for (const rawLine of readFileSync(path, 'utf8').split(/\r?\n/)) {
    const line = rawLine.trim()
    if (!line || line.startsWith('#')) continue
    const separator = line.indexOf('=')
    if (separator < 0) continue
    const key = line.slice(0, separator).trim()
    const value = line.slice(separator + 1).trim()
    if (configKeys.has(key)) {
      if (value.includes('\0'))
        throw new Error(`${key} contains a NUL character`)
      values[key] = value
    }
  }
  return values
}

function findLaunchCommand(config) {
  const nodeEntry = entry => ({
    program: config.NODE_EXE || process.env.NODE_EXE || process.execPath,
    args: [resolve(entry)],
  })
  const explicit = config.COPILOT_ENTRY || process.env.COPILOT_ENTRY
  if (explicit) {
    if (!existsSync(explicit)) throw new Error('COPILOT_ENTRY does not exist')
    return nodeEntry(explicit)
  }
  // Homebrew and native installers expose an executable without an npm loader.
  // On Windows, npm's .cmd shim is handled via the loader below, avoiding shell
  // expansion of forwarded arguments.
  const executable = process.platform === 'win32' ? 'copilot.exe' : 'copilot'
  for (const path of (process.env.PATH || '').split(delimiter)) {
    const candidate = join(path.replace(/^"|"$/g, ''), executable)
    try {
      accessSync(candidate, constants.X_OK)
      if (statSync(candidate).isFile())
        return { program: resolve(candidate), args: [] }
    } catch {
      // Continue through PATH, then try npm's global installation.
    }
  }
  let npmRoot = ''
  try {
    npmRoot =
      process.platform === 'win32'
        ? execFileSync('cmd.exe', ['/d', '/s', '/c', 'npm root -g'], {
            encoding: 'utf8',
            stdio: ['ignore', 'pipe', 'ignore'],
          }).trim()
        : execFileSync('npm', ['root', '-g'], {
            encoding: 'utf8',
            stdio: ['ignore', 'pipe', 'ignore'],
          }).trim()
  } catch {
    // The conventional locations also cover installs without npm on PATH.
  }
  const candidates = [
    npmRoot && join(npmRoot, '@github/copilot/npm-loader.js'),
    join(
      homedir(),
      '.npm-global/lib/node_modules/@github/copilot/npm-loader.js'
    ),
    '/usr/local/lib/node_modules/@github/copilot/npm-loader.js',
    '/usr/lib/node_modules/@github/copilot/npm-loader.js',
    process.env.APPDATA &&
      join(
        process.env.APPDATA,
        'npm/node_modules/@github/copilot/npm-loader.js'
      ),
  ]
  const entry = candidates.find(path => path && existsSync(path))
  if (!entry) {
    throw new Error(
      'GitHub Copilot CLI was not found. Install with npm install -g @github/copilot, or set COPILOT_ENTRY.'
    )
  }
  return nodeEntry(entry)
}

function main() {
  const extraArgs = []
  const args = process.argv.slice(2)
  let model
  let showConfig = false
  for (let index = 0; index < args.length; index++) {
    const arg = args[index]
    if (arg === '--') {
      extraArgs.push(...args.slice(index + 1))
      break
    }
    if (arg === '--help' || arg === '-h') {
      console.log(`copilot-local — launch a saved GitHub Copilot configuration

Usage: copilot-local [options] [Copilot arguments]
  --model, -m MODEL  Override the model for this run
  --config          Show the configuration (API key hidden)
  --help, -h        Show this help
  --                Pass remaining arguments directly to Copilot

Reads ~/.droidgear/copilot/config.env by default. Set COPILOT_CONFIG_FILE
for a different file. A config.env beside the wrapper takes precedence
when COPILOT_CONFIG_FILE is unset. Restart Copilot to switch BYOK models.`)
      return
    }
    if (arg === '--model' || arg === '-m') {
      model = args[++index]?.trim()
      if (!model || model.startsWith('-'))
        throw new Error(`Missing model after ${arg}`)
    } else if (arg.startsWith('--model=')) {
      model = arg.slice('--model='.length).trim()
      if (!model) throw new Error('Missing model after --model=')
    } else if (arg === '--config') {
      showConfig = true
    } else {
      extraArgs.push(arg)
    }
  }

  const besideWrapper = join(
    dirname(fileURLToPath(import.meta.url)),
    '..',
    'config.env'
  )
  const configPath =
    process.env.COPILOT_CONFIG_FILE ||
    (existsSync(besideWrapper)
      ? besideWrapper
      : join(homedir(), '.droidgear/copilot/config.env'))
  if (!existsSync(configPath)) {
    throw new Error(
      `Configuration not found: ${configPath}. Apply a Copilot profile in DroidGear first.`
    )
  }
  const config = readConfig(configPath)
  const byok =
    config.COPILOT_OFFLINE !== 'false' &&
    (config.COPILOT_OFFLINE === 'true' || providerKeys.some(key => config[key]))
  const clearKeys = new Set([
    ...providerKeys,
    ...providerOverrides,
    'COPILOT_OFFLINE',
  ])
  if (byok) clearKeys.add('COPILOT_AUTH_TOKEN')
  const env = Object.fromEntries(
    Object.entries(process.env).filter(
      ([key]) =>
        !clearKeys.has(process.platform === 'win32' ? key.toUpperCase() : key)
    )
  )
  if (byok) {
    env.COPILOT_OFFLINE = 'true'
    for (const key of providerKeys) {
      if (config[key]) env[key] = config[key]
    }
    env.COPILOT_PROVIDER_TYPE = (
      env.COPILOT_PROVIDER_TYPE || 'openai'
    ).toLowerCase()
    if (model) env.COPILOT_MODEL = model
    const forwardedModelIndex = extraArgs.findLastIndex(
      arg => arg === '--model' || arg.startsWith('--model=')
    )
    if (forwardedModelIndex >= 0) {
      const arg = extraArgs[forwardedModelIndex]
      const forwardedModel = (
        arg === '--model'
          ? extraArgs[forwardedModelIndex + 1]
          : arg.slice('--model='.length)
      )?.trim()
      if (!forwardedModel || forwardedModel.startsWith('-'))
        throw new Error('Missing model after --model')
      env.COPILOT_MODEL = forwardedModel
    } else if (env.COPILOT_MODEL) {
      extraArgs.unshift('--model', env.COPILOT_MODEL)
    }
    if (env.COPILOT_PROVIDER_BASE_URL) {
      env.COPILOT_PROVIDER_BASE_URL = normalizeBaseUrl(
        env.COPILOT_PROVIDER_BASE_URL,
        env.COPILOT_PROVIDER_TYPE
      )
    }
  } else if (model) {
    extraArgs.push('--model', model)
  }

  if (showConfig) {
    console.log(
      `Config file: ${configPath}\nMode: ${byok ? 'BYOK' : 'official subscription'}`
    )
    for (const key of providerKeys) {
      console.log(
        `${key}=${key === 'COPILOT_PROVIDER_API_KEY' && env[key] ? '(set; hidden)' : env[key] || '(not set)'}`
      )
    }
    return
  }
  if (byok) {
    const required = [
      'COPILOT_PROVIDER_BASE_URL',
      'COPILOT_MODEL',
      'COPILOT_PROVIDER_API_KEY',
    ]
    const missing = required.filter(key => !env[key])
    if (missing.length)
      throw new Error(`Complete the BYOK configuration: ${missing.join(', ')}`)
    for (const key of [
      'COPILOT_PROVIDER_MAX_PROMPT_TOKENS',
      'COPILOT_PROVIDER_MAX_OUTPUT_TOKENS',
    ]) {
      if (env[key] && !/^[1-9][0-9]*$/.test(env[key]))
        throw new Error(`${key} must be a positive integer`)
    }
  }

  const command = findLaunchCommand(config)
  const child = spawn(command.program, [...command.args, ...extraArgs], {
    env,
    stdio: 'inherit',
  })
  child.on('error', error => {
    console.error(`Unable to launch Copilot: ${error.message}`)
    process.exitCode = 1
  })
  child.on('exit', (code, signal) => {
    process.exitCode = code ?? (signal === 'SIGINT' ? 130 : 1)
  })
  // Copilot shares the foreground terminal. Keep the wrapper alive until it exits.
  process.on('SIGINT', () => {
    // The foreground child receives the terminal's SIGINT directly.
  })
}

try {
  main()
} catch (error) {
  console.error(error.message)
  process.exitCode = 1
}
