import { beforeEach, describe, expect, it, vi } from 'vitest'
import userEvent from '@testing-library/user-event'

const { toastMock, commandMocks } = vi.hoisted(() => {
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
    commandMocks: {
      listClaudeProfiles: vi.fn(),
      createDefaultClaudeProfile: vi.fn(),
      getClaudeTemporaryRunPlan: vi.fn(),
      getActiveClaudeProfileId: vi.fn(),
      getClaudeConfigStatus: vi.fn(),
      readClaudeCurrentConfig: vi.fn(),
      saveClaudeProfile: vi.fn(),
      deleteClaudeProfile: vi.fn(),
      duplicateClaudeProfile: vi.fn(),
      applyClaudeProfile: vi.fn(),
      launchClaude: vi.fn(),
    },
  }
})

vi.mock('sonner', () => ({
  toast: toastMock,
}))

vi.mock('@/lib/bindings', () => ({
  commands: commandMocks,
}))

import { render, screen, waitFor } from '@/test/test-utils'
import { useClaudeStore } from '@/store/claude-store'
import { ClaudeConfigPage } from './ClaudeConfigPage'

const sampleProfiles = [
  {
    id: 'profile-a',
    name: 'Claude Alpha',
    description: 'Demo profile',
    baseUrl: 'https://proxy.example.com',
    bearerToken: 'token-a',
    model: 'claude-sonnet-4-5',
    smallModelUsesMainModel: true,
    smallModel: null,
    reasoningEffort: 'medium',
    thinkingMode: 'inherit',
    createdAt: '2026-01-01T00:00:00Z',
    updatedAt: '2026-01-01T00:00:00Z',
  },
]

