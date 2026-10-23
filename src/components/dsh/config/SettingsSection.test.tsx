import { describe, expect, it } from 'vitest'
import { render, screen } from '@/test/test-utils'
import { SettingsSection } from './SettingsSection'

describe('SettingsSection', () => {
  it('renders the title, description, action, and children', () => {
    render(
      <SettingsSection
        title="Appearance"
        description="How the DSH interface looks"
        action={<button type="button">Edit</button>}
      >
        <p>Body content</p>
      </SettingsSection>
    )

    expect(
      screen.getByRole('heading', { name: 'Appearance' })
    ).toBeInTheDocument()
    expect(screen.getByText('How the DSH interface looks')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Edit' })).toBeInTheDocument()
    expect(screen.getByText('Body content')).toBeInTheDocument()
  })

  it('omits the description and action when they are not provided', () => {
    render(
      <SettingsSection title="Models">
        <p>Body content</p>
      </SettingsSection>
    )

    expect(screen.getByRole('heading', { name: 'Models' })).toBeInTheDocument()
    expect(screen.queryByRole('button')).not.toBeInTheDocument()
    expect(screen.getByText('Body content')).toBeInTheDocument()
  })
})
