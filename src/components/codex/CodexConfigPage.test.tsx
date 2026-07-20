import { beforeEach, describe, expect, it, vi } from 'vitest'
import userEvent from '@testing-library/user-event'

const { toastMock, commandMocks, openMock } = vi.hoisted(() => {
  const toast = Object.assign(vi.fn(), {
    success: vi.fn(),
    error: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
    dismiss: vi.fn(),
    loading: vi.fn(),
  })

  return {
    toastMock: toast,
    openMock: vi.fn().mockResolvedValue('/home/user/projects'),
    commandMocks: {
      listCodexProfiles: vi.fn(),
      createDefaultCodexProfile: vi.fn(),
      getActiveCodexProfileId: vi.fn(),
      getCodexConfigStatus: vi.fn(),
      launchCodex: vi.fn(),
      applyCodexProfile: vi.fn(),
      readCodexCurrentConfig: vi.fn(),
      saveCodexProfile: vi.fn(),
      deleteCodexProfile: vi.fn(),
      duplicateCodexProfile: vi.fn(),
      listCodexAuthProfiles: vi.fn(),
    },
  }
})

vi.mock('sonner', () => ({
  toast: toastMock,
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: openMock,
}))

vi.mock('@/lib/bindings', () => ({
  commands: commandMocks,
}))

import { render, screen, waitFor } from '@/test/test-utils'
import { useCodexStore } from '@/store/codex-store'
import { CodexConfigPage } from './CodexConfigPage'

const sampleProfiles = [
  {
    id: 'profile-a',
    name: 'Profile A',
    description: 'Demo profile',
    createdAt: '2026-01-01T00:00:00Z',
    updatedAt: '2026-01-01T00:00:00Z',
    providers: {
      openai: {
        name: 'OpenAI',
        model: 'gpt-5.5',
        apiKey: null,
      },
    },
    modelProvider: 'openai',
    model: 'gpt-5.5',
    modelReasoningEffort: 'medium',
    apiKey: null,
  },
]

