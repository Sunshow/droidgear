import userEvent from '@testing-library/user-event'
import { toast } from 'sonner'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { act, render, screen, waitFor } from '@/test/test-utils'
import { commands, type VideoModel } from '@/lib/bindings'
import { usePlaygroundStore, type VideoTask } from '@/store/playground-store'
import { VideoPlaygroundPage } from './VideoPlaygroundPage'

vi.mock('sonner', () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}))

vi.mock('@/lib/bindings', () => ({
  commands: {
    loadVideoModels: vi.fn().mockResolvedValue({ status: 'ok', data: [] }),
    saveVideoModels: vi.fn().mockResolvedValue({ status: 'ok', data: null }),
    loadVideoTasks: vi.fn().mockResolvedValue({ status: 'ok', data: [] }),
    saveVideoTasks: vi.fn().mockResolvedValue({ status: 'ok', data: null }),
    createVideoTask: vi.fn(),
    queryVideoTask: vi.fn(),
    savePlaygroundMediaAs: vi.fn(),
  },
}))

const initialState = usePlaygroundStore.getState()

const model: VideoModel = {
  id: 'vm_1',
  name: 'Seedance',
  modelId: 'doubao-seedance-2-0-mini-260615',
  source: { kind: 'channel', channelId: 'ch_1', tokenId: 1 },
  protocol: 'doubao',
  durations: [5],
  resolutions: ['720p'],
  supportsImageInput: true,
}

const params = {
  prompt: 'a cat',
  duration: '5',
  resolution: '720p',
  imageUrl: '',
  extraJson: '',
}

function task(overrides: Partial<VideoTask> = {}): VideoTask {
  return {
    id: 'task_1',
    modelId: model.id,
    modelName: model.name,
    model,
    params,
    status: 'completed',
    progress: 100,
    createdAt: Date.now() - 60_000,
    completedAt: Date.now() - 30_000,
    videoUrl: null,
    localPath: '/tmp/droidgear/video.mp4',
    error: null,
    ...overrides,
  }
}

function seed(videoTasks: VideoTask[]) {
  usePlaygroundStore.setState(
    {
      ...initialState,
      videoModels: [model],
      videoModelsLoaded: true,
      videoTasksLoaded: true,
      videoTasks,
      selectedVideoTaskId: videoTasks[0]?.id ?? null,
      videoDraft: { ...initialState.videoDraft, modelId: model.id },
    },
    true
  )
}

