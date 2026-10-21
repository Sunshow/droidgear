import { describe, expect, it, vi } from 'vitest'
import userEvent from '@testing-library/user-event'
import { render, screen } from '@/test/test-utils'
import { EnumField } from './EnumField'

const options = [
  { value: 'en', label: 'English' },
  { value: 'zh', label: 'Chinese' },
]

describe('EnumField', () => {
  it('renders the label of the selected option', () => {
    render(
      <EnumField id="locale" value="en" options={options} onChange={vi.fn()} />
    )

    expect(screen.getByRole('combobox')).toHaveTextContent('English')
  })

  it('renders the empty placeholder when the value is null', () => {
    render(
      <EnumField
        id="locale"
        value={null}
        options={options}
        onChange={vi.fn()}
      />
    )

    expect(screen.getByRole('combobox')).toHaveTextContent('Not set')
  })

  it('calls onChange when another option is picked', async () => {
    const user = userEvent.setup()
    const onChange = vi.fn()
    render(
      <EnumField id="locale" value="en" options={options} onChange={onChange} />
    )

    await user.click(screen.getByRole('combobox'))
    await user.click(await screen.findByRole('option', { name: 'Chinese' }))

    expect(onChange).toHaveBeenCalledWith('zh')
  })

  it('marks the control dirty with the unsaved indicator', () => {
    render(<EnumField value="en" options={options} onChange={vi.fn()} dirty />)

    expect(screen.getByRole('img', { name: 'Unsaved' })).toBeInTheDocument()
    expect(
      screen.getByRole('combobox').closest('[data-dirty]')
    ).toHaveAttribute('data-dirty', 'true')
  })

  it('disables the control', () => {
    render(
      <EnumField value="en" options={options} onChange={vi.fn()} disabled />
    )

    expect(screen.getByRole('combobox')).toBeDisabled()
  })
})
