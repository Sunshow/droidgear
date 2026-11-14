import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { usePlaygroundStore } from './playground-store'

const initialState = usePlaygroundStore.getState()

const params = {
  prompt: 'a cat',
  duration: '5',
  resolution: '1280x720',
  imageUrl: '',
  extraJson: '',
}

async function refresh(taskId?: string) {
  const pending = usePlaygroundStore.getState().refreshVideoTasks(taskId)
  await vi.runAllTimersAsync()
  await pending
}

describe('playground store (demo)', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    usePlaygroundStore.setState(initialState, true)
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('adds a model and selects it in the create form', () => {
    const model = usePlaygroundStore.getState().addVideoModel({
      name: 'Test',
      modelId: 'test-video',
      source: { kind: 'custom', baseUrl: 'https://x', apiKey: '' },
      durations: [5],
      resolutions: [],
      supportsImageInput: false,
    })

    const state = usePlaygroundStore.getState()
    expect(state.videoModels.at(-1)?.id).toBe(model.id)
    expect(state.videoDraft.modelId).toBe(model.id)
  })

  it('creates a queued task at the top of the list and selects it', () => {
    const id = usePlaygroundStore
      .getState()
      .createVideoTask('demo-kling', params)

    const state = usePlaygroundStore.getState()
    expect(state.videoTasks[0]).toMatchObject({
      id,
      status: 'queued',
      modelName: 'Kling v2',
    })
    expect(state.selectedVideoTaskId).toBe(id)
  })

  it('advances a task to completed with a playable URL after refreshes', async () => {
    const id = usePlaygroundStore
      .getState()
      .createVideoTask('demo-kling', params)

    for (let i = 0; i < 10; i++) await refresh(id)

    const task = usePlaygroundStore
      .getState()
      .videoTasks.find(item => item.id === id)
    expect(task?.status).toBe('completed')
    expect(task?.progress).toBe(100)
    expect(task?.videoUrl).toMatch(/\.mp4$/)
    expect(usePlaygroundStore.getState().lastRefreshedAt).not.toBeNull()
  })

  it('refreshing one task leaves other running tasks untouched', async () => {
    const first = usePlaygroundStore
      .getState()
      .createVideoTask('demo-kling', params)
    const second = usePlaygroundStore
      .getState()
      .createVideoTask('demo-kling', params)

    await refresh(first)

    const tasks = usePlaygroundStore.getState().videoTasks
    expect(tasks.find(t => t.id === first)?.status).toBe('in_progress')
    expect(tasks.find(t => t.id === second)?.status).toBe('queued')
  })

  it('stops refreshing a cancelled task', async () => {
    const id = usePlaygroundStore
      .getState()
      .createVideoTask('demo-kling', params)
    usePlaygroundStore.getState().cancelVideoTask(id)

    await refresh()

    const task = usePlaygroundStore
      .getState()
      .videoTasks.find(item => item.id === id)
    expect(task?.status).toBe('cancelled')
  })

  it('clears the selection when the selected task is deleted', () => {
    const id = usePlaygroundStore
      .getState()
      .createVideoTask('demo-kling', params)
    usePlaygroundStore.getState().deleteVideoTask(id)

    const state = usePlaygroundStore.getState()
    expect(state.videoTasks.some(task => task.id === id)).toBe(false)
    expect(state.selectedVideoTaskId).toBeNull()
  })
})
