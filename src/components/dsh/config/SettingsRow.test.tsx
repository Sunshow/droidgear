import { describe, expect, it } from 'vitest'
import { render, screen } from '@/test/test-utils'
import { SettingsRow } from './SettingsRow'

describe('SettingsRow', () => {
  it('renders the label, description, badges, and control', () => {
    render(
      <SettingsRow
        label="Locale"
        description="Language of the DSH interface"
        htmlFor="locale-select"
        badges={<span>Beta</span>}
      >
        <select id="locale-select" aria-label="locale">
          <option>English</option>
        </select>
      </SettingsRow>
    )

    expect(screen.getByText('Locale')).toBeInTheDocument()
    expect(
      screen.getByText('Language of the DSH interface')
    ).toBeInTheDocument()
    expect(screen.getByText('Beta')).toBeInTheDocument()
    expect(screen.getByLabelText('locale')).toBeInTheDocument()
  })

  it('defaults to the horizontal layout', () => {
    render(
      <SettingsRow label="Theme">
        <span>control</span>
      </SettingsRow>
    )

    expect(screen.getByText('Theme').closest('[data-layout]')).toHaveAttribute(
      'data-layout',
      'horizontal'
    )
    expect(screen.getByText('control')).toBeInTheDocument()
  })

  it('supports the vertical layout', () => {
    render(
      <SettingsRow label="Theme" layout="vertical">
        <span>control</span>
      </SettingsRow>
    )

    expect(screen.getByText('Theme').closest('[data-layout]')).toHaveAttribute(
      'data-layout',
      'vertical'
    )
  })

  it('prefers the explicit control prop over children', () => {
    render(
      <SettingsRow label="Font size" control={<span>control slot</span>}>
        <span>child slot</span>
      </SettingsRow>
    )

    expect(screen.getByText('control slot')).toBeInTheDocument()
    expect(screen.queryByText('child slot')).not.toBeInTheDocument()
  })
})
