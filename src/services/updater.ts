/**
 * Update service module - handles managed and portable app updates
 */

import { createElement } from 'react'
import { openUrl } from '@tauri-apps/plugin-opener'
import { relaunch } from '@tauri-apps/plugin-process'
import { type Update, check } from '@tauri-apps/plugin-updater'
import { toast } from 'sonner'
import { UpdateNotificationContent } from '@/components/updates/UpdateNotificationContent'
import i18n from '@/i18n/config'
import {
  commands,
  type PortableUpdateInfo,
  type UpdateChannel,
  unwrapResult,
} from '@/lib/tauri-bindings'
import { logger } from '@/lib/logger'
import { type PendingUpdate, useUIStore } from '@/store/ui-store'

const RELEASES_BASE_URL = 'https://github.com/Sunshow/droidgear/releases/tag/'

let cachedManagedUpdate: Update | null = null
let cachedPortableUpdate: PortableUpdateInfo | null = null
let cachedUpdateChannel: UpdateChannel | null = null

export function normalizeVersion(version: string): string {
  return version.trim().replace(/^v/i, '')
}

export function buildReleaseUrl(version: string): string {
  return `${RELEASES_BASE_URL}v${normalizeVersion(version)}`
}

export function clearCachedUpdate(): void {
  cachedManagedUpdate = null
  cachedPortableUpdate = null
  cachedUpdateChannel = null
}

export function isUpdateCheckDisabled(): boolean {
  const disableUpdate = import.meta.env.DROIDGEAR_DISABLE_UPDATE_CHECK
  return disableUpdate === 'true' || disableUpdate === '1'
}

export function hasCachedUpdate(pendingUpdate: PendingUpdate | null): boolean {
  if (!pendingUpdate) {
    return false
  }

  if (pendingUpdate.channel === 'portable') {
    return (
      cachedPortableUpdate !== null &&
      normalizeVersion(cachedPortableUpdate.version) === pendingUpdate.version
    )
  }

  return (
    cachedManagedUpdate !== null &&
    normalizeVersion(cachedManagedUpdate.version) === pendingUpdate.version
  )
}

async function resolveUpdateChannel(): Promise<UpdateChannel> {
  if (cachedUpdateChannel) {
    return cachedUpdateChannel
  }

  cachedUpdateChannel = unwrapResult(await commands.getUpdateChannel())
  return cachedUpdateChannel
}

function createPendingManagedUpdate(update: Update): PendingUpdate {
  cachedManagedUpdate = update

  return {
    version: normalizeVersion(update.version),
    body: update.body ?? undefined,
    channel: 'managed',
    releaseUrl: buildReleaseUrl(update.version),
  }
}

function createPendingPortableUpdate(
  portableUpdate: PortableUpdateInfo
): PendingUpdate {
  cachedPortableUpdate = portableUpdate

  return {
    version: normalizeVersion(portableUpdate.version),
    body: portableUpdate.body ?? undefined,
    channel: 'portable',
    releaseUrl: portableUpdate.releaseUrl,
  }
}

async function checkManagedUpdate(): Promise<PendingUpdate | null> {
  const update = await check()
  return update ? createPendingManagedUpdate(update) : null
}

async function checkPortableUpdate(): Promise<PendingUpdate | null> {
  const portableUpdate = unwrapResult(await commands.checkPortableUpdate())
  return portableUpdate ? createPendingPortableUpdate(portableUpdate) : null
}

export async function checkForUpdate(): Promise<PendingUpdate | null> {
  if (isUpdateCheckDisabled()) {
    logger.debug('Update check disabled via environment variable')
    return null
  }

  try {
    const channel = await resolveUpdateChannel()
    const update =
      channel === 'portable'
        ? await checkPortableUpdate()
        : await checkManagedUpdate()
    if (!update) {
      clearCachedUpdate()
    }
    return update
  } catch (error) {
    logger.error('Update check failed', { error })
    throw error
  }
}

export async function hydratePendingUpdate(
  pendingUpdate: PendingUpdate
): Promise<PendingUpdate | null> {
  if (hasCachedUpdate(pendingUpdate)) {
    return pendingUpdate
  }

  const refreshedUpdate = await checkForUpdate()
  if (!refreshedUpdate) {
    clearCachedUpdate()
  }

  return refreshedUpdate
}

async function openReleasePage(releaseUrl: string): Promise<void> {
  try {
    await openUrl(releaseUrl)
  } catch (error) {
    logger.error('Failed to open release page', { releaseUrl, error })
  }
}

