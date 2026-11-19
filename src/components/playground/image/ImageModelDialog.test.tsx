import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { render, screen } from '@/test/test-utils'
import type { Channel } from '@/lib/bindings'
import { useChannelStore } from '@/store/channel-store'
import type { ImageModelDraft } from '@/store/image-playground-store'
import { ImageModelDialog } from './ImageModelDialog'

const channel: Channel = {
  id: 'channel-1',
  name: 'My Gateway',
  type: 'new-api',
  baseUrl: 'https://gateway.example',
  enabled: true,
  createdAt: 0,
}

function renderDialog(options: {
  model?: Parameters<typeof ImageModelDialog>[0]['model']
  onSave?: (draft: ImageModelDraft) => void
}) {
  const onOpenChange = vi.fn()
  render(
    <ImageModelDialog
      open
      onOpenChange={onOpenChange}
      model={options.model}
      onSave={options.onSave ?? vi.fn()}
      onSaveMany={vi.fn()}
    />
  )
  return { onOpenChange }
}

describe('ImageModelDialog', () => {
  beforeEach(() => {
    // Seed one channel so the form does not try to load the channel list.
    useChannelStore.setState({ channels: [channel], isLoading: false })
  })

  it('adds a chat model from a channel with the chat size defaults', async () => {
    const onSave = vi.fn()
    const user = userEvent.setup({ delay: null })
    const { onOpenChange } = renderDialog({ onSave })

    await user.type(screen.getByLabelText('Display name'), 'Nano Banana')
    await user.type(screen.getByLabelText('Model ID'), 'gemini-2.5-flash-image')
    expect(
      screen.getByRole('combobox', { name: 'Image protocol' })
    ).toHaveTextContent('ChatCompletions')

    await user.click(screen.getByRole('combobox', { name: 'Channel' }))
    await user.click(await screen.findByRole('option', { name: /My Gateway/ }))
    await user.click(screen.getByRole('button', { name: 'Add' }))

    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({
        name: 'Nano Banana',
        modelId: 'gemini-2.5-flash-image',
        protocol: 'chat',
        source: {
          kind: 'channel',
          channelId: 'channel-1',
          tokenId: null,
        },
        sizes: [
          '1024x1024',
          '1536x1024',
          '1024x1536',
          '16:9',
          '9:16',
          '4:3',
          '3:4',
        ],
        qualities: [],
        supportsImageInput: true,
      })
    )
    expect(onOpenChange).toHaveBeenCalledWith(false)
  })

  it('follows the protocol defaults and saves a custom endpoint', async () => {
    const onSave = vi.fn()
    const user = userEvent.setup({ delay: null })
    renderDialog({ onSave })

    await user.type(screen.getByLabelText('Display name'), 'DALL-E')
    await user.type(screen.getByLabelText('Model ID'), 'gpt-image-1')
    await user.click(screen.getByRole('combobox', { name: 'Image protocol' }))
    await user.click(
      await screen.findByRole('option', { name: 'OpenAI Images' })
    )

    expect(screen.getByLabelText('Sizes / ratios')).toHaveValue(
      'auto, 1024x1024, 1536x1024, 1024x1536'
    )
    expect(screen.getByLabelText('Quality values')).toHaveValue(
      'auto, low, medium, high'
    )

    await user.click(screen.getByRole('radio', { name: 'Custom endpoint' }))
    await user.type(screen.getByLabelText('Base URL'), 'https://api.example')
    await user.type(screen.getByLabelText('API key'), 'sk-test')
    await user.click(screen.getByRole('button', { name: 'Add' }))

    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({
        protocol: 'images',
        source: {
          kind: 'custom',
          baseUrl: 'https://api.example',
          apiKey: 'sk-test',
        },
        sizes: ['auto', '1024x1024', '1536x1024', '1024x1536'],
        qualities: ['auto', 'low', 'medium', 'high'],
      })
    )
  })

  it('prefills an existing model and keeps its option lists', async () => {
    const onSave = vi.fn()
    const user = userEvent.setup({ delay: null })
    renderDialog({
      onSave,
      model: {
        id: 'im_1',
        name: 'Imagen',
        modelId: 'imagen-4.0-generate-001',
        source: { kind: 'custom', baseUrl: 'https://gw.example', apiKey: 'sk' },
        protocol: 'google',
        sizes: ['16:9'],
        qualities: ['2K'],
        supportsImageInput: false,
      },
    })

    expect(screen.getByLabelText('Display name')).toHaveValue('Imagen')
    expect(screen.getByLabelText('Model ID')).toHaveValue(
      'imagen-4.0-generate-001'
    )
    expect(screen.getByRole('switch')).not.toBeChecked()

    await user.click(screen.getByRole('button', { name: 'Save' }))

    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({
        name: 'Imagen',
        protocol: 'google',
        sizes: ['16:9'],
        qualities: ['2K'],
        supportsImageInput: false,
      })
    )
  })
})
