/** Splits the reference-image textarea into one URL/data URL per line.
 *
 * Commas are intentionally not separators: base64 data URLs contain them.
 */
export function parseImageUrls(value: string): string[] {
  return value
    .split(/\r?\n/)
    .map(item => item.trim())
    .filter(Boolean)
}
