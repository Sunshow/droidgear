import { beforeEach, describe, expect, it, vi } from 'vitest'
import userEvent from '@testing-library/user-event'

const { commandMocks } = vi.hoisted(() => ({
  commandMocks: {
    getChannelApiKey: vi.fn(),
    fetchModelsByApiKey: vi.fn(),
    prepareOmpChannelImport: vi.fn(),
  },
}))

vi.mock('@/lib/bindings', () => ({
  commands: commandMocks,
}))

import type { Channel } from '@/lib/bindings'
import { useChannelStore } from '@/store/channel-store'
import { render, screen, waitFor } from '@/test/test-utils'
import { OmpImportFromChannelDialog } from './OmpImportFromChannelDialog'

const channel: Channel = {
  id: 'channel-1',
  name: 'My API',
  type: 'general',
  baseUrl: 'https://api.example.com/v1',
  enabled: true,
  createdAt: 0,
}

function setChannels(channels: Channel[]) {
  useChannelStore.setState({
    channels,
    keys: {},
    keysFetchState: {},
  })
}

describe('OmpImportFromChannelDialog', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    setChannels([])
    commandMocks.prepareOmpChannelImport.mockImplementation(
      async (input: {
        baseUrl: string
        apiKey: string
        providerId?: string
        modelIds: string[]
        existing?: { models?: { id: string }[]; auth?: string } | null
      }) => ({
        status: 'ok',
        data: {
          providerId: input.providerId || 'my-api',
          provider: {
            ...(input.existing ?? {}),
            baseUrl: input.baseUrl,
            api: 'openai-completions',
            apiKey: input.apiKey,
            auth: input.existing?.auth ?? 'apiKey',
            models:
              input.modelIds.length > 0
                ? input.modelIds.map(id => ({ id }))
                : (input.existing?.models ?? []),
          },
        },
      })
    )
  })

  it('imports a provider from an api-key channel', async () => {
    const user = userEvent.setup({ delay: null })
    const onImported = vi.fn().mockResolvedValue(undefined)
    setChannels([channel])
    commandMocks.getChannelApiKey.mockResolvedValue({
      status: 'ok',
      data: 'sk-test',
    })
    commandMocks.fetchModelsByApiKey.mockResolvedValue({
      status: 'ok',
      data: [{ id: 'gpt-5.2', name: 'GPT-5.2' }],
    })

    render(
      <OmpImportFromChannelDialog
        open
        onOpenChange={vi.fn()}
        existingProviders={[]}
        onImported={onImported}
      />
    )

    await user.click(screen.getByRole('combobox'))
    await user.click(await screen.findByRole('option', { name: /My API/ }))
    await user.click(screen.getByRole('button', { name: 'Next' }))

    const providerIdInput = await screen.findByDisplayValue('my-api')
    expect(providerIdInput).toBeEnabled()

    await waitFor(() =>
      expect(commandMocks.fetchModelsByApiKey).toHaveBeenCalledWith(
        'https://api.example.com/v1',
        'sk-test',
        null
      )
    )

    await user.click(screen.getByRole('button', { name: 'Import' }))

    await waitFor(() => expect(onImported).toHaveBeenCalledTimes(1))
    expect(onImported).toHaveBeenCalledWith(
      'my-api',
      expect.objectContaining({
        baseUrl: 'https://api.example.com/v1',
        api: 'openai-completions',
        apiKey: 'sk-test',
        auth: 'apiKey',
        models: [expect.objectContaining({ id: 'gpt-5.2' })],
      })
    )
  })

  it('updates an existing provider with a locked ID and merged models', async () => {
    const user = userEvent.setup({ delay: null })
    const onImported = vi.fn().mockResolvedValue(undefined)
    setChannels([channel])
    commandMocks.getChannelApiKey.mockResolvedValue({
      status: 'ok',
      data: 'sk-new',
    })
    commandMocks.fetchModelsByApiKey.mockResolvedValue({
      status: 'ok',
      data: [],
    })

    render(
      <OmpImportFromChannelDialog
        open
        onOpenChange={vi.fn()}
        existingProviders={[
          {
            providerId: 'existing-provider',
            config: {
              apiKey: 'sk-old',
              models: [{ id: 'kept-model', name: 'Kept' }],
              headers: { 'X-Custom': 'keep' },
            },
          },
        ]}
        targetProviderId="existing-provider"
        onImported={onImported}
      />
    )

    await user.click(screen.getByRole('combobox'))
    await user.click(await screen.findByRole('option', { name: /My API/ }))
    await user.click(screen.getByRole('button', { name: 'Next' }))

    const providerIdInput = await screen.findByDisplayValue('existing-provider')
    expect(providerIdInput).toBeDisabled()

    await user.click(screen.getByRole('button', { name: 'Update' }))

    await waitFor(() => expect(onImported).toHaveBeenCalledTimes(1))
    expect(onImported).toHaveBeenCalledWith(
      'existing-provider',
      expect.objectContaining({
        apiKey: 'sk-new',
        headers: { 'X-Custom': 'keep' },
        models: [expect.objectContaining({ id: 'kept-model' })],
      })
    )
  })

  it('hints when no channels are configured', () => {
    render(
      <OmpImportFromChannelDialog
        open
        onOpenChange={vi.fn()}
        existingProviders={[]}
        onImported={vi.fn()}
      />
    )

    expect(screen.getByText(/No channels available/i)).toBeInTheDocument()
  })
})
