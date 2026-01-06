import { useEffect } from 'react'
import { check } from '@tauri-apps/plugin-updater'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { toast } from 'sonner'
import { initializeCommandSystem } from './lib/commands'
import { buildAppMenu, setupMenuLanguageListener } from './lib/menu'
import { initializeLanguage } from './i18n/language-init'
import i18n from './i18n/config'
import { logger } from './lib/logger'
import { cleanupOldFiles } from './lib/recovery'
import { commands } from './lib/tauri-bindings'
import { useUIStore } from './store/ui-store'
import './App.css'
import { MainWindow } from './components/layout/MainWindow'
import { ThemeProvider } from './components/ThemeProvider'
import { ErrorBoundary } from './components/ErrorBoundary'
import { LegacyConfigDialog } from './components/LegacyConfigDialog'

/**
 * Show main window after frontend is ready
 */
async function showMainWindow() {
  try {
    const mainWindow = getCurrentWindow()
    await mainWindow.show()
    await mainWindow.setFocus()
  } catch (error) {
    logger.warn('Failed to show main window', { error })
  }
}

function App() {
  // Initialize command system and cleanup on app startup
  useEffect(() => {
    logger.info('🚀 Frontend application starting up')
    initializeCommandSystem()
    logger.debug('Command system initialized')

    // Initialize language based on saved preference or system locale
    const initLanguageAndMenu = async () => {
      try {
        // Load preferences to get saved language
        const result = await commands.loadPreferences()
        const savedLanguage =
          result.status === 'ok' ? result.data.language : null

        // Initialize language (will use system locale if no preference)
        await initializeLanguage(savedLanguage)

        // Build the application menu with the initialized language
        await buildAppMenu()
        logger.debug('Application menu built')
        setupMenuLanguageListener()
      } catch (error) {
        logger.warn('Failed to initialize language or menu', { error })
      }

      // Show main window after initialization
      await showMainWindow()
    }

    initLanguageAndMenu()

    // Clean up old recovery files on startup
    cleanupOldFiles().catch(error => {
      logger.warn('Failed to cleanup old recovery files', { error })
    })

    // Example of logging with context
    logger.info('App environment', {
      isDev: import.meta.env.DEV,
      mode: import.meta.env.MODE,
    })

    // Auto-updater logic - check for updates 5 seconds after app loads
    const checkForUpdates = async () => {
      try {
        const update = await check()
        if (update) {
          logger.info(`Update available: ${update.version}`)

          // Store update info for GeneralPane to display
          useUIStore.getState().setPendingUpdate({
            version: update.version,
            body: update.body ?? undefined,
          })

          // Show non-blocking toast notification with action to view details
          toast.info(
            i18n.t('update.availableNotification', { version: update.version }),
            {
              duration: Infinity,
              action: {
                label: i18n.t('update.viewDetails'),
                onClick: () => {
                  useUIStore.getState().setPreferencesOpen(true)
                },
              },
            }
          )
        }
      } catch (checkError) {
        logger.error(`Update check failed: ${String(checkError)}`)
        // Silent fail for update checks - don't bother user with network issues
      }
    }

    // Check for updates 5 seconds after app loads
    const updateTimer = setTimeout(checkForUpdates, 5000)

    // Check for updates every hour (3600000ms = 1 hour)
    const hourlyUpdateInterval = setInterval(checkForUpdates, 3600000)

    return () => {
      clearTimeout(updateTimer)
      clearInterval(hourlyUpdateInterval)
    }
  }, [])

  return (
    <ErrorBoundary>
      <ThemeProvider>
        <MainWindow />
        <LegacyConfigDialog />
      </ThemeProvider>
    </ErrorBoundary>
  )
}

export default App