describe('ClaudeConfigPage', () => {
  beforeEach(() => {
    vi.clearAllMocks()

    useClaudeStore.setState({
      profiles: [],
      activeProfileId: null,
      currentProfile: null,
      isLoading: false,
      error: null,
      configStatus: null,
    })

    commandMocks.listClaudeProfiles.mockResolvedValue({
      status: 'ok',
      data: sampleProfiles,
    })
    commandMocks.createDefaultClaudeProfile.mockResolvedValue({
      status: 'ok',
      data: sampleProfiles[0],
    })
    commandMocks.getClaudeTemporaryRunPlan.mockResolvedValue({
      status: 'ok',
      data: {
        program: 'claude',
        args: ['--settings', '<generated at launch>'],
        env: [],
        unsetEnv: [],
        secretEnvKeys: [],
        warnings: [],
      },
    })
    commandMocks.getActiveClaudeProfileId.mockResolvedValue({
      status: 'ok',
      data: 'profile-a',
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
        baseUrl: 'https://proxy.example.com',
        bearerToken: 'token-a',
        model: 'claude-sonnet-4-5',
        smallModelUsesMainModel: true,
        smallModel: null,
        reasoningEffort: 'medium',
        thinkingMode: 'inherit',
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
      data: sampleProfiles[0],
    })
    commandMocks.applyClaudeProfile.mockResolvedValue({
      status: 'ok',
      data: null,
    })
    commandMocks.launchClaude.mockResolvedValue({
      status: 'ok',
      data: null,
    })
  })

  it('launches Claude temporary run from the selected profile after saving current edits', async () => {
    const user = userEvent.setup()
    render(<ClaudeConfigPage />)

    await screen.findByRole('button', {
      name: 'Run Profile',
    })

    useClaudeStore.setState(state => ({
      currentProfile: state.currentProfile
        ? {
            ...state.currentProfile,
            model: 'claude-opus-4-1',
            bearerToken: 'token-updated',
          }
        : null,
    }))

    await user.click(
      await screen.findByRole('button', {
        name: 'Run Profile',
      })
    )

    await waitFor(() => {
      expect(commandMocks.saveClaudeProfile).toHaveBeenCalledWith(
        expect.objectContaining({
          id: 'profile-a',
          model: 'claude-opus-4-1',
          bearerToken: 'token-updated',
        })
      )
      expect(commandMocks.launchClaude).toHaveBeenCalledWith('profile-a')
    })

    const saveCallOrder =
      commandMocks.saveClaudeProfile.mock.invocationCallOrder[0]
    const launchCallOrder =
      commandMocks.launchClaude.mock.invocationCallOrder[0]
    if (saveCallOrder === undefined || launchCallOrder === undefined) {
      throw new Error('Expected save and launch call order to be recorded')
    }
    expect(saveCallOrder).toBeLessThan(launchCallOrder)
    expect(toastMock.success).toHaveBeenCalledWith(
      'Claude Code launched in a new terminal window'
    )
  })

  it('does not launch Claude when saving the current profile fails', async () => {
    const user = userEvent.setup()
    commandMocks.saveClaudeProfile.mockResolvedValue({
      status: 'error',
      error: 'Failed to save Claude profile to disk',
    })

    render(<ClaudeConfigPage />)

    await user.click(
      await screen.findByRole('button', {
        name: 'Run Profile',
      })
    )

    await waitFor(() => {
      expect(toastMock.error).toHaveBeenCalledWith(
        'Failed to save Claude profile to disk'
      )
    })
    expect(commandMocks.launchClaude).not.toHaveBeenCalled()
    expect(
      await screen.findByText('Failed to save Claude profile to disk')
    ).toBeInTheDocument()
  })

  it('shows a visible error when default profile bootstrap fails on first load', async () => {
    commandMocks.listClaudeProfiles.mockResolvedValueOnce({
      status: 'ok',
      data: [],
    })
    commandMocks.createDefaultClaudeProfile.mockResolvedValueOnce({
      status: 'error',
      error: 'Claude profile directory is not writable',
    })
    commandMocks.getActiveClaudeProfileId.mockResolvedValueOnce({
      status: 'ok',
      data: null,
    })

    render(<ClaudeConfigPage />)

    expect(
      await screen.findByText('Claude profile directory is not writable')
    ).toBeInTheDocument()
  })

  it('warns when the selected model looks like an opaque custom deployment id', async () => {
    commandMocks.listClaudeProfiles.mockResolvedValueOnce({
      status: 'ok',
      data: [
        {
          ...sampleProfiles[0],
          model: 'gateway-prod-model',
        },
      ],
    })

    render(<ClaudeConfigPage />)

    expect(await screen.findByText('Capability warning')).toBeInTheDocument()
    expect(
      screen.getByText(
        'The model name "gateway-prod-model" does not look like an official Claude model id. Claude Code may ignore Reasoning Effort or Thinking Mode for custom deployment names and gateway aliases.'
      )
    ).toBeInTheDocument()
  })

  it('shows a friendly missing-cli message instead of the raw probe failure', async () => {
    const user = userEvent.setup()
    commandMocks.launchClaude.mockResolvedValue({
      status: 'error',
      error: 'Failed to execute claude --version: No such file or directory',
    })

    render(<ClaudeConfigPage />)

    await user.click(
      await screen.findByRole('button', {
        name: 'Run Profile',
      })
    )

    await waitFor(() => {
      expect(toastMock.error).toHaveBeenCalledWith(
        'Claude CLI is not installed or not available in PATH.'
      )
    })
    expect(
      await screen.findByText(
        'Claude CLI is not installed or not available in PATH.'
      )
    ).toBeInTheDocument()
  })

  it('surfaces temporary-run preview warnings before launching Claude', async () => {
    const user = userEvent.setup()
    commandMocks.getClaudeTemporaryRunPlan.mockResolvedValue({
      status: 'ok',
      data: {
        program: 'claude',
        args: ['--settings', '<generated at launch>'],
        env: [],
        unsetEnv: [],
        secretEnvKeys: [],
        warnings: [
          'Failed to copy inherited CLAUDE_ENV_FILE from /tmp/live.env: permission denied. Claude temporary run will continue without inheriting that runtime env file.',
        ],
      },
    })

    render(<ClaudeConfigPage />)

    await user.click(
      await screen.findByRole('button', {
        name: 'Run Profile',
      })
    )

    await waitFor(() => {
      expect(toastMock.warning).toHaveBeenCalledWith(
        'Failed to copy inherited CLAUDE_ENV_FILE from /tmp/live.env: permission denied. Claude temporary run will continue without inheriting that runtime env file.'
      )
    })
    expect(commandMocks.launchClaude).toHaveBeenCalledWith('profile-a')
  })
})
