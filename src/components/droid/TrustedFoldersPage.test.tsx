import { beforeEach, describe, expect, it, vi } from 'vitest'
import userEvent from '@testing-library/user-event'

const { openMock, toastMock } = vi.hoisted(() => ({
  openMock: vi.fn(),
  toastMock: Object.assign(vi.fn(), {
    success: vi.fn(),
    error: vi.fn(),
  }),
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({ open: openMock }))
vi.mock('sonner', () => ({ toast: toastMock }))

import { render, screen, waitFor } from '@/test/test-utils'
import { commands } from '@/lib/tauri-bindings'
import { TrustedFoldersPage } from './TrustedFoldersPage'

describe('TrustedFoldersPage', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    openMock.mockResolvedValue('/home/user/project')
    vi.mocked(commands.listDroidTrustedFolders).mockResolvedValue({
      status: 'ok',
      data: [
        {
          path: '/home/user/existing',
          trustedAt: '2026-01-01T00:00:00.000Z',
        },
      ],
    })
    vi.mocked(commands.addDroidTrustedFolder).mockResolvedValue({
      status: 'ok',
      data: {
        path: '/home/user/project',
        trustedAt: '2026-01-02T00:00:00.000Z',
      },
    })
    vi.mocked(commands.removeDroidTrustedFolder).mockResolvedValue({
      status: 'ok',
      data: null,
    })
  })

  it('loads folders and adds a directory selected from the native picker', async () => {
    const user = userEvent.setup()
    render(<TrustedFoldersPage />)

    expect(await screen.findByText('/home/user/existing')).toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'Add folder' }))

    await waitFor(() => {
      expect(openMock).toHaveBeenCalledWith({
        directory: true,
        multiple: false,
        title: 'Select a folder to trust',
      })
      expect(commands.addDroidTrustedFolder).toHaveBeenCalledWith(
        '/home/user/project'
      )
    })
    expect(await screen.findByText('/home/user/project')).toBeInTheDocument()
  })

  it('requires confirmation before removing a folder', async () => {
    const user = userEvent.setup()
    render(<TrustedFoldersPage />)

    await screen.findByText('/home/user/existing')
    await user.click(
      screen.getByRole('button', { name: 'Remove trusted folder' })
    )
    expect(
      await screen.findByText(
        'Remove "/home/user/existing" from the trusted-folder list?'
      )
    ).toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: 'Delete' }))
    await waitFor(() => {
      expect(commands.removeDroidTrustedFolder).toHaveBeenCalledWith(
        '/home/user/existing'
      )
    })
    await waitFor(() => {
      expect(screen.queryByText('/home/user/existing')).not.toBeInTheDocument()
    })
  })

  it('selects all folders and removes them with one batch command', async () => {
    const user = userEvent.setup()
    vi.mocked(commands.listDroidTrustedFolders).mockResolvedValue({
      status: 'ok',
      data: [
        {
          path: '/home/user/a',
          trustedAt: '2026-01-01T00:00:00.000Z',
        },
        {
          path: '/home/user/b',
          trustedAt: '2026-01-02T00:00:00.000Z',
        },
      ],
    })

    render(<TrustedFoldersPage />)
    await screen.findByText('/home/user/a')

    await user.click(screen.getByRole('checkbox', { name: 'Select All' }))
    await user.click(
      screen.getByRole('button', { name: /Remove selected \(2\)/ })
    )
    expect(
      await screen.findByText(
        'Remove 2 selected folders from the trusted-folder list?'
      )
    ).toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: 'Delete' }))
    await waitFor(() => {
      expect(commands.removeDroidTrustedFolders).toHaveBeenCalledWith([
        '/home/user/a',
        '/home/user/b',
      ])
    })
    await waitFor(() => {
      expect(screen.queryByText('/home/user/a')).not.toBeInTheDocument()
      expect(screen.queryByText('/home/user/b')).not.toBeInTheDocument()
    })
  })
})
