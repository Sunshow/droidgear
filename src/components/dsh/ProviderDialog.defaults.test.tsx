import { beforeEach, describe, expect, it, vi } from 'vitest'
import userEvent from '@testing-library/user-event'

const { commandMocks, onOpenChangeMock } = vi.hoisted(() => ({
  onOpenChangeMock: vi.fn(),
  commandMocks: {
    saveDshProvider: vi.fn(),
    saveDshCredentialRef: vi.fn(),
    saveDshConfigEntry: vi.fn(),
    fetchDshModels: vi.fn(),
  },
}))

vi.mock('@/lib/bindings', () => ({ commands: commandMocks }))

vi.mock('@/components/channels/ChannelModelPickerDialog', () => ({
  ChannelModelPickerDialog: () => null,
}))

import { render, screen, waitFor } from '@/test/test-utils'
import { useDshStore } from '@/store/dsh-store'
import { makeEntry, makeField, makeSnapshot } from './dsh-test-fixtures'
import { ProviderDialog } from './ProviderDialog'

const snapshot = makeSnapshot({
  entries: [
    makeEntry({
      id: 'agent-default-model',
      config: { provider: 'other', model: 'x' },
      fields: [
        makeField({ name: 'provider', value: 'other' }),
        makeField({ name: 'model', value: 'x' }),
      ],
    }),
  ],
})

describe('ProviderDialog defaults', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    useDshStore.setState({
      providers: {},
      credentials: {},
      profile: 'desktop',
      isLoading: false,
      error: null,
      configStatus: null,
      snapshot,
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
  })

  it('suggests high for registry models and saves reasoning + default model', async () => {
    const user = userEvent.setup()
    render(
      <ProviderDialog
        open
        onOpenChange={onOpenChangeMock}
        editingProviderId={null}
      />
    )

    await user.type(screen.getByLabelText('Provider ID'), 'byok')
    await user.click(screen.getByRole('button', { name: 'Add Model' }))
    await user.type(
      screen.getAllByPlaceholderText('Model ID')[0] as HTMLElement,
      'gpt-5'
    )

    // Suggested from the registry capabilities of gpt-5.
    expect(screen.getByLabelText('Default reasoning effort')).toHaveTextContent(
      'High'
    )
    expect(screen.getByRole('checkbox', { name: 'Text' })).toBeDisabled()
    expect(screen.getByRole('checkbox', { name: 'Image' })).toBeChecked()
    expect(
      screen.getByText(
        'Suggested High from the current models; you can change it'
      )
    ).toBeInTheDocument()

    await user.click(
      screen.getByRole('button', { name: 'Set as default model' })
    )
    await user.click(screen.getByRole('button', { name: 'Save' }))

    await waitFor(() => {
      expect(commandMocks.saveDshProvider).toHaveBeenCalledWith(
        'desktop',
        'byok',
        expect.objectContaining({
          reasoning: 'high',
          models: [
            expect.objectContaining({ id: 'gpt-5', input: ['text', 'image'] }),
          ],
        })
      )
    })
    await waitFor(() => {
      expect(commandMocks.saveDshConfigEntry).toHaveBeenCalledWith(
        'desktop',
        'agent-default-model',
        { provider: 'byok', model: 'gpt-5', reasoningEffort: 'high' },
        'abc123'
      )
    })
    expect(onOpenChangeMock).toHaveBeenCalledWith(false)
  })

  it('lets the user turn image support off for a custom gateway', async () => {
    const user = userEvent.setup()
    render(
      <ProviderDialog
        open
        onOpenChange={onOpenChangeMock}
        editingProviderId={null}
      />
    )

    await user.type(screen.getByLabelText('Provider ID'), 'byok')
    await user.click(screen.getByRole('button', { name: 'Add Model' }))
    await user.type(
      screen.getAllByPlaceholderText('Model ID')[0] as HTMLElement,
      'gpt-5'
    )

    // Registry suggests image, then the user disables it.
    await user.click(screen.getByRole('checkbox', { name: 'Image' }))
    expect(screen.getByRole('checkbox', { name: 'Image' })).not.toBeChecked()
    await user.click(screen.getByRole('button', { name: 'Save' }))

    await waitFor(() => {
      expect(commandMocks.saveDshProvider).toHaveBeenCalledWith(
        'desktop',
        'byok',
        expect.objectContaining({
          models: [expect.objectContaining({ id: 'gpt-5', input: ['text'] })],
        })
      )
    })
  })

  it('omits reasoningEffort when the default level is not set', async () => {
    const user = userEvent.setup()
    render(
      <ProviderDialog
        open
        onOpenChange={onOpenChangeMock}
        editingProviderId={null}
      />
    )

    await user.type(screen.getByLabelText('Provider ID'), 'byok')
    await user.click(screen.getByRole('button', { name: 'Add Model' }))
    await user.type(
      screen.getAllByPlaceholderText('Model ID')[0] as HTMLElement,
      'unknown-model'
    )
    await user.click(
      screen.getByRole('button', { name: 'Set as default model' })
    )
    await user.click(screen.getByRole('button', { name: 'Save' }))

    await waitFor(() => {
      expect(commandMocks.saveDshProvider).toHaveBeenCalledWith(
        'desktop',
        'byok',
        expect.objectContaining({ reasoning: null })
      )
    })
    await waitFor(() => {
      expect(commandMocks.saveDshConfigEntry).toHaveBeenCalledWith(
        'desktop',
        'agent-default-model',
        { provider: 'byok', model: 'unknown-model' },
        'abc123'
      )
    })
  })

  it('keeps the dialog open when only the default-model write fails', async () => {
    const user = userEvent.setup()
    commandMocks.saveDshConfigEntry.mockResolvedValue({
      status: 'error',
      error: 'DSH_CONFIG_CONFLICT: stale',
    })
    render(
      <ProviderDialog
        open
        onOpenChange={onOpenChangeMock}
        editingProviderId={null}
      />
    )

    await user.type(screen.getByLabelText('Provider ID'), 'byok')
    await user.click(screen.getByRole('button', { name: 'Add Model' }))
    await user.type(
      screen.getAllByPlaceholderText('Model ID')[0] as HTMLElement,
      'gpt-5'
    )
    await user.click(
      screen.getByRole('button', { name: 'Set as default model' })
    )
    await user.click(screen.getByRole('button', { name: 'Save' }))

    await waitFor(() => {
      expect(commandMocks.saveDshConfigEntry).toHaveBeenCalled()
    })
    expect(onOpenChangeMock).not.toHaveBeenCalledWith(false)
    expect(
      screen.getByText(
        'The config was changed by DSH or another tool. Refresh and try again.'
      )
    ).toBeInTheDocument()
  })
})
