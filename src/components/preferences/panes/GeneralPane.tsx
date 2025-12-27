import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useQuery } from '@tanstack/react-query'
import { toast } from 'sonner'
import { check, type Update } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { Switch } from '@/components/ui/switch'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Button } from '@/components/ui/button'
import { ShortcutPicker } from '../ShortcutPicker'
import { SettingsField, SettingsSection } from '../shared/SettingsComponents'
import { usePreferences, useSavePreferences } from '@/services/preferences'
import { commands } from '@/lib/tauri-bindings'
import { logger } from '@/lib/logger'

export function GeneralPane() {
  const { t } = useTranslation()
  // Example local state - these are NOT persisted to disk
  // To add persistent preferences:
  // 1. Add the field to AppPreferences in both Rust and TypeScript
  // 2. Use usePreferencesManager() and updatePreferences()
  const [exampleText, setExampleText] = useState('Example value')
  const [exampleToggle, setExampleToggle] = useState(true)

  // Update state
  const [updateStatus, setUpdateStatus] = useState<
    'idle' | 'checking' | 'downloading' | 'installing'
  >('idle')
  const [availableUpdate, setAvailableUpdate] = useState<Update | null>(null)
  const [updateError, setUpdateError] = useState<string | null>(null)

  // Load preferences for keyboard shortcuts
  const { data: preferences } = usePreferences()
  const savePreferences = useSavePreferences()

  // Get the default shortcut from the backend
  const { data: defaultShortcut } = useQuery({
    queryKey: ['default-quick-pane-shortcut'],
    queryFn: async () => {
      return await commands.getDefaultQuickPaneShortcut()
    },
    staleTime: Infinity, // Never refetch - this is a constant
  })

  // Get current app version
  const { data: appVersion } = useQuery({
    queryKey: ['app-version'],
    queryFn: async () => {
      return await commands.getAppVersion()
    },
    staleTime: Infinity,
  })

  const handleCheckForUpdates = async () => {
    setUpdateStatus('checking')
    setUpdateError(null)
    setAvailableUpdate(null)

    try {
      const update = await check()
      if (update) {
        setAvailableUpdate(update)
        logger.info('Update available', { version: update.version })
      } else {
        logger.info('No updates available')
      }
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : String(error)
      setUpdateError(errorMessage)
      logger.error('Failed to check for updates', { error: errorMessage })
    } finally {
      setUpdateStatus('idle')
    }
  }

  const handleDownloadAndInstall = async () => {
    if (!availableUpdate) return

    setUpdateStatus('downloading')
    try {
      await availableUpdate.downloadAndInstall(event => {
        if (event.event === 'Started') {
          logger.info('Download started', {
            contentLength: event.data.contentLength,
          })
        } else if (event.event === 'Progress') {
          logger.debug('Download progress', {
            chunkLength: event.data.chunkLength,
          })
        } else if (event.event === 'Finished') {
          logger.info('Download finished')
          setUpdateStatus('installing')
        }
      })
      toast.success(t('update.completed'))
      await relaunch()
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : String(error)
      setUpdateError(errorMessage)
      setUpdateStatus('idle')
      logger.error('Failed to download/install update', { error: errorMessage })
    }
  }

  const handleShortcutChange = async (newShortcut: string | null) => {
    if (!preferences) return

    // Capture old shortcut for rollback if save fails
    const oldShortcut = preferences.quick_pane_shortcut

    logger.info('Updating quick pane shortcut', { oldShortcut, newShortcut })

    // First, try to register the new shortcut
    const result = await commands.updateQuickPaneShortcut(newShortcut)

    if (result.status === 'error') {
      logger.error('Failed to register shortcut', { error: result.error })
      toast.error(t('toast.error.shortcutFailed'), {
        description: result.error,
      })
      return
    }

    // If registration succeeded, try to save the preference
    try {
      await savePreferences.mutateAsync({
        ...preferences,
        quick_pane_shortcut: newShortcut,
      })
    } catch {
      // Save failed - roll back the backend registration
      logger.warn('Save failed, rolling back shortcut registration', {
        oldShortcut,
        newShortcut,
      })

      const rollbackResult = await commands.updateQuickPaneShortcut(oldShortcut)

      if (rollbackResult.status === 'error') {
        logger.error(
          'Rollback failed - backend and preferences are out of sync',
          {
            error: rollbackResult.error,
            attemptedShortcut: newShortcut,
            originalShortcut: oldShortcut,
          }
        )
        toast.error(t('toast.error.shortcutRestoreFailed'), {
          description: t('toast.error.shortcutRestoreDescription'),
        })
      } else {
        logger.info('Successfully rolled back shortcut registration')
      }
    }
  }

  return (
    <div className="space-y-6">
      <SettingsSection title={t('preferences.general.softwareUpdate')}>
        <SettingsField
          label={t('preferences.general.currentVersion', {
            version: appVersion ?? '...',
          })}
          description=""
        >
          <div className="flex items-center gap-3">
            <Button
              variant="outline"
              size="sm"
              onClick={handleCheckForUpdates}
              disabled={updateStatus !== 'idle'}
            >
              {updateStatus === 'checking'
                ? t('preferences.general.checking')
                : updateStatus === 'downloading'
                  ? t('preferences.general.downloading')
                  : updateStatus === 'installing'
                    ? t('preferences.general.installing')
                    : t('preferences.general.checkForUpdates')}
            </Button>
          </div>
        </SettingsField>

        {/* Update status display */}
        {updateStatus === 'idle' && availableUpdate && (
          <SettingsField
            label={t('preferences.general.updateAvailable', {
              version: availableUpdate.version,
            })}
            description=""
          >
            <Button
              variant="default"
              size="sm"
              onClick={handleDownloadAndInstall}
            >
              {t('preferences.general.downloadAndInstall')}
            </Button>
          </SettingsField>
        )}

        {updateStatus === 'idle' && !availableUpdate && !updateError && (
          <p className="text-sm text-muted-foreground">
            {t('preferences.general.upToDate')}
          </p>
        )}

        {updateError && (
          <p className="text-sm text-destructive">
            {t('preferences.general.updateFailed')}: {updateError}
          </p>
        )}
      </SettingsSection>

      <SettingsSection title={t('preferences.general.keyboardShortcuts')}>
        <SettingsField
          label={t('preferences.general.quickPaneShortcut')}
          description={t('preferences.general.quickPaneShortcutDescription')}
        >
          <ShortcutPicker
            value={preferences?.quick_pane_shortcut ?? null}
            // Fallback matches DEFAULT_QUICK_PANE_SHORTCUT in src-tauri/src/lib.rs
            defaultValue={defaultShortcut ?? 'CommandOrControl+Shift+.'}
            onChange={handleShortcutChange}
            disabled={!preferences || savePreferences.isPending}
          />
        </SettingsField>
      </SettingsSection>

      <SettingsSection title={t('preferences.general.exampleSettings')}>
        <SettingsField
          label={t('preferences.general.exampleText')}
          description={t('preferences.general.exampleTextDescription')}
        >
          <Input
            value={exampleText}
            onChange={e => setExampleText(e.target.value)}
            placeholder={t('preferences.general.exampleTextPlaceholder')}
          />
        </SettingsField>

        <SettingsField
          label={t('preferences.general.exampleToggle')}
          description={t('preferences.general.exampleToggleDescription')}
        >
          <div className="flex items-center space-x-2">
            <Switch
              id="example-toggle"
              checked={exampleToggle}
              onCheckedChange={setExampleToggle}
            />
            <Label htmlFor="example-toggle" className="text-sm">
              {exampleToggle ? t('common.enabled') : t('common.disabled')}
            </Label>
          </div>
        </SettingsField>
      </SettingsSection>
    </div>
  )
}
