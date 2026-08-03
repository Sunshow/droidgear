import type { ClaudeTemporaryRunDebugPreview } from '@/lib/bindings'

/**
 * Formats a temporary-run debug preview as plain text for display.
 *
 * Only secret env *keys* are shown, never their values. Keep this in sync
 * with the TUI formatter (`droidgear-tui/src/tui/utils.rs`,
 * `format_claude_temporary_run_preview`).
 */
export function formatClaudePreview(
  preview: ClaudeTemporaryRunDebugPreview
): string {
  const lines: string[] = []

  lines.push(`Launcher: ${preview.program} ${preview.args.join(' ')}`.trim())
  const childArgs = preview.childArgs.join(' ')
  lines.push(`Command: ${preview.childProgram} ${childArgs}`.trim())
  lines.push(`Config dir: ${preview.liveConfigDir}`)
  if (preview.inheritedEnvFileSource) {
    lines.push(`Inherited env file: ${preview.inheritedEnvFileSource}`)
  }

  if (preview.env.length > 0) {
    lines.push('')
    lines.push('Environment:')
    for (const [key, value] of preview.env) {
      lines.push(`  ${key}=${value}`)
    }
  }

  if (preview.unsetEnv.length > 0) {
    lines.push('')
    lines.push(`Unset environment: ${preview.unsetEnv.join(', ')}`)
  }

  if (preview.secretEnvKeys.length > 0) {
    lines.push('')
    lines.push(
      `Secret environment (values hidden): ${preview.secretEnvKeys.join(', ')}`
    )
  }

  if (preview.warnings.length > 0) {
    lines.push('')
    lines.push('Warnings:')
    for (const warning of preview.warnings) {
      lines.push(`  - ${warning}`)
    }
  }

  lines.push('')
  lines.push('Settings overlay:')
  lines.push(preview.settingsOverlayJson)

  return lines.join('\n')
}
