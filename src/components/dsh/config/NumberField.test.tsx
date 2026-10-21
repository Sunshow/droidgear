import { describe, expect, it, vi } from 'vitest'
import { fireEvent } from '@testing-library/react'
import { render, screen } from '@/test/test-utils'
import { NumberField } from './NumberField'

describe('NumberField', () => {
  it('renders the value, range, step, and unit', () => {
    render(
      <NumberField
        id="timeout"
        value={120}
        min={0}
        max={600}
        step={10}
        unit="ms"
        onChange={vi.fn()}
      />
    )

    const input = screen.getByRole('spinbutton')
    expect(input).toHaveValue(120)
    expect(input).toHaveAttribute('min', '0')
    expect(input).toHaveAttribute('max', '600')
    expect(input).toHaveAttribute('step', '10')
    expect(screen.getByText('Range: 0 – 600')).toBeInTheDocument()
    expect(screen.getByText('ms')).toBeInTheDocument()
  })

  it('renders a one-sided range hint', () => {
    render(<NumberField value={null} min={1} onChange={vi.fn()} />)

    expect(screen.getByText('Minimum: 1')).toBeInTheDocument()
  })

  it('reports numbers and clears to null', () => {
    const onChange = vi.fn()
    render(<NumberField value={120} onChange={onChange} />)

    fireEvent.change(screen.getByRole('spinbutton'), {
      target: { value: '240' },
    })
    expect(onChange).toHaveBeenLastCalledWith(240)

    fireEvent.change(screen.getByRole('spinbutton'), {
      target: { value: '' },
    })
    expect(onChange).toHaveBeenLastCalledWith(null)
  })

  it('marks the control dirty with the unsaved indicator', () => {
    render(<NumberField value={1} onChange={vi.fn()} dirty />)

    expect(screen.getByRole('img', { name: 'Unsaved' })).toBeInTheDocument()
  })

  it('disables the control', () => {
    render(<NumberField value={1} onChange={vi.fn()} disabled />)

    expect(screen.getByRole('spinbutton')).toBeDisabled()
  })
})
