import { commands } from '@/lib/bindings'
import { logger } from '@/lib/logger'

/**
 * Global shell environment cache.
 * Fetches shell environment variables once and caches them for all terminals.
 * This avoids the expensive operation of spawning a login shell for each terminal.
 */

let cachedEnv: Record<string, string> | null = null
let fetchPromise: Promise<Record<string, string>> | null = null

/**
 * Get shell environment variables (cached).
 * Only fetches once, subsequent calls return cached result.
 */
export async function getShellEnv(): Promise<Record<string, string>> {
  // Return cached result if available
  if (cachedEnv !== null) {
    return cachedEnv
  }

  // If already fetching, wait for that promise
  if (fetchPromise !== null) {
    return fetchPromise
  }

  // Start fetching
  fetchPromise = fetchShellEnv()

  try {
    cachedEnv = await fetchPromise
    return cachedEnv
  } finally {
    fetchPromise = null
  }
}

async function fetchShellEnv(): Promise<Record<string, string>> {
  logger.debug('Fetching shell environment variables (one-time)')

  try {
    const result = await commands.getShellEnv()

    if (result.status === 'ok') {
      if (result.data == null) {
        logger.warn('Shell env returned null/undefined data')
        return {}
      }
      // Convert Partial<Record> to Record by filtering out undefined values
      const env: Record<string, string> = {}
      for (const [key, value] of Object.entries(result.data)) {
        if (value !== undefined) {
          env[key] = value
        }
      }
      logger.info('Shell environment cached', {
        variableCount: Object.keys(env).length,
      })
      return env
    } else {
      logger.error('Failed to get shell env', { error: result.error })
      return {}
    }
  } catch (error) {
    logger.error('getShellEnv exception', { error })
    return {}
  }
}

/**
 * Preload shell environment at app startup.
 * Call this early to avoid delay when first terminal is created.
 */
export function preloadShellEnv(): Promise<void> {
  return getShellEnv()
    .then(() => {
      /* discard resolved value */
    })
    .catch(() => {
      // Error already logged in fetchShellEnv
    })
}

/**
 * Clear the cached environment (for testing or refresh).
 */
export function clearShellEnvCache(): void {
  cachedEnv = null
  fetchPromise = null
}
