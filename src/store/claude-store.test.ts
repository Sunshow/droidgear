import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { ClaudeCodeProfile } from '@/lib/bindings'

const { commandMocks } = vi.hoisted(() => ({
  commandMocks: {
    listClaudeProfiles: vi.fn(),
    createDefaultClaudeProfile: vi.fn(),
    getActiveClaudeProfileId: vi.fn(),
    getClaudeConfigStatus: vi.fn(),
    readClaudeCurrentConfig: vi.fn(),
    saveClaudeProfile: vi.fn(),
    deleteClaudeProfile: vi.fn(),
    duplicateClaudeProfile: vi.fn(),
    applyClaudeProfile: vi.fn(),
  },
}))

vi.mock('@/lib/bindings', () => ({
  commands: commandMocks,
}))

import { useClaudeStore } from './claude-store'

const profileA: ClaudeCodeProfile = {
  id: 'profile-a',
  name: 'Claude Alpha',
  description: 'Primary profile',
  baseUrl: 'https://proxy.example.com',
  bearerToken: 'token-a',
  model: 'claude-sonnet-4-5',
  smallModelUsesMainModel: true,
  smallModel: null,
  reasoningEffort: 'medium',
  thinkingMode: 'inherit',
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: '2026-01-01T00:00:00Z',
}

const profileB: ClaudeCodeProfile = {
  ...profileA,
  id: 'profile-b',
  name: 'Claude Beta',
  bearerToken: 'token-b',
  model: 'claude-opus-4-1',
}

function deferred<T>() {
  let resolve!: (value: T) => void
  const promise = new Promise<T>(res => {
    resolve = res
  })

  return { promise, resolve }
}

describe('useClaudeStore', () => {
  beforeEach(() => {
    vi.clearAllMocks()

    useClaudeStore.setState({
      profiles: [profileA, profileB],
      activeProfileId: null,
      currentProfile: JSON.parse(JSON.stringify(profileA)) as ClaudeCodeProfile,
      isLoading: false,
      error: null,
      configStatus: null,
    })

    commandMocks.listClaudeProfiles.mockResolvedValue({
      status: 'ok',
      data: [profileA, profileB],
    })
    commandMocks.createDefaultClaudeProfile.mockResolvedValue({
      status: 'ok',
      data: profileA,
    })
    commandMocks.getActiveClaudeProfileId.mockResolvedValue({
      status: 'ok',
      data: null,
    })
    commandMocks.getClaudeConfigStatus.mockResolvedValue({
      status: 'ok',
      data: {
        settingsExists: true,
        settingsPath: '/home/user/.claude/settings.json',
        configDir: '/home/user/.claude',
        parseError: null,
      },
    })
    commandMocks.readClaudeCurrentConfig.mockResolvedValue({
      status: 'ok',
      data: {
        baseUrl: profileA.baseUrl,
        bearerToken: profileA.bearerToken,
        model: profileA.model,
        smallModelUsesMainModel: profileA.smallModelUsesMainModel,
        smallModel: profileA.smallModel,
        reasoningEffort: profileA.reasoningEffort,
        thinkingMode: profileA.thinkingMode,
      },
    })
    commandMocks.saveClaudeProfile.mockResolvedValue({
      status: 'ok',
      data: null,
    })
    commandMocks.deleteClaudeProfile.mockResolvedValue({
      status: 'ok',
      data: null,
    })
    commandMocks.duplicateClaudeProfile.mockResolvedValue({
      status: 'ok',
      data: profileB,
    })
    commandMocks.applyClaudeProfile.mockResolvedValue({
      status: 'ok',
      data: null,
    })
  })

  it('does not overwrite newer in-memory edits after a blur save refreshes profiles', async () => {
    const saveResult = deferred<{ status: 'ok'; data: null }>()
    commandMocks.saveClaudeProfile.mockReturnValue(saveResult.promise)
    commandMocks.listClaudeProfiles.mockResolvedValue({
      status: 'ok',
      data: [profileA, profileB],
    })

    const savePromise = useClaudeStore.getState().saveProfile()

    useClaudeStore.setState(state => ({
      currentProfile: state.currentProfile
        ? {
            ...state.currentProfile,
            model: 'claude-opus-4-1',
            updatedAt: '2026-01-02T00:00:00Z',
          }
        : null,
    }))

    saveResult.resolve({ status: 'ok', data: null })
    await savePromise

    expect(useClaudeStore.getState().currentProfile).toMatchObject({
      id: 'profile-a',
      model: 'claude-opus-4-1',
      updatedAt: '2026-01-02T00:00:00Z',
    })
  })

  it('does not restore the previously saved profile after the user switches selection', async () => {
    const saveResult = deferred<{ status: 'ok'; data: null }>()
    commandMocks.saveClaudeProfile.mockReturnValue(saveResult.promise)

    const savePromise = useClaudeStore.getState().saveProfile()

    useClaudeStore.getState().selectProfile('profile-b')

    saveResult.resolve({ status: 'ok', data: null })
    await savePromise

    expect(useClaudeStore.getState().currentProfile).toMatchObject({
      id: 'profile-b',
      name: 'Claude Beta',
    })
  })
})
