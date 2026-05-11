import type { AppCommand } from './types'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { exit } from '@tauri-apps/plugin-process'
import i18n from '@/i18n/config'
import { useModelStore } from '@/store/model-store'
import { useUIStore } from '@/store/ui-store'

export const windowCommands: AppCommand[] = [
  {
    id: 'window-close',
    labelKey: 'commands.windowClose.label',
    descriptionKey: 'commands.windowClose.description',
    shortcut: '⌘+W',

    execute: async _context => {
      try {
        const { hasChanges } = useModelStore.getState()

        if (hasChanges) {
          // Show the styled AlertDialog; do NOT close yet
          useUIStore.getState().setCloseConfirmOpen(true)
          return
        }

        // No unsaved changes: close immediately
        const appWindow = getCurrentWindow()
        await appWindow.close()
      } catch (error) {
        console.error('[window-close] close() failed:', error)
        // Fallback: try destroy()
        try {
          const appWindow = getCurrentWindow()
          await appWindow.destroy()
        } catch (destroyError) {
          console.error('[window-close] destroy() also failed:', destroyError)
          // Last resort: exit the process
          await exit(0)
        }
      }
    },
  },

  {
    id: 'window-minimize',
    labelKey: 'commands.windowMinimize.label',
    descriptionKey: 'commands.windowMinimize.description',
    shortcut: '⌘+M',

    execute: async context => {
      try {
        const appWindow = getCurrentWindow()
        await appWindow.minimize()
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Unknown error'
        context.showToast(
          i18n.t('toast.error.windowMinimizeFailed', { message }),
          'error'
        )
      }
    },
  },

  {
    id: 'window-toggle-maximize',
    labelKey: 'commands.windowToggleMaximize.label',
    descriptionKey: 'commands.windowToggleMaximize.description',

    execute: async context => {
      try {
        const appWindow = getCurrentWindow()
        await appWindow.toggleMaximize()
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Unknown error'
        context.showToast(
          i18n.t('toast.error.windowMaximizeFailed', { message }),
          'error'
        )
      }
    },
  },

  {
    id: 'window-fullscreen',
    labelKey: 'commands.windowFullscreen.label',
    descriptionKey: 'commands.windowFullscreen.description',
    shortcut: 'F11',

    execute: async context => {
      try {
        const appWindow = getCurrentWindow()
        await appWindow.setFullscreen(true)
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Unknown error'
        context.showToast(
          i18n.t('toast.error.fullscreenEnterFailed', { message }),
          'error'
        )
      }
    },
  },

  {
    id: 'window-exit-fullscreen',
    labelKey: 'commands.windowExitFullscreen.label',
    descriptionKey: 'commands.windowExitFullscreen.description',
    shortcut: 'Escape',

    execute: async context => {
      try {
        const appWindow = getCurrentWindow()
        await appWindow.setFullscreen(false)
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Unknown error'
        context.showToast(
          i18n.t('toast.error.fullscreenExitFailed', { message }),
          'error'
        )
      }
    },
  },
]
