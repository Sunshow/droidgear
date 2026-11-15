import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  commands,
  type VideoModel,
  type VideoTaskSnapshot,
} from '@/lib/bindings'
import { usePlaygroundStore } from './playground-store'

vi.mock('@/lib/bindings', () => ({
  commands: {
    loadVideoModels: vi.fn(),
    saveVideoModels: vi.fn(),
    loadVideoTasks: vi.fn(),
    saveVideoTasks: vi.fn(),
    createVideoTask: vi.fn(),
    queryVideoTask: vi.fn(),
  },
}))

vi.mock('sonner', () => ({
  toast: { error: vi.fn(), success: vi.fn() },
}))

const initialState = usePlaygroundStore.getState()

const model: VideoModel = {
  id: 'vm_1',
  name: 'Seedance',
  modelId: 'doubao-seedance-2-0-mini-260615',
  source: { kind: 'channel', channelId: 'ch_1', tokenId: 3 },
  protocol: 'doubao',
  durations: [5, 10],
  resolutions: ['720p', '1080p'],
  supportsImageInput: true,
}

const params = {
  prompt: 'a cat',
  duration: '5',
  resolution: '720p',
  imageUrl: '',
  extraJson: '',
}

function snapshot(
  overrides: Partial<VideoTaskSnapshot> = {}
): VideoTaskSnapshot {
  return {
    id: 'task_1',
    status: 'queued',
    progress: 0,
    videoUrl: null,
    localPath: null,
    error: null,
    completedAt: null,
    ...overrides,
  }
}

const loadVideoModels = vi.mocked(commands.loadVideoModels)
const saveVideoModels = vi.mocked(commands.saveVideoModels)
const loadVideoTasks = vi.mocked(commands.loadVideoTasks)
const saveVideoTasks = vi.mocked(commands.saveVideoTasks)
const createVideoTask = vi.mocked(commands.createVideoTask)
const queryVideoTask = vi.mocked(commands.queryVideoTask)

async function seedModel() {
  loadVideoModels.mockResolvedValue({ status: 'ok', data: [model] })
  await usePlaygroundStore.getState().loadVideoModels()
}

