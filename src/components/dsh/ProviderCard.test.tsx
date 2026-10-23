import { beforeEach, describe, expect, it, vi } from 'vitest'

vi.mock('@/lib/bindings', () => ({ commands: {} }))

import { render, screen } from '@/test/test-utils'
import { useDshStore } from '@/store/dsh-store'
import type { DshProviderConfig } from '@/lib/bindings'
import { ProviderCard } from './ProviderCard'

const config: DshProviderConfig = {
  displayName: 'My BYOK',
  baseURL: 'https://api.example.com/v1',
  apiKeyEnv: 'MY_KEY',
  api: 'openai-completions',
  models: [{ id: 'gpt-5' }],
}

function renderCard(
  overrides: {
    defaultModel?: { model: string; reasoningEffort: string | null } | null
  } = {}
) {
  return render(
    <ProviderCard
      providerId="my-provider"
      config={config}
      defaultModel={overrides.defaultModel ?? null}
      onEdit={vi.fn()}
      onDelete={vi.fn()}
    />
  )
}

describe('ProviderCard', () => {
  beforeEach(() => {
    useDshStore.setState({ credentials: { MY_KEY: 'sk-test' } })
  })

  it('renders the provider details and API key state', () => {
    renderCard()

    expect(screen.getByText('my-provider')).toBeInTheDocument()
    expect(screen.getByText('My BYOK')).toBeInTheDocument()
    expect(screen.getByText('https://api.example.com/v1')).toBeInTheDocument()
    expect(screen.getByText('API Key configured')).toBeInTheDocument()
  })

  it('shows the default-model badge and summary', () => {
    renderCard({ defaultModel: { model: 'gpt-5', reasoningEffort: 'high' } })

    expect(screen.getByText('Default model')).toBeInTheDocument()
    expect(screen.getByText('Default model: gpt-5 (High)')).toBeInTheDocument()
  })

  it('keeps unknown effort values verbatim', () => {
    renderCard({ defaultModel: { model: 'gpt-5', reasoningEffort: 'custom' } })

    expect(
      screen.getByText('Default model: gpt-5 (custom)')
    ).toBeInTheDocument()
  })

  it('omits the default badge when the provider is not the default', () => {
    renderCard()

    expect(screen.queryByText('Default model')).not.toBeInTheDocument()
    expect(screen.queryByText(/Default model:/)).not.toBeInTheDocument()
  })
})
