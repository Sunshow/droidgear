import { beforeEach, describe, expect, it, vi } from 'vitest'
import { check } from '@tauri-apps/plugin-updater'
import { fireEvent, render, screen } from '@/test/test-utils'
import { checkForUpdate, clearCachedUpdate } from '@/services/updater'
import { useUIStore } from '@/store/ui-store'
import { GeneralPane } from './GeneralPane'

describe('GeneralPane software update section', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    clearCachedUpdate()
    useUIStore.setState({
      pendingUpdate: null,
      isUpdateInstalling: false,
    })
    vi.mocked(check).mockResolvedValue(null)
  })

  it('keeps the check button available while an older release is displayed', async () => {
    vi.mocked(check).mockResolvedValueOnce({
      version: '0.5.4',
      body: 'Stale release notes',
      downloadAndInstall: vi.fn(),
    } as never)

    const staleUpdate = await checkForUpdate()
    if (!staleUpdate) {
      throw new Error('Expected an update to be available')
    }
    useUIStore.getState().setPendingUpdate(staleUpdate)

    vi.mocked(check).mockResolvedValueOnce({
      version: '0.5.5',
      body: 'Fresh release notes',
      downloadAndInstall: vi.fn(),
    } as never)

    render(<GeneralPane />)

    expect(
      await screen.findByText('Update available: 0.5.4')
    ).toBeInTheDocument()

    // The button used to be hidden once a pending update was shown, which made
    // it impossible to look for a release published after it.
    fireEvent.click(screen.getByRole('button', { name: 'Check for Updates' }))

    expect(
      await screen.findByText('Update available: 0.5.5')
    ).toBeInTheDocument()
    expect(vi.mocked(check)).toHaveBeenCalledTimes(2)
  })
})
