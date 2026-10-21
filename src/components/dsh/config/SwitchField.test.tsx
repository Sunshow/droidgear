import { describe, expect, it, vi } from 'vitest'
import userEvent from '@testing-library/user-event'
import { render, screen } from '@/test/test-utils'
import { SwitchField } from './SwitchField'

describe('SwitchField', () => {
  it('reflects the boolean value', () => {
    const { rerender } = render(
      <SwitchField value={false} onChange={vi.fn()} />
    )
    expect(screen.getByRole('switch')).not.toBeChecked()

    rerender(<SwitchField value onChange={vi.fn()} />)
    expect(screen.getByRole('switch')).toBeChecked()
  })

  it('calls onChange with the next state when toggled', async () => {
    const user = userEvent.setup()
    const onChange = vi.fn()
    render(<SwitchField value={false} onChange={onChange} />)

    await user.click(screen.getByRole('switch'))

    expect(onChange).toHaveBeenCalledWith(true)
  })

  it('marks the control dirty with the unsaved indicator', () => {
    render(<SwitchField value onChange={vi.fn()} dirty />)

    expect(screen.getByRole('img', { name: 'Unsaved' })).toBeInTheDocument()
  })

  it('disables the control', () => {
    render(<SwitchField value onChange={vi.fn()} disabled />)

    expect(screen.getByRole('switch')).toBeDisabled()
  })
})
