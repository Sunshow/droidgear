import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { render, screen } from '@/test/test-utils'
import type { VideoModel } from '@/lib/bindings'
import { usePlaygroundStore } from '@/store/playground-store'
import { VideoCreatePanel } from './VideoCreatePanel'

vi.mock('sonner', () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}))

const initialState = usePlaygroundStore.getState()

const model: VideoModel = {
  id: 'vm_1',
  name: 'Seedance',
  modelId: 'doubao-seedance-2-0-mini-260615',
  source: { kind: 'channel', channelId: 'ch_1', tokenId: 1 },
  protocol: 'doubao',
  durations: [],
  resolutions: [],
  supportsImageInput: true,
}

function seed(videoModels: VideoModel[], modelId: string) {
  usePlaygroundStore.setState(
    {
      ...initialState,
      videoModels,
      videoModelsLoaded: true,
      videoDraft: { ...initialState.videoDraft, modelId },
    },
    true
  )
}

describe('VideoCreatePanel', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('falls back to the shared presets when a model saved no lists', async () => {
    seed([model], model.id)
    const user = userEvent.setup({ delay: null })

    render(<VideoCreatePanel />)

    const duration = screen.getByRole('button', { name: 'Duration' })
    const resolution = screen.getByRole('button', { name: 'Resolution' })
    expect(duration).toBeEnabled()
    expect(resolution).toBeEnabled()

    await user.click(duration)
    expect(await screen.findByRole('option', { name: '5' })).toBeInTheDocument()

    await user.click(resolution)
    expect(
      await screen.findByRole('option', { name: '1280x720' })
    ).toBeInTheDocument()
  })

  it('keeps the shared presets available before a model is picked', async () => {
    seed([], '')
    const user = userEvent.setup({ delay: null })

    render(<VideoCreatePanel />)

    const duration = screen.getByRole('button', { name: 'Duration' })
    expect(duration).toBeEnabled()
    expect(screen.getByRole('button', { name: 'Resolution' })).toBeEnabled()

    await user.click(duration)
    expect(await screen.findByRole('option', { name: '5' })).toBeInTheDocument()
  })
})
