/** Derive the DSH pi-ai credential reference from a provider id. */
export function envNameForProviderId(providerId: string): string {
  const trimmed = providerId.trim()
  if (!trimmed) return ''
  return `${trimmed.toUpperCase().replace(/[^A-Z0-9]+/g, '_')}_API_KEY`
}

/**
 * The reference DSH resolves for a route: an explicit override when set,
 * otherwise the name derived from the provider id.
 */
export function resolveApiKeyEnv(
  providerId: string,
  custom: string | null | undefined
): string {
  return (custom ?? '').trim() || envNameForProviderId(providerId)
}

/** References DSH accepts: ASCII letters, digits, underscores, and hyphens. */
const VALID_REFERENCE = /^[A-Za-z0-9_-]+$/

/** Light frontend check for a custom reference (the backend validates too). */
export function isValidApiKeyEnv(reference: string): boolean {
  return VALID_REFERENCE.test(reference.trim())
}

/**
 * Whether a stored reference is a custom override rather than the derived
 * name (empty stored references are the derived default).
 */
export function isCustomApiKeyEnv(
  providerId: string,
  stored: string | null | undefined
): boolean {
  const trimmed = (stored ?? '').trim()
  return trimmed.length > 0 && trimmed !== envNameForProviderId(providerId)
}
