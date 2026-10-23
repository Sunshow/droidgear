import { beforeEach, describe, expect, it, vi } from 'vitest'
import userEvent from '@testing-library/user-event'

const { commandMocks, onOpenChangeMock } = vi.hoisted(() => ({
  onOpenChangeMock: vi.fn(),
  commandMocks: {
    saveDshProvider: vi.fn(),
    saveDshCredentialRef: vi.fn(),
    saveDshConfigEntry: vi.fn(),
    fetchDshModels: vi.fn(),
    readDshCurrentConfig: vi.fn(),
    getDshConfigStatus: vi.fn(),
    readDshConfigSnapshot: vi.fn(),
  },
}))

vi.mock('@/lib/bindings', () => ({ commands: commandMocks }))

vi.mock('@/components/channels/ChannelModelPickerDialog', () => ({
  ChannelModelPickerDialog: () => null,
}))

import { render, screen, waitFor } from '@/test/test-utils'
import { useDshStore } from '@/store/dsh-store'
import type { DshProviderConfig } from '@/lib/bindings'
import { ProviderDialog } from './ProviderDialog'

function providerConfig(
  overrides: Partial<DshProviderConfig> = {}
): DshProviderConfig {
  return {
    displayName: '',
    baseURL: '',
    api: '',
    models: [],
    ...overrides,
  }
}

describe('ProviderDialog credential reference', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    useDshStore.setState({
      providers: {},
      credentials: {},
      profile: 'desktop',
      isLoading: false,
      error: null,
      configStatus: null,
      snapshot: null,
      isLoadingSnapshot: false,
      savingEntryId: null,
    })
    commandMocks.saveDshProvider.mockResolvedValue({ status: 'ok', data: null })
    commandMocks.saveDshCredentialRef.mockResolvedValue({
      status: 'ok',
      data: null,
    })
    commandMocks.saveDshConfigEntry.mockResolvedValue({
      status: 'ok',
      data: { patchHash: 'def456', changed: true },
    })
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
      data: null,
    })
  })

  it('derives the reference from the provider id and tracks edits', async () => {
    const user = userEvent.setup()
    render(
      <ProviderDialog
        open
        onOpenChange={onOpenChangeMock}
        editingProviderId={null}
      />
    )

    // Without a provider id there is no derived reference at all.
    expect(screen.queryByText(/Credential ref:/)).not.toBeInTheDocument()
    expect(
      screen.queryByLabelText('Credential reference')
    ).not.toBeInTheDocument()

    const providerIdInput = screen.getByLabelText('Provider ID')
    await user.type(providerIdInput, 'my-provider')
    expect(
      screen.getByText('Credential ref: MY_PROVIDER_API_KEY')
    ).toBeInTheDocument()

    await user.clear(providerIdInput)
    await user.type(providerIdInput, 'other-provider')
    expect(
      screen.getByText('Credential ref: OTHER_PROVIDER_API_KEY')
    ).toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: 'Save' }))

    await waitFor(() => {
      expect(commandMocks.saveDshProvider).toHaveBeenCalledWith(
        'desktop',
        'other-provider',
        expect.objectContaining({ apiKeyEnv: 'OTHER_PROVIDER_API_KEY' })
      )
    })
    expect(commandMocks.saveDshCredentialRef).toHaveBeenCalledWith(
      'OTHER_PROVIDER_API_KEY',
      ''
    )
  })

  it('lets the user customize the reference name', async () => {
    const user = userEvent.setup()
    render(
      <ProviderDialog
        open
        onOpenChange={onOpenChangeMock}
        editingProviderId={null}
      />
    )

    await user.type(screen.getByLabelText('Provider ID'), 'gateway')
    await user.click(screen.getByRole('button', { name: 'Customize' }))
    const referenceInput = screen.getByLabelText('Credential reference')
    await user.clear(referenceInput)
    await user.type(referenceInput, 'MY_GATEWAY_KEY')
    await user.type(screen.getByLabelText('API Key Value'), 'sk-test')

    expect(
      screen.getByText('Credential ref: MY_GATEWAY_KEY')
    ).toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: 'Save' }))

    await waitFor(() => {
      expect(commandMocks.saveDshProvider).toHaveBeenCalledWith(
        'desktop',
        'gateway',
        expect.objectContaining({ apiKeyEnv: 'MY_GATEWAY_KEY' })
      )
    })
    expect(commandMocks.saveDshCredentialRef).toHaveBeenCalledWith(
      'MY_GATEWAY_KEY',
      'sk-test'
    )
  })

  it('auto-expands a stored custom reference when editing', () => {
    useDshStore.setState({
      providers: {
        byok: providerConfig({ apiKeyEnv: 'CUSTOM_KEY' }),
      },
      credentials: { CUSTOM_KEY: 'sk-existing' },
    })

    render(
      <ProviderDialog
        open
        onOpenChange={onOpenChangeMock}
        editingProviderId="byok"
      />
    )

    expect(screen.getByLabelText('Credential reference')).toHaveValue(
      'CUSTOM_KEY'
    )
    expect(screen.getByLabelText('API Key Value')).toHaveValue('sk-existing')
  })

  it('warns about characters the backend would reject', async () => {
    const user = userEvent.setup()
    render(
      <ProviderDialog
        open
        onOpenChange={onOpenChangeMock}
        editingProviderId={null}
      />
    )

    await user.type(screen.getByLabelText('Provider ID'), 'gateway')
    await user.click(screen.getByRole('button', { name: 'Customize' }))
    await user.type(screen.getByLabelText('Credential reference'), 'MY KEY')

    expect(
      screen.getByText(
        'The reference may contain only letters, digits, underscores, and hyphens.'
      )
    ).toBeInTheDocument()
  })

  it('clears the stored credential when the key value is emptied', async () => {
    const user = userEvent.setup()
    useDshStore.setState({
      providers: {
        byok: providerConfig({ apiKeyEnv: 'CUSTOM_KEY' }),
      },
      credentials: { CUSTOM_KEY: 'sk-existing' },
    })
    render(
      <ProviderDialog
        open
        onOpenChange={onOpenChangeMock}
        editingProviderId="byok"
      />
    )

    const keyInput = screen.getByLabelText('API Key Value')
    await user.clear(keyInput)
    await user.click(screen.getByRole('button', { name: 'Save' }))

    await waitFor(() => {
      expect(commandMocks.saveDshCredentialRef).toHaveBeenCalledWith(
        'CUSTOM_KEY',
        ''
      )
    })
  })

  it('keeps a derived reference collapsed and unchanged when editing', async () => {
    const user = userEvent.setup()
    useDshStore.setState({
      providers: {
        byok: providerConfig({ apiKeyEnv: 'BYOK_API_KEY' }),
      },
    })

    render(
      <ProviderDialog
        open
        onOpenChange={onOpenChangeMock}
        editingProviderId="byok"
      />
    )

    expect(
      screen.queryByLabelText('Credential reference')
    ).not.toBeInTheDocument()
    expect(screen.getByText('Credential ref: BYOK_API_KEY')).toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: 'Save' }))

    await waitFor(() => {
      expect(commandMocks.saveDshProvider).toHaveBeenCalledWith(
        'desktop',
        'byok',
        expect.objectContaining({ apiKeyEnv: 'BYOK_API_KEY' })
      )
    })
  })
})
