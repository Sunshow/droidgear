import fs from 'node:fs/promises'
import path from 'node:path'
import process from 'node:process'
import { fileURLToPath } from 'node:url'
import prettier from 'prettier'

const PACKAGE_NAME = '@earendil-works/pi-ai'
const DEFAULT_VERSION = 'latest'
const EXTENDED_LEVELS = ['xhigh', 'max']
const STANDARD_LEVELS = ['low', 'medium', 'high']

const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const registryPath = path.join(rootDir, 'src/lib/model-registry-data.json')

function parseArgs(args) {
  let version = DEFAULT_VERSION
  let check = false
  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index]
    if (arg === '--check') {
      check = true
    } else if (arg === '--version') {
      version = args[index + 1] ?? ''
      index += 1
      if (!version) throw new Error('--version requires a value')
    } else {
      throw new Error(`Unknown argument: ${arg}`)
    }
  }
  return { version, check }
}

async function fetchJson(url) {
  try {
    const response = await globalThis.fetch(url)
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`)
    }
    return response.json()
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    throw new Error(`Failed to fetch ${url}: ${message}`, { cause: error })
  }
}

async function loadPiCatalog(version) {
  const packageBase = `https://unpkg.com/${PACKAGE_NAME}@${version}`
  const metadata = await fetchJson(`${packageBase}/?meta`)
  const dataFiles = metadata.files
    .map(file => file.path)
    .filter(filePath => /^\/dist\/providers\/data\/[^/]+\.json$/.test(filePath))
    .filter(filePath => !filePath.endsWith('/manifest.json'))

  if (dataFiles.length === 0) {
    throw new Error(`No Pi model catalog shards found for ${PACKAGE_NAME}`)
  }

  const shards = []
  for (const filePath of dataFiles) {
    shards.push(await fetchJson(`${packageBase}${filePath}`))
  }
  const models = []
  for (const shard of shards) {
    for (const apiModels of Object.values(shard)) {
      if (!apiModels || typeof apiModels !== 'object') continue
      for (const model of Object.values(apiModels)) {
        if (model?.id) models.push(model)
      }
    }
  }

  return { models, version: metadata.version ?? version }
}

function indexCatalog(models) {
  const byId = new Map()
  for (const model of models) {
    const id = model.id.toLowerCase()
    const matches = byId.get(id) ?? []
    matches.push(model)
    byId.set(id, matches)
  }
  return byId
}

function matchingModels(entry, catalogById) {
  const matches = []
  const seen = new Set()
  for (const id of [entry.id, ...entry.aliases]) {
    for (const model of catalogById.get(id.toLowerCase()) ?? []) {
      const key = `${model.provider}/${model.api}/${model.id}`
      if (seen.has(key)) continue
      seen.add(key)
      matches.push(model)
    }
  }
  return matches
}

function hasIdentityMapping(models, level) {
  return models.some(model => model.thinkingLevelMap?.[level] === level)
}

function buildThinkingLevelMap(entry, matches) {
  if (!entry.reasoning) return undefined

  const efforts = new Set(entry.reasoningConfig?.efforts ?? [])
  const map = {}

  if (!efforts.has('none')) map.off = null

  // `minimal` is not part of DroidGear's generic effort model. Only expose it
  // when Pi has direct evidence that the underlying model accepts it natively.
  if (!hasIdentityMapping(matches, 'minimal')) map.minimal = null

  for (const level of STANDARD_LEVELS) {
    if (!efforts.has(level)) map[level] = null
  }

  for (const level of EXTENDED_LEVELS) {
    const catalogSupportsLevel =
      matches.length === 0 || hasIdentityMapping(matches, level)
    if (efforts.has(level) && catalogSupportsLevel) map[level] = level
  }

  return Object.keys(map).length > 0 ? map : undefined
}

function updateRegistry(registry, catalogById) {
  let matched = 0
  let mapped = 0
  const unmatched = []

  const updated = registry.map(entry => {
    const matches = matchingModels(entry, catalogById)
    if (matches.length > 0) matched += 1
    else unmatched.push(entry.id)

    const thinkingLevelMap = buildThinkingLevelMap(entry, matches)
    if (thinkingLevelMap) mapped += 1

    const {
      id,
      name,
      aliases,
      platform,
      reasoning,
      input,
      thinkingLevelMap: _oldThinkingLevelMap,
      ...rest
    } = entry

    return {
      id,
      name,
      aliases,
      platform,
      reasoning,
      input,
      ...(thinkingLevelMap ? { thinkingLevelMap } : {}),
      ...rest,
    }
  })

  return { updated, matched, mapped, unmatched }
}

async function main() {
  const options = parseArgs(process.argv.slice(2))
  const source = await fs.readFile(registryPath, 'utf8')
  const registry = JSON.parse(source)
  const catalog = await loadPiCatalog(options.version)
  const result = updateRegistry(registry, indexCatalog(catalog.models))
  const prettierConfig = (await prettier.resolveConfig(registryPath)) ?? {}
  const output = await prettier.format(
    JSON.stringify(result.updated, null, 2),
    {
      ...prettierConfig,
      parser: 'json',
    }
  )

  console.log(
    `Pi AI ${catalog.version}: matched ${result.matched}/${registry.length} registry models; generated ${result.mapped} thinking maps`
  )
  if (result.unmatched.length > 0) {
    console.log(`No Pi catalog match: ${result.unmatched.join(', ')}`)
  }

  if (options.check) {
    if (output !== source) {
      throw new Error(
        'Pi thinking-level metadata is stale; run npm run models:sync-pi-thinking'
      )
    }
    return
  }

  await fs.writeFile(registryPath, output)
}

main().catch(error => {
  console.error(error instanceof Error ? error.message : error)
  process.exitCode = 1
})
