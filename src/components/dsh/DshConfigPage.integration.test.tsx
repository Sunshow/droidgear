import { beforeEach, describe, expect, it, vi } from 'vitest'
import userEvent from '@testing-library/user-event'

const { commandMocks } = vi.hoisted(() => ({
  commandMocks: {
    readDshCurrentConfig: vi.fn(),
    readDshCredentials: vi.fn(),
    getDshConfigStatus: vi.fn(),
    readDshConfigSnapshot: vi.fn(),
    saveDshConfigEntry: vi.fn(),
    setDshConfigEntryDisabled: vi.fn(),
    fetchDshModels: vi.fn(),
    saveDshProvider: vi.fn(),
    deleteDshProvider: vi.fn(),
    saveDshCredentialRef: vi.fn(),
    deleteDshCredentialRef: vi.fn(),
  },
}))

vi.mock('@/lib/bindings', () => ({ commands: commandMocks }))

import { render, screen, waitFor } from '@/test/test-utils'
import { useDshStore } from '@/store/dsh-store'
import { DshConfigPage } from './DshConfigPage'
import { makeEntry, makeField, makeSnapshot } from './dsh-test-fixtures'

const snapshot = makeSnapshot({
  entries: [
    makeEntry({
      id: 'ui-settings',
      config: { enabled: true },
      fields: [makeField({ name: 'enabled', kind: 'boolean', value: true })],
    }),
  ],
})

describe('DshConfigPage integration', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    useDshStore.setState({
      providers: {},
      credentials: {},
      profile: null,
      isLoading: false,
      error: null,
      configStatus: null,
      snapshot: null,
      isLoadingSnapshot: false,
      savingEntryId: null,
    })
    commandMocks.readDshCurrentConfig.mockResolvedValue({
      status: 'ok',
      data: { providers: {} },
    })
    commandMocks.readDshCredentials.mockResolvedValue({
      status: 'ok',
      data: { refs: {} },
    })
    commandMocks.getDshConfigStatus.mockResolvedValue({
      status: 'ok',
      data: {
        profileName: 'desktop',
        configExists: true,
        configPath: '/home/user/.dsh/profiles/desktop/cordis.patch.yml',
        legacyExists: false,
        legacyPath: '/home/user/.dsh/settings.yaml',
        credentialsExists: false,
        credentialsPath: '/home/user/.dsh/.credentials.yaml',
      },
    })
    commandMocks.readDshConfigSnapshot.mockResolvedValue({
      status: 'ok',
      data: snapshot,
    })
  })

  it('mounts the real tabs and loads the composed snapshot', async () => {
    const user = userEvent.setup()
    render(<DshConfigPage profile="desktop" />)

    await screen.findByText('Config status')
    expect(useDshStore.getState().snapshot?.patchHash).toBe('abc123')

    await user.click(screen.getByRole('tab', { name: 'General' }))
    expect(await screen.findByText('Show developer tools')).toBeInTheDocument()

    await user.click(screen.getByRole('tab', { name: 'Diagnostics' }))
    expect(await screen.findByText('ui-settings')).toBeInTheDocument()
    expect(screen.getByText('Plugin entries')).toBeInTheDocument()

    await user.click(screen.getByRole('tab', { name: 'Models' }))
    expect(
      await screen.findByRole('button', { name: 'Add Provider' })
    ).toBeInTheDocument()
  })

  it('keeps the store error banner visible across tab switches', async () => {
    const user = userEvent.setup()
    commandMocks.readDshConfigSnapshot.mockResolvedValue({
      status: 'error',
      error: 'DSH_CONFIG_CONFLICT: stale',
    })
    render(<DshConfigPage profile="desktop" />)

    await waitFor(() => {
      expect(useDshStore.getState().error).toBe(
        'The config was changed by DSH or another tool. Refresh and try again.'
      )
    })
    expect(
      screen.getByText(
        'The config was changed by DSH or another tool. Refresh and try again.'
      )
    ).toBeInTheDocument()

    await user.click(screen.getByRole('tab', { name: 'Diagnostics' }))
    expect(
      screen.getByText(
        'The config was changed by DSH or another tool. Refresh and try again.'
      )
    ).toBeInTheDocument()
  })
})
