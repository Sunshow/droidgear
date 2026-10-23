import { beforeEach, describe, expect, it, vi } from 'vitest'
import userEvent from '@testing-library/user-event'

const { storeMocks } = vi.hoisted(() => ({
  storeMocks: {
    error: null as string | null,
    setError: vi.fn(),
    setProfile: vi.fn(),
    loadProviders: vi.fn(),
    loadCredentials: vi.fn(),
    loadConfigStatus: vi.fn(),
    loadSnapshot: vi.fn(),
  },
}))

vi.mock('@/store/dsh-store', () => ({
  useDshStore: (selector: (state: typeof storeMocks) => unknown) =>
    selector(storeMocks),
}))

vi.mock('./DshOverviewTab', () => ({
  DshOverviewTab: () => <div>overview content</div>,
}))
vi.mock('./DshModelsTab', () => ({
  DshModelsTab: () => <div>models content</div>,
}))
vi.mock('./DshGeneralTab', () => ({
  DshGeneralTab: () => <div>general content</div>,
}))
vi.mock('./DshAgentTab', () => ({
  DshAgentTab: () => <div>agent content</div>,
}))
vi.mock('./DshNetworkTab', () => ({
  DshNetworkTab: () => <div>network content</div>,
}))
vi.mock('./DshAdvancedTab', () => ({
  DshAdvancedTab: () => <div>advanced content</div>,
}))
vi.mock('./DshDiagnosticsTab', () => ({
  DshDiagnosticsTab: () => <div>diagnostics content</div>,
}))

import { render, screen } from '@/test/test-utils'
import { DshConfigPage } from './DshConfigPage'

describe('DshConfigPage', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    storeMocks.error = null
  })

  it('loads the profile and switches between all seven tabs', async () => {
    const user = userEvent.setup()
    render(<DshConfigPage profile="desktop" />)

    expect(storeMocks.setProfile).toHaveBeenCalledWith('desktop')
    expect(storeMocks.loadSnapshot).toHaveBeenCalled()

    for (const label of [
      'Overview',
      'Models',
      'General',
      'Agent',
      'Network',
      'Advanced',
      'Diagnostics',
    ]) {
      expect(screen.getByRole('tab', { name: label })).toBeInTheDocument()
    }

    expect(screen.getByText('overview content')).toBeInTheDocument()

    await user.click(screen.getByRole('tab', { name: 'General' }))
    expect(screen.getByText('general content')).toBeInTheDocument()
    expect(
      screen.getByText('Language, theme, chat display, and developer tools')
    ).toBeInTheDocument()

    await user.click(screen.getByRole('tab', { name: 'Advanced' }))
    expect(screen.getByText('advanced content')).toBeInTheDocument()
  })

  it('refreshes every slice from the header button', async () => {
    const user = userEvent.setup()
    render(<DshConfigPage profile="web" />)

    await user.click(screen.getByRole('button', { name: 'Refresh' }))

    expect(storeMocks.loadProviders).toHaveBeenCalledTimes(2)
    expect(storeMocks.loadSnapshot).toHaveBeenCalledTimes(2)
  })

  it('renders the store error with a dismiss action', async () => {
    const user = userEvent.setup()
    storeMocks.error = 'DSH_CONFIG_CONFLICT'
    render(<DshConfigPage profile="desktop" />)

    expect(screen.getByText('DSH_CONFIG_CONFLICT')).toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'Dismiss' }))
    expect(storeMocks.setError).toHaveBeenCalledWith(null)
  })
})
