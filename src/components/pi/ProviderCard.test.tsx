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

  it('disables testing until the provider has a model', () => {
    render(
      <ProviderCard
        providerId="empty-provider"
        config={{ ...providerConfig, models: [] }}
        onEdit={() => undefined}
        onDelete={() => undefined}
      />
    )

    expect(
      screen.getByRole('button', { name: 'Test Connection' })
    ).toBeDisabled()
  })

  it('disables testing when every model ID is blank', () => {
    render(
      <ProviderCard
        providerId="empty-provider"
        config={{ ...providerConfig, models: [{ id: '  ' }] }}
        onEdit={() => undefined}
        onDelete={() => undefined}
      />
    )

    expect(
      screen.getByRole('button', { name: 'Test Connection' })
    ).toBeDisabled()
  })
})
