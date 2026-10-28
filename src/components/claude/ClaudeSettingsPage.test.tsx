import { beforeEach, describe, expect, it } from 'vitest'
import userEvent from '@testing-library/user-event'
import { render, screen } from '@/test/test-utils'
import {
  type ClaudeSettingsDoc,
  useClaudeSettingsStore,
} from '@/store/claude-settings-store'
import { ClaudeSettingsPage } from './ClaudeSettingsPage'

function seed(doc: ClaudeSettingsDoc): void {
  useClaudeSettingsStore.setState({
    files: [],
    activeFile: {
      name: 'Global',
      path: '/home/tester/.claude/settings.json',
      isGlobal: true,
      isActive: true,
      exists: true,
    },
    currentJson: doc,
    hasChanges: false,
    isLoading: false,
    isLaunching: false,
    error: null,
  })
}

function currentDoc(): ClaudeSettingsDoc {
  return useClaudeSettingsStore.getState().currentJson ?? {}
}

async function pickAutoCompactOption(
  user: ReturnType<typeof userEvent.setup>,
  name: string
): Promise<void> {
  await user.click(screen.getByLabelText('Auto-compact window'))
  await user.click(await screen.findByRole('option', { name }))
}

describe('ClaudeSettingsPage auto-compact window', () => {
  beforeEach(() => seed({ env: { ANTHROPIC_MODEL: 'claude-opus-4-8' } }))

  it('writes a picked preset at the document root', async () => {
    const user = userEvent.setup()
    render(<ClaudeSettingsPage />)

    await pickAutoCompactOption(user, '1M (1,000,000)')

    expect(currentDoc().autoCompactWindow).toBe(1_000_000)
    expect(useClaudeSettingsStore.getState().hasChanges).toBe(true)
  })

  it('removes the key when switched back to auto', async () => {
    seed({ autoCompactWindow: 500_000 })
    const user = userEvent.setup()
    render(<ClaudeSettingsPage />)

    await pickAutoCompactOption(user, 'Auto (unset)')

    expect(currentDoc()).not.toHaveProperty('autoCompactWindow')
  })

  it('accepts a custom in-range value', async () => {
    const user = userEvent.setup()
    render(<ClaudeSettingsPage />)

    await pickAutoCompactOption(user, 'Custom')
    const input = screen.getByPlaceholderText('e.g. 500000')
    await user.type(input, '300000')

    expect(currentDoc().autoCompactWindow).toBe(300_000)
    expect(
      screen.queryByText(
        'Enter a whole number of tokens between 100,000 and 1,000,000.'
      )
    ).not.toBeInTheDocument()
  })

  it('flags out-of-range input without writing it', async () => {
    const user = userEvent.setup()
    render(<ClaudeSettingsPage />)

    await pickAutoCompactOption(user, 'Custom')
    await user.type(screen.getByPlaceholderText('e.g. 500000'), '200')

    expect(currentDoc()).not.toHaveProperty('autoCompactWindow')
    expect(
      screen.getByText(
        'Enter a whole number of tokens between 100,000 and 1,000,000.'
      )
    ).toBeInTheDocument()
  })
})
