import { beforeEach, describe, expect, it, vi } from 'vitest'
import userEvent from '@testing-library/user-event'

const { commandMocks } = vi.hoisted(() => ({
  commandMocks: {
    testPiProviderConnection: vi.fn(),
  },
}))

vi.mock('@/lib/bindings', () => ({
  commands: commandMocks,
}))

import { render, screen, waitFor } from '@/test/test-utils'
import { ProviderCard } from './ProviderCard'

vi.stubGlobal(
  'ResizeObserver',
  class ResizeObserver {
    observe = vi.fn()
    unobserve = vi.fn()
    disconnect = vi.fn()
  }
)

const providerConfig = {
  baseUrl: 'https://api.example.com/v1',
  api: 'openai-completions',
  apiKey: 'sk-test',
  models: [{ id: 'test-model' }],
}

describe('Pi ProviderCard', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('tests the provider through the Pi command and shows success details', async () => {
    const user = userEvent.setup()
    commandMocks.testPiProviderConnection.mockResolvedValue({
      status: 'ok',
      data: {
        success: true,
        providerId: 'test-provider',
        modelId: 'test-model',
        latencyMs: 321,
        responseText: 'OK',
      },
    })

    render(
      <ProviderCard
        providerId="test-provider"
        config={providerConfig}
        onEdit={() => undefined}
        onDelete={() => undefined}
      />
    )

    const testButton = screen.getByRole('button', {
      name: 'Test Connection',
    })
    await user.click(testButton)

    expect(commandMocks.testPiProviderConnection).toHaveBeenCalledWith(
      'test-provider',
      providerConfig
    )
    await waitFor(() => expect(testButton).toBeEnabled())
    await user.unhover(testButton)
    await user.hover(testButton)
    const tooltips = await screen.findAllByRole('tooltip')
    const tooltip = tooltips.at(-1)
    expect(tooltip).toHaveTextContent('Connected')
    expect(tooltip).toHaveTextContent('Model ID: test-model')
    expect(tooltip).toHaveTextContent('Latency: 321ms')
    expect(tooltip).toHaveTextContent('Response: OK')
  })

  it.each([
    { name: 'missing provider configuration', config: undefined },
    { name: 'empty provider configuration', config: {} },
    {
      name: 'API key without models',
      config: { apiKey: 'sk-test' },
    },
    {
      name: 'OAuth without models',
      config: { oauth: 'radius' },
    },
    {
      name: 'built-in model overrides without custom models',
      config: {
        modelOverrides: { 'built-in-model': { contextWindow: 64000 } },
      },
    },
    { name: 'empty model list', config: { ...providerConfig, models: [] } },
    {
      name: 'blank model IDs',
      config: { ...providerConfig, models: [{ id: '  ' }] },
    },
  ])('renders safely and disables testing for $name', async ({ config }) => {
    const user = userEvent.setup()
    const onEdit = vi.fn()
    const onDelete = vi.fn()

    render(
      <ProviderCard
        providerId="empty-provider"
        config={config}
        onEdit={onEdit}
        onDelete={onDelete}
      />
    )

    expect(screen.getByText('empty-provider')).toBeInTheDocument()
    const testButton = screen.getByRole('button', { name: 'Test Connection' })
    expect(testButton).toBeDisabled()
    await user.click(testButton)
    expect(commandMocks.testPiProviderConnection).not.toHaveBeenCalled()

    await user.click(screen.getByRole('button', { name: 'Edit' }))
    expect(onEdit).toHaveBeenCalledOnce()
    await user.click(screen.getByRole('button', { name: 'Delete' }))
    expect(onDelete).toHaveBeenCalledOnce()
  })

  it('enables testing when a valid model follows a blank model ID', () => {
    render(
      <ProviderCard
        providerId="test-provider"
        config={{
          ...providerConfig,
          models: [{ id: '  ' }, { id: 'test-model' }],
        }}
        onEdit={() => undefined}
        onDelete={() => undefined}
      />
    )

    expect(
      screen.getByRole('button', { name: 'Test Connection' })
    ).toBeEnabled()
  })
})
