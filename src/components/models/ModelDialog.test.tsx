import { describe, expect, it, vi } from 'vitest'
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
    const providerComboboxes = screen.getAllByRole('combobox')
    const providerSelect = providerComboboxes[0]
    if (!providerSelect) throw new Error('Provider combobox not found')
    await user.click(providerSelect)
    const openaiOptions = screen.getAllByText(/OpenAI/i)
    const lastOption = openaiOptions[openaiOptions.length - 1]
    if (lastOption) {
      await user.click(lastOption)
    }

    expect(screen.getByLabelText(/api url/i)).toHaveValue(
      'https://api.example.com/custom'
    )
  })

  it('preserves an existing anthropic xhigh effort when editing and saving', async () => {
    const user = userEvent.setup()
    const onSave = vi.fn()

    render(
      <ModelDialog
        open
        onOpenChange={() => undefined}
        mode="edit"
        model={{
          provider: 'anthropic',
          model: 'claude-sonnet-4.6',
          baseUrl: 'https://api.anthropic.com',
          apiKey: 'test-key',
          displayName: 'Claude Sonnet 4.6',
          extraArgs: {
            thinking: { type: 'adaptive' },
            output_config: { effort: 'xhigh' },
          },
        }}
        onSave={onSave}
      />
    )

    await user.click(screen.getByRole('button', { name: 'Save' }))

    expect(onSave).toHaveBeenCalledTimes(1)
    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({
        extraArgs: expect.objectContaining({
          thinking: { type: 'adaptive' },
          output_config: { effort: 'xhigh' },
        }),
      })
    )
  })

  it('keeps anthropic xhigh selected when changing to another fallback claude model', async () => {
    const user = userEvent.setup()
    const onSave = vi.fn()

    render(
      <ModelDialog
        open
        onOpenChange={() => undefined}
        mode="edit"
        model={{
          provider: 'anthropic',
          model: 'claude-opus-4.7',
          baseUrl: 'https://api.anthropic.com',
          apiKey: 'test-key',
          displayName: 'Claude Opus 4.7',
          extraArgs: {
            thinking: { type: 'adaptive' },
            output_config: { effort: 'xhigh' },
          },
        }}
        onSave={onSave}
      />
    )

    const modelInput = screen.getByRole('textbox', { name: /^model$/i })
    await user.clear(modelInput)
    await user.type(modelInput, 'claude-sonnet-4.6')
    await user.click(screen.getByRole('button', { name: 'Save' }))

    expect(onSave).toHaveBeenCalledTimes(1)
    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({
        model: 'claude-sonnet-4.6',
        extraArgs: expect.objectContaining({
          thinking: { type: 'adaptive' },
          output_config: { effort: 'xhigh' },
        }),
      })
    )
  })

  it('emits Opus 4.7 style extraArgs and strips sampling params for claude-sonnet-5', async () => {
    const user = userEvent.setup()
    const onSave = vi.fn()

    render(
      <ModelDialog
        open
        onOpenChange={() => undefined}
        mode="edit"
        model={{
          provider: 'anthropic',
          model: 'claude-sonnet-5',
          baseUrl: 'https://api.anthropic.com',
          apiKey: 'test-key',
          displayName: 'claude-sonnet-5',
          extraArgs: {
            thinking: { type: 'adaptive' },
            output_config: { effort: 'xhigh' },
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
          },
        }}
        onSave={onSave}
      />
    )

    await user.click(screen.getByRole('button', { name: 'Save' }))

    expect(onSave).toHaveBeenCalledTimes(1)
    const saved = onSave.mock.calls[0]?.[0]
    expect(saved.model).toBe('claude-sonnet-5')
    expect(saved.extraArgs).toEqual({
      thinking: { type: 'adaptive' },
      output_config: { effort: 'xhigh' },
    })
    expect(saved.extraArgs).not.toHaveProperty('temperature')
    expect(saved.extraArgs).not.toHaveProperty('top_p')
    expect(saved.extraArgs).not.toHaveProperty('top_k')
  })

  it('trims whitespace from baseUrl, apiKey and model id on save', async () => {
    const user = userEvent.setup()
    const onSave = vi.fn()

    render(
      <ModelDialog
        open
        onOpenChange={() => undefined}
        mode="add"
        onSave={onSave}
      />
    )

    const baseUrlInput = screen.getByLabelText(/api url/i)
    await user.clear(baseUrlInput)
    await user.type(baseUrlInput, '  https://api.example.com/v1  ')

    const apiKeyInput = screen.getByLabelText(/api key/i)
    await user.type(apiKeyInput, '  sk-abc123  ')

    const modelInput = screen.getByRole('textbox', { name: /^model$/i })
    await user.type(modelInput, '  claude-sonnet-4-5-20250929  ')

    await user.click(screen.getByRole('button', { name: 'Add' }))

    expect(onSave).toHaveBeenCalledTimes(1)
    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({
        baseUrl: 'https://api.example.com/v1',
        apiKey: 'sk-abc123',
        model: 'claude-sonnet-4-5-20250929',
      })
    )
  })
})
