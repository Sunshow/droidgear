import { describe, expect, it } from 'vitest'
import { render, screen } from '@/test/test-utils'
import userEvent from '@testing-library/user-event'
import { ModelDialog } from './ModelDialog'

describe('ModelDialog', () => {
  it('does not clear api url when switching provider in add mode', async () => {
    const user = userEvent.setup()
    render(
      <ModelDialog
        open
        onOpenChange={() => undefined}
        mode="add"
        onSave={() => undefined}
      />
    )

    const baseUrlInput = screen.getByLabelText(/api url/i)
    await user.clear(baseUrlInput)
    await user.type(baseUrlInput, 'https://api.example.com/custom')

    // Open provider select and switch provider
    await user.click(screen.getByRole('combobox'))
    await user.click(screen.getAllByText(/OpenAI/i).at(-1)!)

    expect(screen.getByLabelText(/api url/i)).toHaveValue(
      'https://api.example.com/custom'
    )
  })
})
