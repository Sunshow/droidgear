import { beforeEach, describe, expect, it, vi } from 'vitest'
import userEvent from '@testing-library/user-event'

const { commandMocks } = vi.hoisted(() => ({
  commandMocks: {
    readDshConfigSnapshot: vi.fn(),
    saveDshConfigEntry: vi.fn(),
    setDshConfigEntryDisabled: vi.fn(),
  },
}))

vi.mock('@/lib/bindings', () => ({ commands: commandMocks }))

import { render, screen, waitFor, within } from '@/test/test-utils'
import { useDshStore } from '@/store/dsh-store'
import { makeEntry, makeField, makeSnapshot } from './dsh-test-fixtures'
import { DshAdvancedTab } from './DshAdvancedTab'

const snapshot = () =>
  makeSnapshot({
    entries: [
      makeEntry({
        id: 'bash-sandbox',
        tier: 'A',
        config: { timeoutMs: 60000 },
        fields: [
          makeField({
            name: 'timeoutMs',
            kind: 'number',
            value: 60000,
            default: 120000,
          }),
          makeField({
            name: 'graceMs',
            kind: 'number',
            value: 5000,
            volatile: false,
          }),
        ],
      }),
      makeEntry({
        id: 'webserver',
        tier: 'C',
        config: { port: 3000 },
        fields: [
          makeField({
            name: 'port',
            kind: 'number',
            value: 3000,
            tier: 'C',
          }),
        ],
      }),
      makeEntry({
        id: 'sandbox-policy',
        tier: 'A',
        hasJs: true,
        config: {
          mode: { $js: "process.env.DSH_PERMISSION_MODE ?? 'workspace-write'" },
        },
        fields: [
          makeField({
            name: 'mode',
            value: {
              $js: "process.env.DSH_PERMISSION_MODE ?? 'workspace-write'",
            },
          }),
        ],
      }),
      makeEntry({
        id: 'web-search-deepseek',
        tier: 'A',
        config: { apiKey: { $secret: true } },
        fields: [
          makeField({
            name: 'apiKey',
            kind: 'string',
            value: { $secret: true },
            secret: true,
          }),
        ],
      }),
    ],
  })

function cardFor(entryId: string): HTMLElement {
  const card = document.querySelector(`[data-entry-id="${entryId}"]`)
  if (!(card instanceof HTMLElement)) {
    throw new Error(`Card for ${entryId} not found`)
  }
  return card
}

describe('DshAdvancedTab', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    useDshStore.setState({
      profile: 'desktop',
      snapshot: snapshot(),
      isLoadingSnapshot: false,
      error: null,
      savingEntryId: null,
    })
    commandMocks.saveDshConfigEntry.mockResolvedValue({
      status: 'ok',
      data: { patchHash: 'def456', changed: true },
    })
    commandMocks.setDshConfigEntryDisabled.mockResolvedValue({
      status: 'ok',
      data: { patchHash: 'def456', changed: true },
    })
    commandMocks.readDshConfigSnapshot.mockResolvedValue({
      status: 'ok',
      data: snapshot(),
    })
  })

  it('renders editable A-tier fields and read-only C-tier fields', async () => {
    const user = userEvent.setup()
    render(<DshAdvancedTab />)

    await user.click(screen.getByRole('button', { name: 'bash-sandbox' }))
    const editableCard = cardFor('bash-sandbox')
    expect(within(editableCard).getAllByRole('spinbutton')).toHaveLength(2)
    expect(
      within(editableCard).getByText('Restart to apply')
    ).toBeInTheDocument()
    expect(
      within(editableCard).queryByText('Read-only')
    ).not.toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: 'webserver' }))
    const readOnlyCard = cardFor('webserver')
    expect(within(readOnlyCard).queryByRole('spinbutton')).toBeNull()
    expect(within(readOnlyCard).getByText('3000')).toBeInTheDocument()
    expect(within(readOnlyCard).getByText('Read-only')).toBeInTheDocument()
  })

  it('renders !!js and secret fields as read-only with badges', async () => {
    const user = userEvent.setup()
    render(<DshAdvancedTab />)

    await user.click(screen.getByRole('button', { name: /Sandbox policy/ }))
    const jsCard = cardFor('sandbox-policy')
    expect(
      within(jsCard).getAllByText('!!js expression (read-only)').length
    ).toBeGreaterThan(0)
    expect(
      within(jsCard).getByText(/!!js process\.env\.DSH_PERMISSION_MODE/)
    ).toBeInTheDocument()

    await user.click(screen.getByRole('button', { name: /Web search/ }))
    const secretCard = cardFor('web-search-deepseek')
    expect(
      within(secretCard).getByText('Secret (read-only)')
    ).toBeInTheDocument()
    expect(within(secretCard).getByText('••••••••')).toBeInTheDocument()
  })

  it('saves a modified field and shows the restart badge for non-volatile fields', async () => {
    const user = userEvent.setup()
    render(<DshAdvancedTab />)

    await user.click(screen.getByRole('button', { name: 'bash-sandbox' }))
    const card = cardFor('bash-sandbox')

    const save = within(card).getByRole('button', { name: 'Save' })
    expect(save).toBeDisabled()

    const input = within(card).getByLabelText('timeoutMs')
    await user.clear(input)
    await user.type(input, '30000')
    expect(save).toBeEnabled()
    await user.click(save)

    await waitFor(() => {
      expect(commandMocks.saveDshConfigEntry).toHaveBeenCalledWith(
        'desktop',
        'bash-sandbox',
        { timeoutMs: 30000 },
        'abc123'
      )
    })
  })

  it('toggles an entry disabled from the card header', async () => {
    const user = userEvent.setup()
    render(<DshAdvancedTab />)

    await user.click(screen.getByRole('button', { name: 'bash-sandbox' }))
    const card = cardFor('bash-sandbox')
    const disabledSwitch = within(card).getByRole('switch')
    await user.click(disabledSwitch)

    await waitFor(() => {
      expect(commandMocks.setDshConfigEntryDisabled).toHaveBeenCalledWith(
        'desktop',
        'bash-sandbox',
        true,
        'abc123'
      )
    })
  })
})
