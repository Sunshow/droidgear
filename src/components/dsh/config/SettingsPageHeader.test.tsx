import { describe, expect, it, vi } from 'vitest'
import userEvent from '@testing-library/user-event'
import { render, screen } from '@/test/test-utils'
import { SettingsPageHeader } from './SettingsPageHeader'

describe('SettingsPageHeader', () => {
  it('renders the title and description', () => {
    render(
      <SettingsPageHeader
        title="General"
        description="Language, theme, and chat display"
        onSave={vi.fn()}
        onReset={vi.fn()}
      />
    )

    expect(screen.getByRole('heading', { name: 'General' })).toBeInTheDocument()
    expect(
      screen.getByText('Language, theme, and chat display')
    ).toBeInTheDocument()
  })

  it('keeps both actions disabled while the page is clean', () => {
    render(
      <SettingsPageHeader title="General" onSave={vi.fn()} onReset={vi.fn()} />
    )

    expect(
      screen.getByRole('button', { name: 'Reset to default' })
    ).toBeDisabled()
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled()
  })

  it('calls onSave and onReset from the buttons', async () => {
    const user = userEvent.setup()
    const onSave = vi.fn()
    const onReset = vi.fn()
    render(
      <SettingsPageHeader
        title="General"
        dirty
        onSave={onSave}
        onReset={onReset}
      />
    )

    await user.click(screen.getByRole('button', { name: 'Save' }))
    await user.click(screen.getByRole('button', { name: 'Reset to default' }))

    expect(onSave).toHaveBeenCalledTimes(1)
    expect(onReset).toHaveBeenCalledTimes(1)
  })

  it('shows the saving state and blocks both actions', () => {
    render(
      <SettingsPageHeader
        title="General"
        dirty
        saving
        onSave={vi.fn()}
        onReset={vi.fn()}
      />
    )

    expect(screen.getByRole('button', { name: 'Saving…' })).toBeDisabled()
    expect(
      screen.getByRole('button', { name: 'Reset to default' })
    ).toBeDisabled()
  })

  it('shows the saved confirmation after a successful save', () => {
    render(
      <SettingsPageHeader
        title="General"
        saved
        onSave={vi.fn()}
        onReset={vi.fn()}
      />
    )

    expect(screen.getByText('Saved')).toBeInTheDocument()
  })
})