describe('CodexConfigPage', () => {
  beforeEach(() => {
    vi.clearAllMocks()

    openMock.mockResolvedValue('/home/user/projects')

    useCodexStore.setState({
      profiles: [],
      activeProfileId: null,
      currentProfile: null,
      isLoading: false,
      error: null,
      configStatus: null,
    })

    commandMocks.listCodexProfiles.mockResolvedValue({
      status: 'ok',
      data: sampleProfiles,
    })
    commandMocks.createDefaultCodexProfile.mockResolvedValue({
      status: 'ok',
      data: sampleProfiles[0],
    })
    commandMocks.getActiveCodexProfileId.mockResolvedValue({
      status: 'ok',
      data: 'profile-a',
    })
    commandMocks.getCodexConfigStatus.mockResolvedValue({
      status: 'ok',
      data: {
        authExists: true,
        configExists: true,
        authPath: '/home/user/.codex/auth.json',
        configPath: '/home/user/.codex/config.toml',
      },
    })
    commandMocks.launchCodex.mockResolvedValue({
      status: 'ok',
      data: null,
    })
    commandMocks.applyCodexProfile.mockResolvedValue({
      status: 'ok',
      data: null,
    })
    commandMocks.readCodexCurrentConfig.mockResolvedValue({
      status: 'ok',
      data: {
        providers: {},
        modelProvider: 'openai',
        model: 'gpt-5.5',
        modelReasoningEffort: null,
        apiKey: null,
      },
    })
    commandMocks.saveCodexProfile.mockResolvedValue({
      status: 'ok',
      data: null,
    })
    commandMocks.deleteCodexProfile.mockResolvedValue({
      status: 'ok',
      data: null,
    })
    commandMocks.duplicateCodexProfile.mockResolvedValue({
      status: 'ok',
      data: sampleProfiles[0],
    })
    commandMocks.listCodexAuthProfiles.mockResolvedValue({
      status: 'ok',
      data: {
        active: null,
        profiles: [
          {
            name: 'sub1',
            label: 'Subscription 1',
            createdAt: '2026-01-01T00:00:00Z',
            isOfficial: true,
            codexProfileId: null,
          },
        ],
        isCurrentOfficial: false,
      },
    })
  })

  it('launches Codex temporary run from the selected profile', async () => {
    const user = userEvent.setup()
    render(<CodexConfigPage />)

    await screen.findByRole('button', {
      name: 'Run Profile',
    })

    useCodexStore.setState(state => ({
      currentProfile: state.currentProfile
        ? {
            ...state.currentProfile,
            model: 'gpt-5.6',
            apiKey: 'sk-updated',
          }
        : null,
    }))

    const launchButton = await screen.findByRole('button', {
      name: 'Run Profile',
    })
    await user.click(launchButton)

    await waitFor(() => {
      expect(commandMocks.saveCodexProfile).toHaveBeenCalledWith(
        expect.objectContaining({
          id: 'profile-a',
          model: 'gpt-5.6',
          apiKey: 'sk-updated',
        })
      )
      expect(commandMocks.launchCodex).toHaveBeenCalledWith(
        'profile-a',
        '/home/user/projects'
      )
    })
    const saveCallOrder =
      commandMocks.saveCodexProfile.mock.invocationCallOrder[0]
    const launchCallOrder = commandMocks.launchCodex.mock.invocationCallOrder[0]
    if (saveCallOrder === undefined || launchCallOrder === undefined) {
      throw new Error('Expected save and launch call order to be recorded')
    }
    expect(saveCallOrder).toBeLessThan(launchCallOrder)
    expect(toastMock.success).toHaveBeenCalledWith(
      'Codex launched in a new terminal window'
    )
    expect(toastMock.error).not.toHaveBeenCalled()
  })

  it('does not launch Codex when saving the current profile fails', async () => {
    const user = userEvent.setup()
    commandMocks.saveCodexProfile.mockResolvedValue({
      status: 'error',
      error: 'Failed to save profile to disk',
    })

    render(<CodexConfigPage />)

    const launchButton = await screen.findByRole('button', {
      name: 'Run Profile',
    })
    await user.click(launchButton)

    await waitFor(() => {
      expect(toastMock.error).toHaveBeenCalledWith(
        'Failed to save profile to disk'
      )
    })
    expect(commandMocks.launchCodex).not.toHaveBeenCalled()
    expect(
      await screen.findByText('Failed to save profile to disk')
    ).toBeInTheDocument()
  })

  it('surfaces the raw Codex launch error when the backend reports failure', async () => {
    const user = userEvent.setup()
    commandMocks.launchCodex.mockResolvedValue({
      status: 'error',
      error: 'Failed to execute codex --version: No such file or directory',
    })

    render(<CodexConfigPage />)

    const launchButton = await screen.findByRole('button', {
      name: 'Run Profile',
    })
    await user.click(launchButton)

    await waitFor(() => {
      expect(toastMock.error).toHaveBeenCalledWith(
        'Failed to execute codex --version: No such file or directory'
      )
    })
    expect(
      await screen.findByText(
        'Failed to execute codex --version: No such file or directory'
      )
    ).toBeInTheDocument()
  })

  it('preserves arbitrary launch errors verbatim', async () => {
    const user = userEvent.setup()
    commandMocks.launchCodex.mockResolvedValue({
      status: 'error',
      error: 'Failed to launch preferred terminal: No such file or directory',
    })

    render(<CodexConfigPage />)

    const launchButton = await screen.findByRole('button', {
      name: 'Run Profile',
    })
    await user.click(launchButton)

    await waitFor(() => {
      expect(toastMock.error).toHaveBeenCalledWith(
        'Failed to launch preferred terminal: No such file or directory'
      )
    })
    expect(
      await screen.findByText(
        'Failed to launch preferred terminal: No such file or directory'
      )
    ).toBeInTheDocument()
  })

  it('does not launch Codex when the directory picker is cancelled', async () => {
    const user = userEvent.setup()
    openMock.mockResolvedValueOnce(null)

    render(<CodexConfigPage />)

    const launchButton = await screen.findByRole('button', {
      name: 'Run Profile',
    })
    await user.click(launchButton)

    await waitFor(() => {
      expect(openMock).toHaveBeenCalled()
    })
    expect(commandMocks.saveCodexProfile).not.toHaveBeenCalled()
    expect(commandMocks.launchCodex).not.toHaveBeenCalled()
  })

  it('hides Providers when model provider is openai and shows subscription hints', async () => {
    render(<CodexConfigPage />)

    await screen.findByText('Model Provider')
    expect(
      screen.getByText('Select openai to use an official ChatGPT subscription.')
    ).toBeInTheDocument()
    expect(
      screen.getByText(
        'If you see Error: account/read failed during TUI bootstrap, run: codex logout'
      )
    ).toBeInTheDocument()
    expect(screen.queryByText('Providers')).not.toBeInTheDocument()
    expect(screen.queryByText('Add Provider')).not.toBeInTheDocument()
    expect(await screen.findByText('Load Auth')).toBeInTheDocument()
  })

  it('shows Providers when model provider is custom', async () => {
    commandMocks.listCodexProfiles.mockResolvedValue({
      status: 'ok',
      data: [
        {
          ...sampleProfiles[0],
          modelProvider: 'custom',
          providers: {
            custom: {
              name: 'Custom',
              model: 'gpt-5.5',
              apiKey: null,
            },
          },
        },
      ],
    })

    render(<CodexConfigPage />)

    await screen.findByText('Providers')
    expect(screen.getByText('Add Provider')).toBeInTheDocument()
    expect(
      screen.queryByText(
        'If you see Error: account/read failed during TUI bootstrap, run: codex logout'
      )
    ).not.toBeInTheDocument()
  })
})
