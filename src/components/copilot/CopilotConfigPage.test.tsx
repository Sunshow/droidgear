import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { open } from '@tauri-apps/plugin-dialog'
import { commands, type CopilotProfile } from '@/lib/bindings'
import { useCopilotStore } from '@/store/copilot-store'
import { useChannelStore } from '@/store/channel-store'
import { CopilotConfigPage } from './CopilotConfigPage'

vi.mock('react-i18next', () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn() }))
vi.mock('sonner', () => ({ toast: { success: vi.fn(), error: vi.fn() } }))
vi.mock('@/lib/bindings', () => ({
  commands: {
    listCopilotProfiles: vi.fn(),
    getActiveCopilotProfileId: vi.fn(),
    getCopilotConfigStatus: vi.fn(),
    saveCopilotProfile: vi.fn(),
    applyCopilotProfile: vi.fn(),
    launchCopilot: vi.fn(),
    prepareCopilotChannelImport: vi.fn(),
    fetchModelsByApiKey: vi.fn(),
  },
}))

const profile: CopilotProfile = {
  id: 'profile',
  name: 'Provider',
  description: null,
  createdAt: '',
  updatedAt: '',
  useOfficialAuth: false,
  baseUrl: 'https://example.test/v1',
  providerType: 'openai',
  apiKey: 'test-key',
  model: 'custom-model',
  maxPromptTokens: null,
  maxOutputTokens: null,
}

describe('Copilot temporary run', () => {
  beforeEach(() => {
    vi.resetAllMocks()
    useCopilotStore.setState({
      profiles: [],
      currentProfile: null,
      activeProfileId: null,
      isLoading: false,
      error: null,
      configStatus: null,
    })
    vi.mocked(commands.listCopilotProfiles).mockResolvedValue({
      status: 'ok',
      data: [profile],
    })
    vi.mocked(commands.getActiveCopilotProfileId).mockResolvedValue({
      status: 'ok',
      data: profile.id,
    })
    vi.mocked(commands.getCopilotConfigStatus).mockResolvedValue({
      status: 'ok',
      data: { configExists: true, configPath: '/test/config.env' },
    })
    vi.mocked(commands.saveCopilotProfile).mockResolvedValue({
      status: 'ok',
      data: null,
    })
    vi.mocked(commands.launchCopilot).mockResolvedValue({
      status: 'ok',
      data: null,
    })
    vi.mocked(open).mockResolvedValue('/test/project')
  })

  async function launch() {
    render(<CopilotConfigPage />)
    const button = screen.getByRole('button', {
      name: 'copilot.actions.launch',
    })
    await waitFor(() => expect(button).toBeEnabled())
    fireEvent.click(button)
  }

  it('saves edits and starts the selected profile in the chosen directory without applying', async () => {
    await launch()
    await waitFor(() =>
      expect(commands.launchCopilot).toHaveBeenCalledWith(
        profile.id,
        '/test/project'
      )
    )
    expect(commands.saveCopilotProfile).toHaveBeenCalledWith(profile)
    expect(commands.applyCopilotProfile).not.toHaveBeenCalled()
    expect(
      vi.mocked(commands.saveCopilotProfile).mock.invocationCallOrder[0]
    ).toBeLessThan(
      vi.mocked(commands.launchCopilot).mock.invocationCallOrder[0] ?? 0
    )
  })

  it('shows save failures and does not start a stale profile', async () => {
    vi.mocked(commands.saveCopilotProfile).mockResolvedValue({
      status: 'error',
      error: 'disk full',
    })
    await launch()
    await screen.findByText('disk full')
    expect(commands.launchCopilot).not.toHaveBeenCalled()
    expect(commands.applyCopilotProfile).not.toHaveBeenCalled()
  })

  it('does not save or launch when the directory picker is cancelled', async () => {
    vi.mocked(open).mockResolvedValue(null)
    await launch()
    await waitFor(() => expect(open).toHaveBeenCalled())
    await waitFor(() =>
      expect(
        screen.getByRole('button', { name: 'copilot.actions.launch' })
      ).toBeEnabled()
    )
    expect(commands.saveCopilotProfile).not.toHaveBeenCalled()
    expect(commands.launchCopilot).not.toHaveBeenCalled()
  })

  it.each(['openai', 'anthropic'] as const)(
    'imports a Sub2API %s model through the shared channel picker',
    async provider => {
      const user = userEvent.setup()
      useChannelStore.setState({
        channels: [
          {
            id: 'channel',
            name: 'Sub2API',
            type: 'sub-2-api',
            baseUrl: 'https://channel.test',
            enabled: true,
            createdAt: 0,
          },
        ],
        keys: {
          channel: [
            {
              id: 1,
              name: 'Selected key',
              key: 'channel-key',
              platform: provider,
              status: 1,
              remainQuota: 0,
              usedQuota: 0,
              unlimitedQuota: true,
              groupName: null,
            },
          ],
        },
      })
      vi.mocked(commands.fetchModelsByApiKey).mockResolvedValue({
        status: 'ok',
        data: [{ id: 'channel-model', name: null }],
      })
      const imported = {
        ...profile,
        providerType: provider,
        baseUrl:
          provider === 'openai'
            ? 'https://channel.test/v1'
            : 'https://channel.test',
        model: 'channel-model',
        apiKey: 'channel-key',
      }
      vi.mocked(commands.prepareCopilotChannelImport).mockResolvedValue({
        status: 'ok',
        data: imported,
      })
      render(<CopilotConfigPage />)
      await user.click(
        await screen.findByRole('button', {
          name: 'channels.importFromChannel',
        })
      )
      const dialog = screen.getByRole('dialog')
      await user.click(within(dialog).getByRole('combobox'))
      await user.click(screen.getByRole('option', { name: 'Sub2API' }))
      const keySelect = within(dialog).getAllByRole('combobox')[1]
      if (!keySelect) throw new Error('Expected API key selector')
      await user.click(keySelect)
      await user.click(screen.getByRole('option', { name: 'Selected key' }))
      await user.click(
        await screen.findByRole('radio', { name: 'channel-model' })
      )
      await waitFor(() =>
        expect(useCopilotStore.getState().currentProfile).toEqual(imported)
      )
      expect(commands.fetchModelsByApiKey).toHaveBeenCalledWith(
        'https://channel.test',
        'channel-key',
        provider
      )
      expect(commands.prepareCopilotChannelImport).toHaveBeenCalledWith(
        profile,
        expect.objectContaining({
          channelType: 'sub-2-api',
          baseUrl: 'https://channel.test',
          apiKey: 'channel-key',
          provider,
          model: 'channel-model',
        })
      )
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
      expect(screen.getByDisplayValue(imported.baseUrl)).toBeInTheDocument()
      expect(commands.applyCopilotProfile).not.toHaveBeenCalled()
      expect(commands.saveCopilotProfile).not.toHaveBeenCalled()
    }
  )
})
