import { useEffect } from 'react'
import type { CommandContext } from '@/lib/commands/types'

/**
 * Handles global keyboard shortcuts for the application.
 *
 * Currently handles:
 * - Cmd/Ctrl+, : Open preferences
 */
export function useKeyboardShortcuts(commandContext: CommandContext) {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.metaKey || e.ctrlKey) {
        switch (e.key) {
          case ',': {
            e.preventDefault()
            commandContext.openPreferences()
            break
          }
        }
      }
    }

    document.addEventListener('keydown', handleKeyDown)
    return () => document.removeEventListener('keydown', handleKeyDown)
  }, [commandContext])
}
