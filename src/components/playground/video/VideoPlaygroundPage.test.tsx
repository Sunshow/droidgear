import { beforeEach, describe, expect, it, vi } from 'vitest'
import { act, render, screen } from '@/test/test-utils'
import { commands, type VideoModel } from '@/lib/bindings'
import { usePlaygroundStore, type VideoTask } from '@/store/playground-store'
import { VideoPlaygroundPage } from './VideoPlaygroundPage'

vi.mock('@/lib/bindings', () => ({
  commands: {
    loadVideoModels: vi.fn().mockResolvedValue({ status: 'ok', data: [] }),
    saveVideoModels: vi.fn().mockResolvedValue({ status: 'ok', data: null }),
    loadVideoTasks: vi.fn().mockResolvedValue({ status: 'ok', data: [] }),
    saveVideoTasks: vi.fn().mockResolvedValue({ status: 'ok', data: null }),
    createVideoTask: vi.fn(),
    queryVideoTask: vi.fn(),
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

    expect(screen.getAllByText('Add video model').length).toBeGreaterThan(0)
    expect(screen.getByRole('button', { name: /Create task/ })).toBeDisabled()

    const video = container.querySelector('video')
    expect(video?.getAttribute('src')).toContain('asset://')
    expect(video?.hasAttribute('controls')).toBe(true)
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

  it('shows the add-model entry when there are no models yet', () => {
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

    render(<VideoPlaygroundPage />)

    expect(screen.getAllByText('Add video model').length).toBeGreaterThan(0)
    expect(
      screen.getByText('No tasks yet. Create one on the left.')
    ).toBeInTheDocument()
  })
})
