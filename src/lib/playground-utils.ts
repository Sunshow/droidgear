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

/** Builds the default name the save dialog proposes for a generated file.
 *
 * The extension comes from the source (a downloaded path or provider URL) when
 * it has one, because that is the format the file was actually written in;
 * otherwise the expected extension for the protocol is used.
 */
export function mediaFileName(
  base: string,
  source: string | null | undefined,
  fallbackExtension: string
): string {
  const withoutQuery = source?.split(/[?#]/)[0]?.trim() ?? ''
  const name = withoutQuery.split(/[/\\]/).pop() ?? ''
  const extension = /\.([a-z0-9]{1,8})$/i.exec(name)?.[1]?.toLowerCase()
  return `${base}.${extension ?? fallbackExtension}`
}
