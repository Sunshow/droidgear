import { describe, expect, it, vi } from 'vitest'
import { fireEvent } from '@testing-library/react'
import { render, screen } from '@/test/test-utils'
import { TextField } from './TextField'

describe('TextField', () => {
  it('renders the value and placeholder', () => {
    render(
      <TextField
        id="base-url"
        value="https://api.example.com/v1"
        placeholder="https://api.example.com/v1"
        onChange={vi.fn()}
      />
    )

    const input = screen.getByRole('textbox')
    expect(input).toHaveValue('https://api.example.com/v1')
    expect(input).toHaveAttribute('placeholder', 'https://api.example.com/v1')
  })

  it('calls onChange with the typed text', () => {
    const onChange = vi.fn()
    render(<TextField value="" onChange={onChange} />)

    fireEvent.change(screen.getByRole('textbox'), {
      target: { value: 'zh-CN' },
    })

    expect(onChange).toHaveBeenCalledWith('zh-CN')
  })

  it('marks the control dirty with the unsaved indicator', () => {
    render(<TextField value="zh-CN" onChange={vi.fn()} dirty />)

    expect(screen.getByRole('img', { name: 'Unsaved' })).toBeInTheDocument()
  })

  it('disables the control', () => {
    render(<TextField value="" onChange={vi.fn()} disabled />)

    expect(screen.getByRole('textbox')).toBeDisabled()
  })
})