describe('playground store', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    usePlaygroundStore.setState(initialState, true)
    saveVideoModels.mockResolvedValue({ status: 'ok', data: null })
    loadVideoModels.mockResolvedValue({ status: 'ok', data: [] })
    saveVideoTasks.mockResolvedValue({ status: 'ok', data: null })
    loadVideoTasks.mockResolvedValue({ status: 'ok', data: [] })
  })

  it('loads persisted models and selects the first one', async () => {
    await seedModel()

    const state = usePlaygroundStore.getState()
    expect(state.videoModels).toEqual([model])
    expect(state.videoModelsLoaded).toBe(true)
    expect(state.videoDraft.modelId).toBe('vm_1')
  })

  it('loads persisted tasks and selects the first one', async () => {
    const persisted = {
      ...snapshot({ id: 'task_persisted', status: 'completed' }),
      modelId: 'vm_1',
      modelName: 'Seedance',
      model,
      params,
      createdAt: 1_700_000_000_000,
      videoUrl: 'https://cdn.example/video.mp4',
    }
    loadVideoTasks.mockResolvedValue({ status: 'ok', data: [persisted] })

    await usePlaygroundStore.getState().loadVideoTasks()

    const state = usePlaygroundStore.getState()
    expect(state.videoTasks).toEqual([persisted])
    expect(state.videoTasksLoaded).toBe(true)
    expect(state.selectedVideoTaskId).toBe('task_persisted')
  })

  it('adds a model, persists it, and selects it in the create form', async () => {
    const created = await usePlaygroundStore.getState().addVideoModel({
      name: 'Test',
      modelId: 'test-video',
      source: { kind: 'custom', baseUrl: 'https://x', apiKey: 'sk-x' },
      protocol: 'new-api-videos',
      durations: [5],
      resolutions: [],
      supportsImageInput: false,
    })

    expect(created?.id).toBeTruthy()
    expect(saveVideoModels).toHaveBeenCalledWith([
      expect.objectContaining({
        id: created?.id,
        modelId: 'test-video',
        protocol: 'new-api-videos',
      }),
    ])
    expect(usePlaygroundStore.getState().videoDraft.modelId).toBe(created?.id)
  })

  it('rolls back an added model when persistence fails', async () => {
    saveVideoModels.mockResolvedValue({ status: 'error', error: 'disk full' })

    const created = await usePlaygroundStore.getState().addVideoModel({
      name: 'Test',
      modelId: 'test-video',
      source: { kind: 'custom', baseUrl: 'https://x', apiKey: '' },
      protocol: 'doubao',
      durations: [],
      resolutions: [],
      supportsImageInput: false,
    })

    expect(created).toBeNull()
    expect(usePlaygroundStore.getState().videoModels).toEqual([])
  })

  it('creates a task through the backend and selects it', async () => {
    await seedModel()
    createVideoTask.mockResolvedValue({
      status: 'ok',
      data: snapshot({ id: 'task_1' }),
    })

    const taskId = await usePlaygroundStore
      .getState()
      .createVideoTask('vm_1', params)

    expect(taskId).toBe('task_1')
    expect(createVideoTask).toHaveBeenCalledWith(
      model,
      'a cat',
      5,
      '720p',
      null,
      ''
    )
    const task = usePlaygroundStore
      .getState()
      .videoTasks.find(item => item.id === 'task_1')
    expect(task).toMatchObject({
      status: 'queued',
      modelName: 'Seedance',
    })
    expect(usePlaygroundStore.getState().selectedVideoTaskId).toBe('task_1')
    expect(saveVideoTasks).toHaveBeenCalledWith([
      expect.objectContaining({ id: 'task_1', status: 'queued' }),
    ])
  })

  it('does not insert a task when the backend rejects the request', async () => {
    await seedModel()
    createVideoTask.mockResolvedValue({
      status: 'error',
      error: 'HTTP 400: bad prompt',
    })

    const taskId = await usePlaygroundStore
      .getState()
      .createVideoTask('vm_1', params)

    expect(taskId).toBeNull()
    expect(usePlaygroundStore.getState().videoTasks).toEqual([])
  })

  it('applies query results to the task', async () => {
    await seedModel()
    createVideoTask.mockResolvedValue({
      status: 'ok',
      data: snapshot({ id: 'task_1' }),
    })
    await usePlaygroundStore.getState().createVideoTask('vm_1', params)

    queryVideoTask.mockResolvedValue({
      status: 'ok',
      data: snapshot({
        id: 'task_1',
        status: 'completed',
        progress: 100,
        localPath: '/tmp/video.mp4',
        completedAt: 1_700_000_000_000,
      }),
    })

    await usePlaygroundStore.getState().refreshVideoTasks('task_1')

    const task = usePlaygroundStore
      .getState()
      .videoTasks.find(item => item.id === 'task_1')
    expect(task).toMatchObject({
      status: 'completed',
      progress: 100,
      localPath: '/tmp/video.mp4',
      completedAt: 1_700_000_000_000,
    })
    expect(usePlaygroundStore.getState().lastRefreshedAt).not.toBeNull()
    expect(saveVideoTasks).toHaveBeenCalledWith([
      expect.objectContaining({ id: 'task_1', status: 'completed' }),
    ])
  })

  it('leaves a task untouched when a query fails', async () => {
    await seedModel()
    createVideoTask.mockResolvedValue({
      status: 'ok',
      data: snapshot({ id: 'task_1' }),
    })
    await usePlaygroundStore.getState().createVideoTask('vm_1', params)
    queryVideoTask.mockResolvedValue({
      status: 'error',
      error: 'network down',
    })

    await usePlaygroundStore.getState().refreshVideoTasks('task_1')

    const task = usePlaygroundStore
      .getState()
      .videoTasks.find(item => item.id === 'task_1')
    expect(task?.status).toBe('queued')
  })

  it('stops polling a cancelled task', async () => {
    await seedModel()
    createVideoTask.mockResolvedValue({
      status: 'ok',
      data: snapshot({ id: 'task_1' }),
    })
    await usePlaygroundStore.getState().createVideoTask('vm_1', params)
    await usePlaygroundStore.getState().cancelVideoTask('task_1')

    await usePlaygroundStore.getState().refreshVideoTasks()

    expect(queryVideoTask).not.toHaveBeenCalled()
    const task = usePlaygroundStore
      .getState()
      .videoTasks.find(item => item.id === 'task_1')
    expect(task?.status).toBe('cancelled')
  })

  it('reruns a task after its model was deleted', async () => {
    await seedModel()
    createVideoTask.mockResolvedValue({
      status: 'ok',
      data: snapshot({ id: 'task_1' }),
    })
    await usePlaygroundStore.getState().createVideoTask('vm_1', params)
    await usePlaygroundStore.getState().deleteVideoModel('vm_1')

    createVideoTask.mockResolvedValue({
      status: 'ok',
      data: snapshot({ id: 'task_2' }),
    })
    const taskId = await usePlaygroundStore.getState().rerunVideoTask('task_1')

    expect(taskId).toBe('task_2')
    expect(createVideoTask).toHaveBeenLastCalledWith(
      model,
      'a cat',
      5,
      '720p',
      null,
      ''
    )
  })

  it('clears the selection when the selected task is deleted', async () => {
    await seedModel()
    createVideoTask.mockResolvedValue({
      status: 'ok',
      data: snapshot({ id: 'task_1' }),
    })
    await usePlaygroundStore.getState().createVideoTask('vm_1', params)

    await usePlaygroundStore.getState().deleteVideoTask('task_1')

    const state = usePlaygroundStore.getState()
    expect(state.videoTasks).toEqual([])
    expect(state.selectedVideoTaskId).toBeNull()
  })
})
