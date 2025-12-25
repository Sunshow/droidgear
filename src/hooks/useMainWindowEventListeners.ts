import { useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useCommandContext } from './use-command-context'
import { useKeyboardShortcuts } from './use-keyboard-shortcuts'
import { useUIStore } from '@/store/ui-store'
import { useModelStore } from '@/store/model-store'
import { logger } from '@/lib/logger'

/**
 * Main window event listeners - handles global keyboard shortcuts and cross-window events.
 *
 * This hook composes specialized hooks for different event types:
 * - useKeyboardShortcuts: Global keyboard shortcuts (Cmd+, Cmd+1, Cmd+2)
 * - Quick pane submit listener: Cross-window communication from quick pane
 * - Close request listener: Unsaved changes protection
 */
export function useMainWindowEventListeners() {
  const commandContext = useCommandContext()

  useKeyboardShortcuts(commandContext)

  // Listen for quick pane submissions (cross-window event)
  useEffect(() => {
    let isMounted = true
    let unlisten: (() => void) | null = null

    listen<{ text: string }>('quick-pane-submit', event => {
      logger.debug('Quick pane submit event received', {
        text: event.payload.text,
      })
      const { setLastQuickPaneEntry } = useUIStore.getState()
      setLastQuickPaneEntry(event.payload.text)
    })
      .then(unlistenFn => {
        if (!isMounted) {
          unlistenFn()
        } else {
          unlisten = unlistenFn
        }
      })
      .catch(error => {
        logger.error('Failed to setup quick-pane-submit listener', { error })
      })

    return () => {
      isMounted = false
      if (unlisten) {
        unlisten()
      }
    }
  }, [])

  // Listen for window close request - check for unsaved changes
  useEffect(() => {
    let isMounted = true
    let unlisten: (() => void) | null = null

    const appWindow = getCurrentWindow()

    appWindow
      .onCloseRequested(async event => {
        const { hasChanges } = useModelStore.getState()

        if (hasChanges) {
          // Prevent default close behavior
          event.preventDefault()

          // Show confirmation dialog
          const shouldClose = confirm(
            'You have unsaved changes. Are you sure you want to exit?'
          )

          if (shouldClose) {
            // User confirmed, close the window
            await appWindow.destroy()
          }
        }
      })
      .then(unlistenFn => {
        if (!isMounted) {
          unlistenFn()
        } else {
          unlisten = unlistenFn
        }
      })
      .catch(error => {
        logger.error('Failed to setup close request listener', { error })
      })

    return () => {
      isMounted = false
      if (unlisten) {
        unlisten()
      }
    }
  }, [])
}
