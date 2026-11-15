import { useState } from 'react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it } from 'vitest'
import { fireEvent, render, screen, waitFor } from '@/test/test-utils'
import { VideoReferenceImages } from './VideoReferenceImages'

function Harness() {
  const [value, setValue] = useState('')
  return (
    <div>
      <VideoReferenceImages value={value} onChange={setValue} />
      <output data-testid="value">{value}</output>
    </div>
  )
}

describe('VideoReferenceImages', () => {
  it('adds multiple URL images and removes one', async () => {
    const user = userEvent.setup({ delay: null })
    render(<Harness />)

    await user.click(screen.getByRole('button', { name: 'Add image' }))
    fireEvent.change(screen.getByRole('textbox', { name: 'URLs' }), {
      target: {
        value: 'https://cdn.example/a.png\nhttps://cdn.example/b.png',
      },
    })
    await user.click(screen.getByRole('button', { name: 'Add' }))

    expect(screen.getByTestId('value').textContent).toBe(
      'https://cdn.example/a.png\nhttps://cdn.example/b.png'
    )
    expect(
      screen.getAllByRole('img', { name: /Reference image/ })
    ).toHaveLength(2)
    expect(
      screen.getByRole('img', { name: 'Reference image 2' })
    ).toHaveAttribute('src', 'https://cdn.example/b.png')

    const remove = screen.getAllByRole('button', { name: 'Remove image' }).at(0)
    if (!remove) throw new Error('remove button is missing')
    await user.click(remove)

    expect(screen.getByTestId('value').textContent).toBe(
      'https://cdn.example/b.png'
    )
    expect(
      screen.getAllByRole('img', { name: /Reference image/ })
    ).toHaveLength(1)
    expect(
      screen.getByRole('img', { name: 'Reference image 1' })
    ).toHaveAttribute('src', 'https://cdn.example/b.png')
  })

  it('reads multiple local files as base64 data URLs', async () => {
    const user = userEvent.setup({ delay: null })
    render(<Harness />)

    await user.click(screen.getByRole('button', { name: 'Add image' }))
    await user.click(screen.getByRole('tab', { name: 'Local files' }))

    const input = document.querySelector('input[type="file"]')
    expect(input).not.toBeNull()
    fireEvent.change(input as HTMLInputElement, {
      target: {
        files: [
          new File(['hello'], 'a.png', { type: 'image/png' }),
          new File(['world'], 'b.png', { type: 'image/png' }),
        ],
      },
    })

    await waitFor(() => {
      expect(
        document.querySelectorAll('img[src^="data:image/png;base64,"]')
      ).toHaveLength(2)
    })
    await user.click(screen.getByRole('button', { name: 'Add' }))

    const value = screen.getByTestId('value').textContent ?? ''
    expect(value.split('\n')).toHaveLength(2)
    expect(value).toMatch(/^data:image\/png;base64,/)
    expect(
      screen.getAllByRole('img', { name: /Reference image/ })
    ).toHaveLength(2)
  })

  it('rejects an invalid URL line without adding anything', async () => {
    const user = userEvent.setup({ delay: null })
    render(<Harness />)

    await user.click(screen.getByRole('button', { name: 'Add image' }))
    fireEvent.change(screen.getByRole('textbox', { name: 'URLs' }), {
      target: {
        value: 'https://cdn.example/a.png\nftp://example.com/b.png',
      },
    })
    await user.click(screen.getByRole('button', { name: 'Add' }))

    expect(
      screen.getByText('Line 2 is not a valid http(s) image URL.')
    ).toBeInTheDocument()
    expect(screen.getByTestId('value').textContent).toBe('')
  })
})
