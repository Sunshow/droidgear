import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  commands,
  type ImageModel,
  type ImageTaskRecord,
  type ImageTaskSnapshot,
} from '@/lib/bindings'
import { useImagePlaygroundStore } from './image-playground-store'

vi.mock('@/lib/bindings', () => ({
  commands: {
    loadImageModels: vi.fn(),
    saveImageModels: vi.fn(),
    loadImageTasks: vi.fn(),
    saveImageTasks: vi.fn(),
    createImageTask: vi.fn(),
  },
}))

vi.mock('sonner', () => ({
  toast: { error: vi.fn(), success: vi.fn() },
}))

const initialState = useImagePlaygroundStore.getState()

const model: ImageModel = {
  id: 'im_1',
  name: 'Nano Banana',
  modelId: 'gemini-3-pro-image',
  source: { kind: 'channel', channelId: 'ch_1', tokenId: 3 },
  protocol: 'google',
  sizes: ['16:9', '1:1'],
  qualities: ['1K', '2K'],
  supportsImageInput: true,
}

const params = {
  prompt: 'a cat',
  size: '16:9',
  quality: '2K',
  count: 2,
  imageUrl: 'https://example.com/cat.png',
  extraJson: '',
}

function snapshot(
  overrides: Partial<ImageTaskSnapshot> = {}
): ImageTaskSnapshot {
  return {
    images: [
      {
        localPath: '/tmp/image-0.png',
        url: null,
        mimeType: 'image/png',
      },
    ],
    revisedPrompt: null,
    responseText: null,
    completedAt: 1_700_000_000_000,
    ...overrides,
  }
}

const loadImageModels = vi.mocked(commands.loadImageModels)
const saveImageModels = vi.mocked(commands.saveImageModels)
const loadImageTasks = vi.mocked(commands.loadImageTasks)
const saveImageTasks = vi.mocked(commands.saveImageTasks)
const createImageTask = vi.mocked(commands.createImageTask)

async function seedModel() {
  loadImageModels.mockResolvedValue({ status: 'ok', data: [model] })
  await useImagePlaygroundStore.getState().loadImageModels()
}

