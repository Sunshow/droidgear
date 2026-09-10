import { describe, expect, it, vi } from 'vitest'
import userEvent from '@testing-library/user-event'

const { addProviderMock, updateProviderMock } = vi.hoisted(() => ({
  addProviderMock: vi.fn(),
  updateProviderMock: vi.fn(),
}))

vi.mock('@/store/codex-store', () => ({
  useCodexStore: <T,>(selector: (state: Record<string, unknown>) => T) =>
    selector({
      currentProfile: {
        id: 'profile-a',
        name: 'Profile A',
        description: null,
        createdAt: '2026-01-01T00:00:00Z',
        updatedAt: '2026-01-01T00:00:00Z',
        providers: {
          custom: {
            name: 'Custom',
            model: 'gpt-5.6-sol',
            apiKey: null,
          },
        },
        modelProvider: 'custom',
        model: 'gpt-5.6-sol',
        modelReasoningEffort: null,
        apiKey: null,
        authProfileName: null,
      },
      addProvider: addProviderMock,
      updateProvider: updateProviderMock,
    }),
}))

vi.mock('@/lib/bindings', () => ({
  commands: {},
}))

import { render, screen } from '@/test/test-utils'
import { ProviderDialog } from './ProviderDialog'

// The comboboxes appear in this order: Wire API, Context Window,
// Auto-Compact Token Limit, Reasoning Effort. Re-query each time: the
// trigger nodes are replaced when the selected value re-renders.
async function pickOption(
  user: ReturnType<typeof userEvent.setup>,
  comboboxIndex: number,
  name: RegExp
) {
  const combobox = screen.getAllByRole('combobox')[comboboxIndex]
  if (!combobox) throw new Error(`Combobox ${comboboxIndex} not found`)
  await user.click(combobox)
  const option = await screen.findByRole('option', { name })
  await user.click(option)
}

function selectText(index: number): string {
  const combobox = screen.getAllByRole('combobox')[index]
  if (!combobox) throw new Error(`Combobox ${index} not found`)
  return combobox.textContent ?? ''
}

describe('ProviderDialog context window tier linkage', () => {
  it('links the auto-compact preset when a context window preset is picked and saves both values', async () => {
    const user = userEvent.setup()
    render(
      <ProviderDialog
        open
        onOpenChange={() => undefined}
        editingProviderId="custom"
      />
    )

    // Default: not set on both fields.
    expect(selectText(1)).toContain('Not set')
    expect(selectText(2)).toContain('Not set')

    // Picking the 272K context window links the 250K auto-compact preset.
    await pickOption(user, 1, /272K/)
    expect(selectText(1)).toContain('272K')
    expect(selectText(2)).toContain('250K')

    // Switching to the 1M tier re-links the 900K auto-compact preset.
    await pickOption(user, 1, /1M/)
    expect(selectText(1)).toContain('1M')
    expect(selectText(2)).toContain('900K')

    await user.click(screen.getByRole('button', { name: 'Save' }))

    expect(updateProviderMock).toHaveBeenCalledWith(
      'custom',
      expect.objectContaining({
        modelContextWindow: 1000000,
        modelAutoCompactTokenLimit: 900000,
      })
    )
  })

  it('allows re-picking the auto-compact tier after the linked selection', async () => {
    const user = userEvent.setup()
    render(
      <ProviderDialog
        open
        onOpenChange={() => undefined}
        editingProviderId="custom"
      />
    )

    await pickOption(user, 1, /272K/)
    expect(selectText(2)).toContain('250K')

    // The user may still hand-pick a different auto-compact tier.
    await pickOption(user, 2, /900K/)
    expect(selectText(2)).toContain('900K')

    // The context window selection stays untouched.
    expect(selectText(1)).toContain('272K')
  })
})