describe('VideoPlaygroundPage', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    seed([task()])
  })

  it('shows the create form, task list, and the selected local video', () => {
    const { container } = render(<VideoPlaygroundPage />)

    const modelPicker = screen.getByRole('combobox', { name: 'Model' })
    expect(modelPicker).toHaveTextContent('Seedance')
    expect(modelPicker).not.toHaveTextContent('doubao-seedance-2-0-mini-260615')
    expect(screen.getByRole('button', { name: 'Manage' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /Create task/ })).toBeDisabled()
    expect(
      screen.getByRole('button', { name: 'Query status' })
    ).toBeInTheDocument()

    const video = container.querySelector('video')
    expect(video?.getAttribute('src')).toContain('asset://')
    expect(video?.hasAttribute('controls')).toBe(true)
  })

  it('saves the video through the save dialog and reports the destination', async () => {
    vi.mocked(commands.savePlaygroundMediaAs).mockResolvedValueOnce({
      status: 'ok',
      data: '/Users/me/Downloads/task_1.mp4',
    })
    const user = userEvent.setup({ delay: null })

    render(<VideoPlaygroundPage />)
    await user.click(screen.getByRole('button', { name: 'Save as' }))

    expect(commands.savePlaygroundMediaAs).toHaveBeenCalledWith(
      '/tmp/droidgear/video.mp4',
      null,
      'task_1.mp4'
    )
    await waitFor(() =>
      expect(toast.success).toHaveBeenCalledWith(
        'Saved to /Users/me/Downloads/task_1.mp4'
      )
    )
  })

  it('reports a save failure instead of dropping it silently', async () => {
    vi.mocked(commands.savePlaygroundMediaAs).mockResolvedValueOnce({
      status: 'error',
      error: 'The provider returned HTTP 403 for the media file.',
    })
    const user = userEvent.setup({ delay: null })

    render(<VideoPlaygroundPage />)
    await user.click(screen.getByRole('button', { name: 'Save as' }))

    await waitFor(() =>
      expect(toast.error).toHaveBeenCalledWith(
        'Could not save the file to the chosen location.'
      )
    )
  })

  it('polls running tasks on the auto refresh interval', async () => {
    vi.useFakeTimers()
    try {
      seed([task({ status: 'in_progress', progress: 40, localPath: null })])
      usePlaygroundStore.setState({ refreshIntervalSec: 3 })
      vi.mocked(commands.queryVideoTask).mockResolvedValue({
        status: 'ok',
        data: {
          id: 'task_1',
          status: 'completed',
          progress: 100,
          videoUrl: 'https://cdn.example/video.mp4',
          localPath: null,
          error: null,
          completedAt: 1_700_000_000_000,
        },
      })

      render(<VideoPlaygroundPage />)

      await act(async () => {
        await vi.advanceTimersByTimeAsync(3_500)
      })

      const updated = usePlaygroundStore
        .getState()
        .videoTasks.find(item => item.id === 'task_1')
      expect(updated?.status).toBe('completed')
      expect(updated?.videoUrl).toBe('https://cdn.example/video.mp4')
    } finally {
      vi.useRealTimers()
    }
  })

  it('allows free-form options and multiple Sora reference images', () => {
    const sora: VideoModel = {
      ...model,
      protocol: 'sora',
      durations: [4, 8, 12],
      resolutions: ['1280x720'],
    }
    usePlaygroundStore.setState(
      {
        ...initialState,
        videoModels: [sora],
        videoModelsLoaded: true,
        videoTasksLoaded: true,
        videoTasks: [],
        selectedVideoTaskId: null,
        videoDraft: {
          ...initialState.videoDraft,
          modelId: sora.id,
          prompt: 'a cat',
          duration: '7',
          resolution: '1280x720',
          imageUrl: 'https://example.com/a.png\nhttps://example.com/b.png',
        },
      },
      true
    )

    render(<VideoPlaygroundPage />)

    expect(screen.getByRole('button', { name: /Create task/ })).toBeEnabled()
    expect(
      screen.getByRole('img', { name: 'Reference image 1' })
    ).toHaveAttribute('src', 'https://example.com/a.png')
    expect(
      screen.getByRole('img', { name: 'Reference image 2' })
    ).toHaveAttribute('src', 'https://example.com/b.png')
  })

  it('clears the create form but keeps the selected model', async () => {
    usePlaygroundStore.setState(state => ({
      videoDraft: {
        ...state.videoDraft,
        prompt: 'a cat',
        duration: '5',
        resolution: '720p',
        imageUrl: 'https://example.com/cat.png',
        extraJson: '{"seed":1}',
      },
    }))
    const user = userEvent.setup({ delay: null })

    render(<VideoPlaygroundPage />)

    await user.click(screen.getByRole('button', { name: 'Clear form' }))

    expect(usePlaygroundStore.getState().videoDraft).toMatchObject({
      modelId: model.id,
      prompt: '',
      duration: '',
      resolution: '',
      imageUrl: '',
      extraJson: '',
    })
  })

  it('adds a model from the empty header dropdown', async () => {
    usePlaygroundStore.setState(
      {
        ...initialState,
        videoModels: [],
        videoModelsLoaded: true,
        videoTasksLoaded: true,
        videoTasks: [],
        selectedVideoTaskId: null,
      },
      true
    )
    const user = userEvent.setup({ delay: null })

    render(<VideoPlaygroundPage />)

    expect(screen.getByRole('combobox', { name: 'Model' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Manage' })).toBeInTheDocument()
    expect(
      screen.getByText('No tasks yet. Create one on the left.')
    ).toBeInTheDocument()

    await user.click(screen.getByRole('combobox', { name: 'Model' }))
    await user.click(
      await screen.findByRole('option', { name: /Add video model/ })
    )

    expect(await screen.findByRole('dialog')).toBeInTheDocument()
  })
})