/**
 * Download and install the update with progress tracking
 */
export async function downloadAndInstallUpdate(): Promise<void> {
  const pendingUpdate = useUIStore.getState().pendingUpdate
  if (!pendingUpdate) {
    logger.error('No update available to install')
    return
  }

  const t = i18n.t.bind(i18n)

  if (pendingUpdate.channel === 'portable') {
    try {
      if (!cachedPortableUpdate) {
        const refreshedUpdate = await hydratePendingUpdate(pendingUpdate)
        if (!refreshedUpdate || refreshedUpdate.channel !== 'portable') {
          throw new Error('Portable update metadata is no longer available')
        }
        useUIStore.getState().setPendingUpdate(refreshedUpdate)
      }

      if (!cachedPortableUpdate) {
        throw new Error('Portable update metadata is missing')
      }

      toast.loading(t('update.installing'), {
        id: 'update-progress',
        duration: Infinity,
      })

      unwrapResult(await commands.installPortableUpdate(cachedPortableUpdate))
      clearCachedUpdate()
      useUIStore.getState().clearPendingUpdate()
      toast.success(t('update.completed'), { id: 'update-progress' })
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : String(error)
      logger.error('Failed to install portable update', {
        error: errorMessage,
      })
      toast.error(t('update.failed', { error: errorMessage }), {
        id: 'update-progress',
      })
    }

    return
  }

  if (!cachedManagedUpdate) {
    try {
      const refreshedUpdate = await hydratePendingUpdate(pendingUpdate)
      if (!refreshedUpdate || refreshedUpdate.channel !== 'managed') {
        throw new Error('Managed update metadata is no longer available')
      }
      useUIStore.getState().setPendingUpdate(refreshedUpdate)
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : String(error)
      logger.error('Failed to refresh managed update metadata', {
        error: errorMessage,
      })
      toast.error(t('update.failed', { error: errorMessage }), {
        id: 'update-progress',
      })
      return
    }
  }

  if (!cachedManagedUpdate) {
    logger.error('Managed update metadata is missing')
    return
  }

  let totalSize = 0
  let downloadedSize = 0

  try {
    toast.loading(t('update.downloadProgress', { progress: 0 }), {
      id: 'update-progress',
      duration: Infinity,
    })

    await cachedManagedUpdate.downloadAndInstall(event => {
      if (event.event === 'Started') {
        totalSize = event.data.contentLength ?? 0
        logger.info('Managed update download started', {
          contentLength: totalSize,
        })
      } else if (event.event === 'Progress') {
        downloadedSize += event.data.chunkLength
        const progress =
          totalSize > 0 ? Math.round((downloadedSize / totalSize) * 100) : 0

        toast.loading(t('update.downloadProgress', { progress }), {
          id: 'update-progress',
          duration: Infinity,
        })
      } else if (event.event === 'Finished') {
        logger.info('Managed update download finished')
        toast.loading(t('update.installing'), {
          id: 'update-progress',
          duration: Infinity,
        })
      }
    })

    clearCachedUpdate()
    useUIStore.getState().clearPendingUpdate()
    toast.success(t('update.completed'), { id: 'update-progress' })
    await relaunch()
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    logger.error('Failed to download/install managed update', {
      error: errorMessage,
    })
    toast.error(t('update.failed', { error: errorMessage }), {
      id: 'update-progress',
    })
  }
}

/**
 * Show update notification toast with Install Now and Later buttons
 */
export function showUpdateNotification(
  pendingUpdate: PendingUpdate,
  options: { force?: boolean } = {}
): void {
  const t = i18n.t.bind(i18n)
  const currentPending = useUIStore.getState().pendingUpdate

  if (
    !options.force &&
    currentPending?.channel === pendingUpdate.channel &&
    currentPending.version === pendingUpdate.version
  ) {
    return
  }

  useUIStore.getState().setPendingUpdate(pendingUpdate)

  toast(
    createElement(UpdateNotificationContent, {
      message: t('update.availableNotification', {
        version: pendingUpdate.version,
      }),
      releaseUrl: pendingUpdate.releaseUrl,
      releaseLabel: t('update.viewDetails'),
      onOpenRelease: () => {
        void openReleasePage(pendingUpdate.releaseUrl)
      },
    }),
    {
      id: 'update-available',
      duration: Infinity,
      action: {
        label: t('update.installNow'),
        onClick: () => {
          void downloadAndInstallUpdate()
        },
      },
      cancel: {
        label: t('update.later'),
        onClick: () => {
          toast.dismiss('update-available')
        },
      },
    }
  )
}
