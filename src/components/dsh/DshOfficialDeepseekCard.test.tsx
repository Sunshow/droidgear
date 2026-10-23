import { beforeEach, describe, expect, it, vi } from 'vitest'
import userEvent from '@testing-library/user-event'

const { storeMocks, saveEntryMock, saveCredentialMock } = vi.hoisted(() => {
  const saveEntry = vi.fn()
  const saveCredential = vi.fn()
  return {
    saveEntryMock: saveEntry,
    saveCredentialMock: saveCredential,
    storeMocks: {
      snapshot: null as unknown,
      credentials: {} as Record<string, string>,
      saveEntry,
      saveCredential,
    },
  }
})

vi.mock('@/store/dsh-store', () => ({
  useDshStore: (selector: (state: typeof storeMocks) => unknown) =>
    selector(storeMocks),
}))

import { render, screen, waitFor, within } from '@/test/test-utils'
import { makeEntry, makeField, makeSnapshot } from './dsh-test-fixtures'
import { DshOfficialDeepseekCard } from './DshOfficialDeepseekCard'

function snapshotWithConfig(
  config: Record<string, string>,
  fields: ReturnType<typeof makeField>[] = []
) {
  return makeSnapshot({
    entries: [makeEntry({ id: 'llm-deepseek', config, fields })],
  })
}

describe('DshOfficialDeepseekCard', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    storeMocks.snapshot = null
    storeMocks.credentials = {}
    saveEntryMock.mockResolvedValue(undefined)
    saveCredentialMock.mockResolvedValue(undefined)
  })

  it('shows the default reference and saves the key to credentials', async () => {
    storeMocks.snapshot = snapshotWithConfig({
      apiKeyEnv: 'DEEPSEEK_API_KEY',
    })
    const user = userEvent.setup()
    render(<DshOfficialDeepseekCard />)

    expect(screen.getByText('Not configured')).toBeInTheDocument()
    expect(
      screen.getByText('Credential ref: DEEPSEEK_API_KEY')
    ).toBeInTheDocument()
    expect(screen.getByText('Auto')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled()

    await user.type(screen.getByLabelText('API Key'), 'sk-x')
    await user.click(screen.getByRole('button', { name: 'Save' }))

    await waitFor(() => {
      expect(saveCredentialMock).toHaveBeenCalledWith(
        'DEEPSEEK_API_KEY',
        'sk-x'
      )
    })
    expect(saveEntryMock).not.toHaveBeenCalled()
  })

  it('shows the configured badge and clears the stored key after confirmation', async () => {
    storeMocks.snapshot = snapshotWithConfig({
      apiKeyEnv: 'DEEPSEEK_API_KEY',
    })
    storeMocks.credentials = { DEEPSEEK_API_KEY: 'sk-old' }
    const user = userEvent.setup()
    render(<DshOfficialDeepseekCard />)

    expect(screen.getByText('Configured')).toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: 'Clear' }))
    const dialog = await screen.findByRole('alertdialog')
    await user.click(within(dialog).getByRole('button', { name: 'Clear' }))

    await waitFor(() => {
      expect(saveCredentialMock).toHaveBeenCalledWith('DEEPSEEK_API_KEY', '')
    })
  })

  it('writes a custom reference to llm-deepseek before saving the key', async () => {
    storeMocks.snapshot = snapshotWithConfig({
      apiKeyEnv: 'DEEPSEEK_API_KEY',
      baseURL: 'https://api.deepseek.com',
    })
    const user = userEvent.setup()
    render(<DshOfficialDeepseekCard />)

    await user.type(screen.getByLabelText('API Key'), 'sk-x')
    await user.click(screen.getByRole('button', { name: 'Customize' }))
    const refInput = screen.getByLabelText('Reference name')
    await user.clear(refInput)
    await user.type(refInput, 'MY_KEY')

    expect(screen.getByText('Credential ref: MY_KEY')).toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: 'Save' }))

    await waitFor(() => {
      expect(saveEntryMock).toHaveBeenCalledWith('llm-deepseek', {
        apiKeyEnv: 'MY_KEY',
        baseURL: 'https://api.deepseek.com',
      })
      expect(saveCredentialMock).toHaveBeenCalledWith('MY_KEY', 'sk-x')
    })
    expect(saveEntryMock.mock.invocationCallOrder[0]).toBeLessThan(
      saveCredentialMock.mock.invocationCallOrder[0] as number
    )
  })

  it('rejects invalid custom references before saving', async () => {
    storeMocks.snapshot = snapshotWithConfig({
      apiKeyEnv: 'DEEPSEEK_API_KEY',
    })
    const user = userEvent.setup()
    render(<DshOfficialDeepseekCard />)

    await user.type(screen.getByLabelText('API Key'), 'sk-x')
    await user.click(screen.getByRole('button', { name: 'Customize' }))
    await user.type(screen.getByLabelText('Reference name'), 'MY KEY')

    expect(
      screen.getByText(
        'The reference may contain only letters, digits, underscores, and hyphens.'
      )
    ).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled()
    expect(saveEntryMock).not.toHaveBeenCalled()
    expect(saveCredentialMock).not.toHaveBeenCalled()
  })

  it('falls back to the field value and then the default reference', () => {
    storeMocks.snapshot = snapshotWithConfig({}, [
      makeField({ name: 'apiKeyEnv', value: 'FIELD_KEY' }),
    ])
    const { unmount } = render(<DshOfficialDeepseekCard />)
    expect(screen.getByText('Credential ref: FIELD_KEY')).toBeInTheDocument()

    unmount()
    storeMocks.snapshot = makeSnapshot()
    render(<DshOfficialDeepseekCard />)
    expect(
      screen.getByText('Credential ref: DEEPSEEK_API_KEY')
    ).toBeInTheDocument()
  })
})
