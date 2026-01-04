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
    const openaiOptions = screen.getAllByText(/OpenAI/i)
    const lastOption = openaiOptions[openaiOptions.length - 1]
    if (lastOption) {
      await user.click(lastOption)
    }

    expect(screen.getByLabelText(/api url/i)).toHaveValue(
      'https://api.example.com/custom'
    )
  })
})
