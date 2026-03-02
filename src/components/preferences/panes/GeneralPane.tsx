import { useState, useEffect, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import { useQuery } from '@tanstack/react-query'
import { check } from '@tauri-apps/plugin-updater'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { SettingsField, SettingsSection } from '../shared/SettingsComponents'
import { commands } from '@/lib/tauri-bindings'
import { logger } from '@/lib/logger'
import { useUIStore } from '@/store/ui-store'
import {
  getCachedUpdate,
  setCachedUpdate,
  downloadAndInstallUpdate,
} from '@/services/updater'
import { usePreferences, useSavePreferences } from '@/services/preferences'

export function GeneralPane() {
  const { t } = useTranslation()
  const { data: preferences } = usePreferences()
  const savePreferences = useSavePreferences()

  // Update state
  const [updateStatus, setUpdateStatus] = useState<'idle' | 'checking'>('idle')
  const [hasUpdate, setHasUpdate] = useState(false)
  const [updateError, setUpdateError] = useState<string | null>(null)

  // Read pending update from global store (set by auto-updater in App.tsx)
  const pendingUpdate = useUIStore(state => state.pendingUpdate)

  // Track if we've already triggered auto-check for pending update
  const hasAutoChecked = useRef(false)

  // Get current app version
  const { data: appVersion } = useQuery({
    queryKey: ['app-version'],
    queryFn: async () => {
      return await commands.getAppVersion()
    },
    staleTime: Infinity,
  })

  // Auto-update disabled state
  const disableAutoUpdate = preferences?.disable_auto_update ?? false

  const handleDisableAutoUpdateChange = (checked: boolean) => {
    if (preferences) {
      savePreferences.mutate({
        ...preferences,
        disable_auto_update: checked || null, // Store null instead of false for cleaner serialization
      })
    }
  }

  const handleCheckForUpdates = async () => {
    setUpdateStatus('checking')
    setUpdateError(null)
    setHasUpdate(false)

    try {
      const update = await check()
      if (update) {
        setHasUpdate(true)
        setCachedUpdate(update)
        // Also update global store
        useUIStore.getState().setPendingUpdate({
          version: update.version,
          body: update.body ?? undefined,
        })
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

  // When there's a pending update, check if we have the cached Update object
  useEffect(() => {
    const fetchUpdate = async () => {
      if (pendingUpdate && !hasAutoChecked.current) {
        // Check if we already have the cached update
        const cached = getCachedUpdate()
        if (cached && cached.version === pendingUpdate.version) {
          setHasUpdate(true)
          return
        }

        hasAutoChecked.current = true
        logger.info('Fetching full update object for pending update', {
          version: pendingUpdate.version,
        })

        setUpdateStatus('checking')
        setUpdateError(null)

        try {
          const update = await check()
          if (update) {
            setHasUpdate(true)
            setCachedUpdate(update)
            useUIStore.getState().setPendingUpdate({
              version: update.version,
              body: update.body ?? undefined,
            })
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
    }

    fetchUpdate()
  }, [pendingUpdate])

  const handleDownloadAndInstall = async () => {
    downloadAndInstallUpdate()
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
          {/* Only show check button when no update is available */}
          {!hasUpdate && !pendingUpdate && (
            <div className="flex items-center gap-3">
              <Button
                variant="outline"
                size="sm"
                onClick={handleCheckForUpdates}
                disabled={updateStatus !== 'idle'}
              >
                {updateStatus === 'checking'
                  ? t('preferences.general.checking')
                  : t('preferences.general.checkForUpdates')}
              </Button>
            </div>
          )}
        </SettingsField>

        {/* Update status display - show when update is available */}
        {updateStatus === 'idle' && (hasUpdate || pendingUpdate) && (
          <>
            <SettingsField
              label={t('preferences.general.updateAvailable', {
                version: pendingUpdate?.version,
              })}
              description=""
            >
              {hasUpdate ? (
                <Button
                  variant="default"
                  size="sm"
                  onClick={handleDownloadAndInstall}
                >
                  {t('preferences.general.downloadAndInstall')}
                </Button>
              ) : (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleCheckForUpdates}
                >
                  {t('preferences.general.checkForUpdates')}
                </Button>
              )}
            </SettingsField>
          </>
        )}

        {updateStatus === 'idle' &&
          !hasUpdate &&
          !pendingUpdate &&
          !updateError && (
            <p className="text-sm text-muted-foreground">
              {t('preferences.general.upToDate')}
            </p>
          )}

        {updateError && (
          <p className="text-sm text-destructive">
            {t('preferences.general.updateFailed')}: {updateError}
          </p>
        )}

        {/* Always show release notes if available */}
        {pendingUpdate?.body && (
          <div className="text-sm text-muted-foreground whitespace-pre-wrap max-h-48 overflow-y-auto rounded-md border p-3">
            {pendingUpdate.body}
          </div>
        )}

        {/* Auto-update toggle */}
        <SettingsField
          label={t('preferences.general.autoUpdate')}
          description={t('preferences.general.autoUpdateDescription')}
        >
          <div className="flex items-center space-x-2">
            <Switch
              id="auto-update-toggle"
              checked={!disableAutoUpdate}
              onCheckedChange={checked => handleDisableAutoUpdateChange(!checked)}
              disabled={savePreferences.isPending}
            />
            <Label htmlFor="auto-update-toggle" className="text-sm">
              {!disableAutoUpdate
                ? t('common.enabled')
                : t('common.disabled')}
            </Label>
          </div>
        </SettingsField>
      </SettingsSection>
    </div>
  )
}
