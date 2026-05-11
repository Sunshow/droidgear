import { useEffect } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { initializeCommandSystem } from './lib/commands'
import { buildAppMenu, setupMenuLanguageListener } from './lib/menu'
import { initializeLanguage } from './i18n/language-init'
import { logger } from './lib/logger'
import { cleanupOldFiles } from './lib/recovery'
import { preloadShellEnv } from './services/shell-env'
import {
  checkForUpdate,
  showUpdateNotification,
  isUpdateCheckDisabled,
} from './services/updater'
import { commands } from './lib/tauri-bindings'
import './App.css'
import { MainWindow } from './components/layout/MainWindow'
import { ThemeProvider } from './components/ThemeProvider'
import { ErrorBoundary } from './components/ErrorBoundary'
import { LegacyConfigDialog } from './components/LegacyConfigDialog'

/**
 * Hide the initial loader spinner
 */
function hideAppLoader() {
  const loader = document.getElementById('app-loader')
  if (loader) {
    loader.classList.add('hidden')
    // Remove from DOM after transition; fire-and-forget
    setTimeout(() => {
      loader.remove()
    }, 300)
  }
}

/**
 * Show main window immediately — called right after initial render
 * so the user sees the UI early while background init continues.
 */
async function showMainWindow() {
  try {
    hideAppLoader()
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

    logger.info('App environment', {
      isDev: import.meta.env.DEV,
      mode: import.meta.env.MODE,
    })

    // Show the main window immediately — user sees the UI right away
    showMainWindow()

    // Background initialization after window is visible:
    // - preload shell env (one-time Tauri command)
    // - load language + build native menu (Tauri commands)
    // - cleanup old recovery files (filesystem I/O)
    // All run in parallel and don't block the UI
    const backgroundInit = async () => {
      await Promise.all([
        // Preload shell environment to avoid delay when first terminal is created
        preloadShellEnv().catch(() => {
          /* preload errors are handled internally */
        }),

        // Initialize language and build native menu
        (async () => {
          try {
            const result = await commands.loadPreferences()
            const savedLanguage =
              result.status === 'ok' ? result.data.language : null

            await initializeLanguage(savedLanguage)
            await buildAppMenu()
            logger.debug('Application menu built')
            setupMenuLanguageListener()
          } catch (error) {
            logger.warn('Failed to initialize language or menu', { error })
          }
        })(),

        // Clean up old recovery files
        cleanupOldFiles().catch(error => {
          logger.warn('Failed to cleanup old recovery files', { error })
        }),
      ])
    }

    backgroundInit()

    // Auto-updater logic - check for updates 5 seconds after app loads
    const checkForUpdates = async () => {
      if (isUpdateCheckDisabled()) {
        logger.debug('Update check disabled via environment variable')
        return
      }

      // Check if auto-update is disabled in preferences
      const prefsResult = await commands.loadPreferences()
      if (prefsResult.status === 'ok' && prefsResult.data.disable_auto_update) {
        logger.debug('Auto-update check disabled in preferences')
        return
      }

      try {
        const update = await checkForUpdate()
        if (update) {
          logger.info(`Update available: ${update.version}`)
          showUpdateNotification(update)
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
