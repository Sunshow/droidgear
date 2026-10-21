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

import { render, screen, waitFor } from '@/test/test-utils'
import { useDshStore } from '@/store/dsh-store'
import { makeEntry, makeField, makeSnapshot } from './dsh-test-fixtures'
import { DshGeneralTab } from './DshGeneralTab'

const snapshot = () =>
  makeSnapshot({
    entries: [
      makeEntry({
        id: 'ui-settings',
        config: { enabled: true },
        fields: [
          makeField({
            name: 'enabled',
            kind: 'boolean',
            value: true,
            default: true,
          }),
        ],
      }),
    ],
  })

describe('DshGeneralTab', () => {
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
    commandMocks.readDshConfigSnapshot.mockResolvedValue({
      status: 'ok',
      data: snapshot(),
    })
  })

  it('renders curated entries with localized field labels', () => {
    render(<DshGeneralTab />)

    expect(screen.getByText('Developer tools')).toBeInTheDocument()
    expect(screen.getByText('Show developer tools')).toBeInTheDocument()
    expect(screen.getByRole('switch')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled()
  })

  it('saves the complete effective config with the snapshot hash', async () => {
    const user = userEvent.setup()
    render(<DshGeneralTab />)

    await user.click(screen.getByRole('switch'))

    const save = screen.getByRole('button', { name: 'Save' })
    expect(save).toBeEnabled()
    await user.click(save)

    await waitFor(() => {
      expect(commandMocks.saveDshConfigEntry).toHaveBeenCalledWith(
        'desktop',
        'ui-settings',
        { enabled: false },
        'abc123'
      )
    })
    await waitFor(() => {
      expect(commandMocks.readDshConfigSnapshot).toHaveBeenCalledWith('desktop')
    })
  })

  it('renders enum controls for locale and C-tier curated fields', () => {
    useDshStore.setState({
      snapshot: makeSnapshot({
        entries: [
          makeEntry({
            id: 'locale',
            config: { preference: 'zh' },
            fields: [
              makeField({ name: 'preference', kind: 'string', value: 'zh' }),
            ],
          }),
          makeEntry({
            id: 'ui-conversation',
            tier: 'C',
            config: { busyEnter: 'queue' },
            fields: [
              makeField({
                name: 'busyEnter',
                kind: 'enum',
                value: 'queue',
                enumValues: ['queue', 'steer'],
                tier: 'C',
              }),
            ],
          }),
        ],
      }),
    })

    render(<DshGeneralTab />)

    const comboboxes = screen.getAllByRole('combobox')
    expect(comboboxes).toHaveLength(2)
    expect(comboboxes[0]).toHaveTextContent('中文')
    expect(comboboxes[1]).toHaveTextContent('Queue')
  })

  it('discards local edits without writing', async () => {
    const user = userEvent.setup()
    render(<DshGeneralTab />)

    await user.click(screen.getByRole('switch'))
    expect(screen.getByRole('button', { name: 'Save' })).toBeEnabled()

    await user.click(screen.getByRole('button', { name: 'Discard changes' }))

    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled()
    expect(commandMocks.saveDshConfigEntry).not.toHaveBeenCalled()
  })

  it('toasts and keeps the draft when the backend refuses the write', async () => {
    const user = userEvent.setup()
    commandMocks.saveDshConfigEntry.mockResolvedValue({
      status: 'error',
      error: 'DSH_CONFIG_CONFLICT: stale',
    })
    render(<DshGeneralTab />)

    await user.click(screen.getByRole('switch'))
    await user.click(screen.getByRole('button', { name: 'Save' }))

    await waitFor(() => {
      expect(useDshStore.getState().error).toBe(
        'The config was changed by DSH or another tool. Refresh and try again.'
      )
    })
    expect(screen.getByRole('button', { name: 'Save' })).toBeEnabled()
  })

  it('shows an empty state when no curated entry exists', () => {
    useDshStore.setState({ snapshot: makeSnapshot() })
    render(<DshGeneralTab />)

    expect(
      screen.getByText('No configuration entries to show')
    ).toBeInTheDocument()
  })
})
