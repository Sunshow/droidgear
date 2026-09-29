import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  commands,
  type CopilotProfile,
  type CopilotChannelSelection,
} from '@/lib/bindings'
import { useCopilotStore } from './copilot-store'

vi.mock('@/lib/bindings', () => ({
  commands: {
    listCopilotProfiles: vi.fn(),
    createDefaultCopilotProfile: vi.fn(),
    getActiveCopilotProfileId: vi.fn(),
    getCopilotConfigStatus: vi.fn(),
    saveCopilotProfile: vi.fn(),
    applyCopilotProfile: vi.fn(),
    readCopilotCurrentConfig: vi.fn(),
    deleteCopilotProfile: vi.fn(),
    prepareCopilotChannelImport: vi.fn(),
  },
}))

const profile: CopilotProfile = {
  id: 'existing',
  name: 'Provider',
  description: null,
  createdAt: '',
  updatedAt: '',
  useOfficialAuth: false,
  baseUrl: 'https://example.test/v1',
  providerType: 'openai',
  apiKey: 'test-key',
  model: 'test-model',
  maxPromptTokens: 1000,
  maxOutputTokens: 100,
}

describe('Copilot profile editor', () => {
  beforeEach(() => {
    vi.resetAllMocks()
    useCopilotStore.setState({
      profiles: [profile],
      currentProfile: { ...profile },
      activeProfileId: profile.id,
      error: null,
      isLoading: false,
      configStatus: null,
    })
    vi.mocked(commands.listCopilotProfiles).mockResolvedValue({
      status: 'ok',
      data: [profile],
    })
    vi.mocked(commands.saveCopilotProfile).mockResolvedValue({
      status: 'ok',
      data: null,
    })
    vi.mocked(commands.applyCopilotProfile).mockResolvedValue({
      status: 'ok',
      data: null,
    })
    vi.mocked(commands.getActiveCopilotProfileId).mockResolvedValue({
      status: 'ok',
      data: null,
    })
    vi.mocked(commands.getCopilotConfigStatus).mockResolvedValue({
      status: 'ok',
      data: { configExists: true, configPath: '/test/config.env' },
    })
  })

  it('selects a newly created profile by id even when its name is duplicated', async () => {
    vi.mocked(commands.saveCopilotProfile).mockImplementation(async value => {
      vi.mocked(commands.listCopilotProfiles).mockResolvedValue({
        status: 'ok',
        data: [profile, value],
      })
      return { status: 'ok', data: null }
    })
    await useCopilotStore.getState().createProfile(profile.name)
    const current = useCopilotStore.getState().currentProfile
    expect(current?.id).not.toBe(profile.id)
    expect(current?.name).toBe(profile.name)
    expect(current?.model).toBeNull()
  })

  it('saves an active profile without applying it', async () => {
    await useCopilotStore.getState().saveProfile()
    expect(commands.saveCopilotProfile).toHaveBeenCalledWith(profile)
    expect(commands.applyCopilotProfile).not.toHaveBeenCalled()
  })

  const selection: CopilotChannelSelection = {
    channelType: 'sub-2-api',
    baseUrl: 'https://channel.test',
    apiKey: 'channel-key',
    platform: 'openai',
    provider: 'openai',
    model: 'channel-model',
    maxOutputTokens: null,
  }

  it('imports the channel selection into the draft without saving or applying', async () => {
    const imported = {
      ...profile,
      baseUrl: 'https://channel.test/v1',
      model: selection.model,
      apiKey: selection.apiKey,
    }
    vi.mocked(commands.prepareCopilotChannelImport).mockResolvedValue({
      status: 'ok',
      data: imported,
    })
    await useCopilotStore.getState().importFromChannel(selection)
    expect(commands.prepareCopilotChannelImport).toHaveBeenCalledWith(
      profile,
      selection
    )
    expect(useCopilotStore.getState().currentProfile).toEqual(imported)
    expect(useCopilotStore.getState().profiles).toEqual([profile])
    expect(useCopilotStore.getState().activeProfileId).toBe(profile.id)
    expect(commands.saveCopilotProfile).not.toHaveBeenCalled()
    expect(commands.applyCopilotProfile).not.toHaveBeenCalled()
  })

  it('leaves edits intact on import failure and does not overwrite another selected profile', async () => {
    vi.mocked(commands.prepareCopilotChannelImport).mockResolvedValueOnce({
      status: 'error',
      error: 'Unsupported protocol',
    })
    await useCopilotStore.getState().importFromChannel(selection)
    expect(useCopilotStore.getState().currentProfile).toEqual(profile)
    expect(useCopilotStore.getState().error).toContain('Unsupported protocol')
    const other = { ...profile, id: 'other' }
    vi.mocked(commands.prepareCopilotChannelImport).mockImplementationOnce(
      async () => {
        useCopilotStore.setState({ currentProfile: other })
        return { status: 'ok', data: { ...profile, model: 'imported-model' } }
      }
    )
    await useCopilotStore.getState().importFromChannel(selection)
    expect(useCopilotStore.getState().currentProfile).toEqual(other)
    expect(useCopilotStore.getState().isLoading).toBe(false)
  })

  it('stops applying when saving fails and exposes rejected IPC errors', async () => {
    vi.mocked(commands.saveCopilotProfile).mockResolvedValueOnce({
      status: 'error',
      error: 'disk full',
    })
    await useCopilotStore.getState().applyProfile(profile.id)
    expect(commands.applyCopilotProfile).not.toHaveBeenCalled()
    expect(useCopilotStore.getState().error).toBe('disk full')
    vi.mocked(commands.saveCopilotProfile).mockRejectedValueOnce(
      new Error('IPC disconnected')
    )
    await useCopilotStore.getState().saveProfile()
    expect(useCopilotStore.getState().error).toContain('IPC disconnected')
    expect(useCopilotStore.getState().isLoading).toBe(false)
  })

  it('clears stale credentials and token limits when importing official configuration', async () => {
    vi.mocked(commands.readCopilotCurrentConfig).mockResolvedValue({
      status: 'ok',
      data: {
        isByok: false,
        baseUrl: null,
        providerType: null,
        apiKey: null,
        model: null,
        maxPromptTokens: null,
        maxOutputTokens: null,
      },
    })
    await useCopilotStore.getState().loadFromLiveConfig()
    expect(commands.saveCopilotProfile).toHaveBeenCalledWith(
      expect.objectContaining({
        useOfficialAuth: true,
        baseUrl: null,
        apiKey: null,
        model: null,
        maxPromptTokens: null,
        maxOutputTokens: null,
      })
    )
    expect(commands.applyCopilotProfile).not.toHaveBeenCalled()
  })

  it('clears the active marker and selects a remaining profile after deleting', async () => {
    const remaining = { ...profile, id: 'remaining', name: 'Other' }
    vi.mocked(commands.deleteCopilotProfile).mockResolvedValue({
      status: 'ok',
      data: null,
    })
    vi.mocked(commands.listCopilotProfiles).mockResolvedValue({
      status: 'ok',
      data: [remaining],
    })
    await useCopilotStore.getState().deleteProfile(profile.id)
    expect(useCopilotStore.getState().activeProfileId).toBeNull()
    expect(useCopilotStore.getState().currentProfile?.id).toBe(remaining.id)
  })
})
