import { beforeEach, describe, expect, it, vi } from 'vitest'

const { toastMock } = vi.hoisted(() => {
  const mock = Object.assign(vi.fn(), {
    loading: vi.fn(),
    success: vi.fn(),
    error: vi.fn(),
    dismiss: vi.fn(),
  })
  return { toastMock: mock }
})

vi.mock('sonner', () => ({
  toast: toastMock,
}))

import { check } from '@tauri-apps/plugin-updater'
import { toast } from 'sonner'
import { commands } from '@/lib/tauri-bindings'
import { useUIStore } from '@/store/ui-store'
import {
  buildReleaseUrl,
  checkForUpdate,
  clearCachedUpdate,
  showUpdateNotification,
} from './updater'

describe('updater service', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    clearCachedUpdate()
    useUIStore.setState({ pendingUpdate: null })

    vi.mocked(commands.getUpdateChannel).mockResolvedValue({
      status: 'ok',
      data: 'managed',
    })
    vi.mocked(commands.checkPortableUpdate).mockResolvedValue({
      status: 'ok',
      data: null,
    })
    vi.mocked(check).mockResolvedValue(null)
  })

  it('builds release links with a v-prefixed tag', () => {
    expect(buildReleaseUrl('0.5.4')).toBe(
      'https://github.com/Sunshow/droidgear/releases/tag/v0.5.4'
    )
    expect(buildReleaseUrl('v0.5.4')).toBe(
      'https://github.com/Sunshow/droidgear/releases/tag/v0.5.4'
    )
  })

  it('routes managed installs through the built-in updater', async () => {
    vi.mocked(check).mockResolvedValue({
      version: '0.5.4',
      body: 'Managed release notes',
      downloadAndInstall: vi.fn(),
    } as never)

    const update = await checkForUpdate()

    expect(update).toEqual({
      version: '0.5.4',
      body: 'Managed release notes',
      channel: 'managed',
      releaseUrl: buildReleaseUrl('0.5.4'),
    })
  })

  it('routes portable installs through the portable manifest command', async () => {
    vi.mocked(commands.getUpdateChannel).mockResolvedValue({
      status: 'ok',
      data: 'portable',
    })
    vi.mocked(commands.checkPortableUpdate).mockResolvedValue({
      status: 'ok',
      data: {
        version: '0.5.4',
        body: 'Portable release notes',
        pubDate: '2026-03-27T00:00:00Z',
        url: 'https://example.com/droidgear_windows_x64.exe',
        signature: 'signature',
        sha256: 'sha256',
        releaseUrl: buildReleaseUrl('0.5.4'),
      },
    })

    const update = await checkForUpdate()

    expect(update).toEqual({
      version: '0.5.4',
      body: 'Portable release notes',
      channel: 'portable',
      releaseUrl: buildReleaseUrl('0.5.4'),
    })
  })

  it('deduplicates identical update notifications unless forced', () => {
    const update = {
      version: '0.5.4',
      body: 'Release notes',
      channel: 'managed' as const,
      releaseUrl: buildReleaseUrl('0.5.4'),
    }

    showUpdateNotification(update)
    showUpdateNotification(update)
    showUpdateNotification(update, { force: true })

    expect(useUIStore.getState().pendingUpdate).toEqual(update)
    expect(toast).toHaveBeenCalledTimes(2)
  })
})
