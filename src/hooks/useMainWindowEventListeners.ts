import { useCommandContext } from './use-command-context'
import { useKeyboardShortcuts } from './use-keyboard-shortcuts'

/**
 * Main window event listeners - handles global keyboard shortcuts.
 *
 * Window close is handled directly in the `window-close` command and
 * the AlertDialog confirm handler, both using destroy() to bypass the
 * unreliable async onCloseRequested event.
 */
export function useMainWindowEventListeners() {
  const commandContext = useCommandContext()

  useKeyboardShortcuts(commandContext)
}
