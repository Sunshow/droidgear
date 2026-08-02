import { describe, expect, it, vi } from 'vitest'
import userEvent from '@testing-library/user-event'

const { commandMocks } = vi.hoisted(() => ({
  commandMocks: {
    detectChannelType: vi.fn().mockResolvedValue({
      status: 'ok',
      data: 'general',
    }),
  },
}))

vi.mock('@/lib/bindings', () => ({
  commands: commandMocks,
}))

import { render, screen } from '@/test/test-utils'
import { ChannelDialog } from './ChannelDialog'

describe('ChannelDialog', () => {
  it('trims whitespace from name, baseUrl and apiKey on save', async () => {
    const user = userEvent.setup()
    const onSave = vi.fn()

    render(
      <ChannelDialog open onOpenChange={() => undefined} onSave={onSave} />
    )

    await user.type(screen.getByLabelText(/^name$/i), '  My Channel  ')
    await user.type(
      screen.getByLabelText(/api url/i),
      '  https://api.example.com  '
    )
    await user.type(screen.getByLabelText(/api key/i), '  sk-abc123  ')

    await user.click(screen.getByRole('button', { name: 'Add' }))

    expect(onSave).toHaveBeenCalledTimes(1)
    const [channel, username, apiKey] = onSave.mock.calls[0] ?? []
    expect(channel).toMatchObject({
      name: 'My Channel',
      baseUrl: 'https://api.example.com',
      type: 'general',
    })
    expect(username).toBe('')
    expect(apiKey).toBe('sk-abc123')
  })
})
