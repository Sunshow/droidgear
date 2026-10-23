import { beforeEach, describe, expect, it, vi } from 'vitest'

const { commandMocks } = vi.hoisted(() => ({
  commandMocks: {
    readDshConfigSnapshot: vi.fn(),
    saveDshConfigEntry: vi.fn(),
    setDshConfigEntryDisabled: vi.fn(),
  },
}))

vi.mock('@/lib/bindings', () => ({ commands: commandMocks }))

import { act, render, screen } from '@/test/test-utils'
import { useDshStore } from '@/store/dsh-store'
import { makeSnapshot, makeWarning } from './dsh-test-fixtures'
import { DshOverviewTab } from './DshOverviewTab'

describe('DshOverviewTab', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    useDshStore.setState({
      profile: 'desktop',
      snapshot: null,
      isLoadingSnapshot: false,
      error: null,
    })
  })

  it('renders profile, versions, paths, and hash', () => {
    useDshStore.setState({
      snapshot: makeSnapshot({ installedVersion: '0.2.0' }),
    })

    render(<DshOverviewTab />)

    expect(screen.getByText('Config status')).toBeInTheDocument()
    expect(screen.getByText('desktop')).toBeInTheDocument()
    expect(screen.getByText('0.2.0')).toBeInTheDocument()
    expect(screen.getByText('0.2.1-rc.2')).toBeInTheDocument()
    expect(
      screen.getByText('/home/user/.dsh/profiles/desktop/cordis.patch.yml')
    ).toBeInTheDocument()
    expect(screen.getByText('abc123')).toBeInTheDocument()
    expect(
      screen.getAllByText('No migration issues found').length
    ).toBeGreaterThan(0)
  })

  it('renders migration warnings with i18n title, hint, and severity', () => {
    useDshStore.setState({
      snapshot: makeSnapshot({
        installedVersion: '0.2.0',
        warnings: [
          makeWarning({
            code: 'DSH_CONFIG_TOOLS_MODE_BOTH',
            severity: 'error',
            entryId: 'tools',
            message: 'backend message',
            hint: 'backend hint',
          }),
        ],
      }),
    })

    render(<DshOverviewTab />)

    // Localized copy wins over the backend English message.
    expect(screen.getByText('tools.mode is both')).toBeInTheDocument()
    expect(
      screen.getByText(/DSH 0.2.1 accepts only native or ptc/)
    ).toBeInTheDocument()
    expect(screen.getByText('Error')).toBeInTheDocument()
    expect(screen.getByText('DSH_CONFIG_TOOLS_MODE_BOTH')).toBeInTheDocument()
    expect(screen.getByText('tools')).toBeInTheDocument()
    expect(
      screen.getByText('Config baseline differs from installed version')
    ).toBeInTheDocument()
  })

  it('falls back to the backend copy for unknown warning codes', () => {
    useDshStore.setState({
      snapshot: makeSnapshot({
        warnings: [
          makeWarning({
            code: 'DSH_CONFIG_SOMETHING_NEW',
            severity: 'info',
            message: 'backend detail',
            hint: 'backend suggestion',
          }),
        ],
      }),
    })

    render(<DshOverviewTab />)

    expect(screen.getByText('backend detail')).toBeInTheDocument()
    expect(screen.getByText('backend suggestion')).toBeInTheDocument()
    expect(screen.getByText('Info')).toBeInTheDocument()
  })

  it('shows the loading and empty states without a snapshot', () => {
    useDshStore.setState({ isLoadingSnapshot: true })
    const { rerender } = render(<DshOverviewTab />)
    expect(screen.getByText('Loading configuration…')).toBeInTheDocument()

    act(() => {
      useDshStore.setState({ isLoadingSnapshot: false })
    })
    rerender(<DshOverviewTab />)
    expect(
      screen.getByText('No configuration snapshot yet')
    ).toBeInTheDocument()
  })
})