describe('image playground store', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    useImagePlaygroundStore.setState(initialState, true)
    saveImageModels.mockResolvedValue({ status: 'ok', data: null })
    loadImageModels.mockResolvedValue({ status: 'ok', data: [] })
    saveImageTasks.mockResolvedValue({ status: 'ok', data: null })
    loadImageTasks.mockResolvedValue({ status: 'ok', data: [] })
    createImageTask.mockResolvedValue({ status: 'ok', data: snapshot() })
  })

  it('loads persisted models and selects the first one', async () => {
    await seedModel()

    const state = useImagePlaygroundStore.getState()
    expect(state.imageModels).toEqual([model])
    expect(state.imageModelsLoaded).toBe(true)
    expect(state.imageDraft.modelId).toBe('im_1')
  })

  it('loads persisted tasks and selects the first one', async () => {
    const persisted: ImageTaskRecord = {
      id: 'img_persisted',
      modelId: model.id,
      modelName: model.name,
      model,
      params,
      status: 'completed',
      images: snapshot().images,
      revisedPrompt: null,
      responseText: null,
      createdAt: 1_700_000_000_000,
      completedAt: 1_700_000_001_000,
      error: null,
    }
    loadImageTasks.mockResolvedValue({ status: 'ok', data: [persisted] })

    await useImagePlaygroundStore.getState().loadImageTasks()

    const state = useImagePlaygroundStore.getState()
    expect(state.imageTasks).toEqual([persisted])
    expect(state.imageTasksLoaded).toBe(true)
    expect(state.selectedImageTaskId).toBe('img_persisted')
  })

  it('marks tasks that were running when the app closed as failed', async () => {
    const interrupted: ImageTaskRecord = {
      id: 'img_running',
      modelId: model.id,
      modelName: model.name,
      model,
      params,
      status: 'running',
      images: [],
      revisedPrompt: null,
      responseText: null,
      createdAt: 1_700_000_000_000,
      completedAt: null,
      error: null,
    }
    loadImageTasks.mockResolvedValue({ status: 'ok', data: [interrupted] })

    await useImagePlaygroundStore.getState().loadImageTasks()

    const task = useImagePlaygroundStore
      .getState()
      .imageTasks.find(item => item.id === 'img_running')
    expect(task?.status).toBe('failed')
    expect(task?.error).toContain('closed')
    expect(saveImageTasks).toHaveBeenCalled()
  })

  it('adds a model, persists it, and selects it in the create form', async () => {
    const created = await useImagePlaygroundStore.getState().addImageModel({
      name: 'Test',
      modelId: 'test-image',
      source: { kind: 'custom', baseUrl: 'https://x', apiKey: 'sk-x' },
      protocol: 'responses',
      sizes: ['1024x1024'],
      qualities: ['high'],
      supportsImageInput: false,
    })

    expect(created?.id).toMatch(/^im_/)
    expect(saveImageModels).toHaveBeenCalledWith([
      expect.objectContaining({ id: created?.id, name: 'Test' }),
    ])
    expect(useImagePlaygroundStore.getState().imageDraft).toMatchObject({
      modelId: created?.id,
      size: '',
      quality: '',
      count: 1,
    })
  })

  it('rolls back a model when persistence fails', async () => {
    saveImageModels.mockResolvedValue({ status: 'error', error: 'disk full' })

    const created = await useImagePlaygroundStore.getState().addImageModel({
      name: 'Test',
      modelId: 'test-image',
      source: { kind: 'custom', baseUrl: 'https://x', apiKey: 'sk-x' },
      protocol: 'chat',
      sizes: [],
      qualities: [],
      supportsImageInput: false,
    })

    expect(created).toBeNull()
    expect(useImagePlaygroundStore.getState().imageModels).toEqual([])
  })

  it('submits a task, shows it as running, then stores the images', async () => {
    await seedModel()

    const taskId = await useImagePlaygroundStore
      .getState()
      .createImageTask(model.id, params)

    expect(taskId).toMatch(/^img_/)
    expect(createImageTask).toHaveBeenCalledWith(
      model,
      taskId,
      'a cat',
      '16:9',
      '2K',
      2,
      ['https://example.com/cat.png'],
      ''
    )
    const task = useImagePlaygroundStore
      .getState()
      .imageTasks.find(item => item.id === taskId)
    expect(task?.status).toBe('completed')
    expect(task?.images).toEqual(snapshot().images)
    expect(useImagePlaygroundStore.getState().selectedImageTaskId).toBe(taskId)
    expect(saveImageTasks).toHaveBeenCalled()
  })

  it('marks the task as failed when the request fails', async () => {
    await seedModel()
    createImageTask.mockResolvedValue({
      status: 'error',
      error: 'HTTP 400: bad size',
    })

    const taskId = await useImagePlaygroundStore
      .getState()
      .createImageTask(model.id, params)

    const task = useImagePlaygroundStore
      .getState()
      .imageTasks.find(item => item.id === taskId)
    expect(task?.status).toBe('failed')
    expect(task?.error).toBe('HTTP 400: bad size')
  })

  it('reruns a task from its model snapshot after the model is deleted', async () => {
    await seedModel()
    const first = await useImagePlaygroundStore
      .getState()
      .createImageTask(model.id, params)

    useImagePlaygroundStore.setState({ imageModels: [] })
    const second = await useImagePlaygroundStore
      .getState()
      .rerunImageTask(first as string)

    expect(second).not.toBe(first)
    expect(createImageTask).toHaveBeenLastCalledWith(
      model,
      second,
      params.prompt,
      params.size,
      params.quality,
      params.count,
      ['https://example.com/cat.png'],
      params.extraJson
    )
  })

  it('deletes a task and clears the selection', async () => {
    await seedModel()
    const taskId = await useImagePlaygroundStore
      .getState()
      .createImageTask(model.id, params)

    await useImagePlaygroundStore.getState().deleteImageTask(taskId as string)

    expect(useImagePlaygroundStore.getState().imageTasks).toEqual([])
    expect(useImagePlaygroundStore.getState().selectedImageTaskId).toBeNull()
  })
})
