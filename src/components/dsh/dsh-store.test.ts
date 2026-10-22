import { beforeEach, describe, expect, it, vi } from 'vitest'

const { commandMocks } = vi.hoisted(() => ({
  commandMocks: {
    readDshConfigSnapshot: vi.fn(),
    saveDshConfigEntry: vi.fn(),
    setDshConfigEntryDisabled: vi.fn(),
    saveDshProvider: vi.fn(),
    deleteDshProvider: vi.fn(),
    readDshCurrentConfig: vi.fn(),
    getDshConfigStatus: vi.fn(),
  },
}))

vi.mock('@/lib/bindings', () => ({ commands: commandMocks }))

import { useDshStore } from '@/store/dsh-store'
import { makeSnapshot } from './dsh-test-fixtures'

const snapshot = makeSnapshot()

function resetStore() {
  useDshStore.setState({
    providers: {},
    credentials: {},
    profile: 'desktop',
    isLoading: false,
    error: null,
    configStatus: null,
    snapshot,
    isLoadingSnapshot: false,
    savingEntryId: null,
  })
}

describe('dsh store config writes', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    resetStore()
  })

  it('turns a DSH_CONFIG_CONFLICT into a friendly error and rejects', async () => {
    commandMocks.saveDshConfigEntry.mockResolvedValue({
      status: 'error',
      error: 'DSH_CONFIG_CONFLICT: patch changed',
    })

    await expect(
      useDshStore.getState().saveEntry('ui-theme', { preference: 'dark' })
    ).rejects.toThrow(
      'The config was changed by DSH or another tool. Refresh and try again.'
    )

    expect(commandMocks.saveDshConfigEntry).toHaveBeenCalledWith(
      'desktop',
      'ui-theme',
      { preference: 'dark' },
      'abc123'
    )
    expect(useDshStore.getState().error).toBe(
      'The config was changed by DSH or another tool. Refresh and try again.'
    )
    expect(useDshStore.getState().savingEntryId).toBeNull()
  })

  it('turns a DSH_CONFIG_SHADOWED into the home patch message', async () => {
    commandMocks.saveDshConfigEntry.mockResolvedValue({
      status: 'error',
      error: 'DSH_CONFIG_SHADOWED: home overlay wins',
    })

    await expect(
      useDshStore.getState().saveEntry('ui-theme', { preference: 'dark' })
    ).rejects.toThrow(
      'This entry is shadowed by the home patch; writes to the profile have no effect.'
    )
    expect(useDshStore.getState().error).toBe(
      'This entry is shadowed by the home patch; writes to the profile have no effect.'
    )
  })

  it('refreshes the snapshot after a successful save', async () => {
    commandMocks.saveDshConfigEntry.mockResolvedValue({
      status: 'ok',
      data: { patchHash: 'def456', changed: true },
    })
    commandMocks.readDshConfigSnapshot.mockResolvedValue({
      status: 'ok',
      data: { ...snapshot, patchHash: 'def456' },
    })

    await useDshStore.getState().saveEntry('ui-settings', { enabled: false })

    expect(commandMocks.readDshConfigSnapshot).toHaveBeenCalledWith('desktop')
    expect(useDshStore.getState().snapshot?.patchHash).toBe('def456')
    expect(useDshStore.getState().error).toBeNull()
    expect(useDshStore.getState().savingEntryId).toBeNull()
  })

  it('forwards the disabled flag with the expected hash', async () => {
    commandMocks.setDshConfigEntryDisabled.mockResolvedValue({
      status: 'ok',
      data: { patchHash: 'def456', changed: true },
    })
    commandMocks.readDshConfigSnapshot.mockResolvedValue({
      status: 'ok',
      data: snapshot,
    })

    await useDshStore.getState().toggleEntryDisabled('bash-sandbox', true)

    expect(commandMocks.setDshConfigEntryDisabled).toHaveBeenCalledWith(
      'desktop',
      'bash-sandbox',
      true,
      'abc123'
    )
  })

  it('refreshes the snapshot after saving a provider', async () => {
    commandMocks.saveDshProvider.mockResolvedValue({ status: 'ok', data: null })
    commandMocks.readDshCurrentConfig.mockResolvedValue({
      status: 'ok',
      data: { providers: {} },
    })
    commandMocks.getDshConfigStatus.mockResolvedValue({
      status: 'ok',
      data: null,
    })
    commandMocks.readDshConfigSnapshot.mockResolvedValue({
      status: 'ok',
      data: { ...snapshot, patchHash: 'after-provider' },
    })

    await useDshStore.getState().saveProvider('byok', {
      models: [{ id: 'gpt-5' }],
      reasoning: 'high',
    })

    expect(commandMocks.saveDshProvider).toHaveBeenCalledWith(
      'desktop',
      'byok',
      expect.objectContaining({ reasoning: 'high' })
    )
    expect(commandMocks.readDshConfigSnapshot).toHaveBeenCalledWith('desktop')
    expect(useDshStore.getState().snapshot?.patchHash).toBe('after-provider')
  })

  it('loads a snapshot for the active profile', async () => {
    commandMocks.readDshConfigSnapshot.mockResolvedValue({
      status: 'ok',
      data: { ...snapshot, patchHash: 'fresh' },
    })

    await useDshStore.getState().loadSnapshot()

    expect(useDshStore.getState().snapshot?.patchHash).toBe('fresh')
    expect(useDshStore.getState().isLoadingSnapshot).toBe(false)
  })
})
