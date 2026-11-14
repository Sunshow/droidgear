import { create } from 'zustand'
import { devtools } from 'zustand/middleware'

export type VideoModelSource =
  | { kind: 'channel'; channelId: string }
  | { kind: 'custom'; baseUrl: string; apiKey: string }

export interface VideoModel {
  id: string
  name: string
  /** Model name sent to the provider API */
  modelId: string
  source: VideoModelSource
  durations: number[]
  resolutions: string[]
  supportsImageInput: boolean
}

export type VideoModelDraft = Omit<VideoModel, 'id'>

export type VideoTaskStatus =
  | 'queued'
  | 'in_progress'
  | 'completed'
  | 'failed'
  | 'cancelled'

export interface VideoTaskParams {
  prompt: string
  duration: string
  resolution: string
  imageUrl: string
  extraJson: string
}

export interface VideoDraft extends VideoTaskParams {
  modelId: string
}

export interface VideoTask {
  id: string
  modelId: string
  /** Snapshot so the task still reads well after its model is deleted */
  modelName: string
  params: VideoTaskParams
  status: VideoTaskStatus
  progress: number
  createdAt: number
  completedAt: number | null
  videoUrl: string | null
  error: string | null
}

export function isActiveVideoTask(task: VideoTask): boolean {
  return task.status === 'queued' || task.status === 'in_progress'
}

// Demo-only: everything below the store types simulates provider behaviour so
// the UI can be reviewed before real model calls exist.
const DEMO_VIDEO_URL =
  'https://interactive-examples.mdn.mozilla.net/media/cc0-videos/flower.mp4'
const DEMO_LATENCY_MS = 400

function newId(prefix: string): string {
  return `${prefix}_${crypto.randomUUID().slice(0, 8)}`
}

function advanceDemoTask(task: VideoTask, now: number): VideoTask {
  if (task.status === 'queued') {
    return { ...task, status: 'in_progress', progress: 5 }
  }
  if (task.status !== 'in_progress') return task
  const progress = Math.min(
    100,
    task.progress + 15 + Math.floor(Math.random() * 20)
  )
  if (progress < 100) return { ...task, progress }
  return {
    ...task,
    status: 'completed',
    progress: 100,
    completedAt: now,
    videoUrl: DEMO_VIDEO_URL,
  }
}

const DEMO_MODELS: VideoModel[] = [
  {
    id: 'demo-kling',
    name: 'Kling v2',
    modelId: 'kling-v2-master',
    source: { kind: 'custom', baseUrl: 'https://api.example.com', apiKey: '' },
    durations: [5, 10],
    resolutions: ['1280x720', '720x1280', '1920x1080'],
    supportsImageInput: true,
  },
  {
    id: 'demo-veo',
    name: 'Veo 3',
    modelId: 'veo-3',
    source: { kind: 'custom', baseUrl: 'https://api.example.com', apiKey: '' },
    durations: [4, 8],
    resolutions: ['1280x720', '720x1280'],
    supportsImageInput: false,
  },
]

function demoTasks(now: number): VideoTask[] {
  const minute = 60_000
  return [
    {
      id: 'task_demo0003',
      modelId: 'demo-kling',
      modelName: 'Kling v2',
      params: {
        prompt: '一只橘猫在雨后的窗台上伸懒腰，镜头缓慢推进，电影感光影',
        duration: '5',
        resolution: '1280x720',
        imageUrl: '',
        extraJson: '',
      },
      status: 'in_progress',
      progress: 40,
      createdAt: now - 2 * minute,
      completedAt: null,
      videoUrl: null,
      error: null,
    },
    {
      id: 'task_demo0002',
      modelId: 'demo-veo',
      modelName: 'Veo 3',
      params: {
        prompt: 'A drone shot over a misty mountain lake at sunrise',
        duration: '8',
        resolution: '1280x720',
        imageUrl: '',
        extraJson: '{\n  "negative_prompt": "blurry"\n}',
      },
      status: 'failed',
      progress: 0,
      createdAt: now - 12 * minute,
      completedAt: now - 11 * minute,
      videoUrl: null,
      error: 'HTTP 400: content policy violation (demo error)',
    },
    {
      id: 'task_demo0001',
      modelId: 'demo-kling',
      modelName: 'Kling v2',
      params: {
        prompt: '微距镜头下一朵花在风中轻轻摇曳',
        duration: '5',
        resolution: '1280x720',
        imageUrl: '',
        extraJson: '',
      },
      status: 'completed',
      progress: 100,
      createdAt: now - 30 * minute,
      completedAt: now - 28 * minute,
      videoUrl: DEMO_VIDEO_URL,
      error: null,
    },
  ]
}

const EMPTY_DRAFT: VideoDraft = {
  modelId: 'demo-kling',
  prompt: '',
  duration: '',
  resolution: '',
  imageUrl: '',
  extraJson: '',
}

interface PlaygroundState {
  videoModels: VideoModel[]
  videoTasks: VideoTask[]
  videoDraft: VideoDraft
  selectedVideoTaskId: string | null
  autoRefresh: boolean
  refreshIntervalSec: number
  isRefreshing: boolean
  lastRefreshedAt: number | null

  addVideoModel: (draft: VideoModelDraft) => VideoModel
  updateVideoModel: (id: string, draft: VideoModelDraft) => void
  deleteVideoModel: (id: string) => void
  updateVideoDraft: (patch: Partial<VideoDraft>) => void
  createVideoTask: (modelId: string, params: VideoTaskParams) => string
  refreshVideoTasks: (taskId?: string) => Promise<void>
  cancelVideoTask: (id: string) => void
  deleteVideoTask: (id: string) => void
  selectVideoTask: (id: string | null) => void
  setAutoRefresh: (enabled: boolean) => void
  setRefreshIntervalSec: (seconds: number) => void
}

export const usePlaygroundStore = create<PlaygroundState>()(
  devtools(
    (set, get) => ({
      videoModels: DEMO_MODELS,
      videoTasks: demoTasks(Date.now()),
      videoDraft: EMPTY_DRAFT,
      selectedVideoTaskId: 'task_demo0001',
      autoRefresh: true,
      refreshIntervalSec: 5,
      isRefreshing: false,
      lastRefreshedAt: null,

      addVideoModel: draft => {
        const model: VideoModel = { ...draft, id: newId('vm') }
        set(
          state => ({
            videoModels: [...state.videoModels, model],
            videoDraft: { ...state.videoDraft, modelId: model.id },
          }),
          undefined,
          'addVideoModel'
        )
        return model
      },

      updateVideoModel: (id, draft) =>
        set(
          state => ({
            videoModels: state.videoModels.map(m =>
              m.id === id ? { ...draft, id } : m
            ),
          }),
          undefined,
          'updateVideoModel'
        ),

      deleteVideoModel: id =>
        set(
          state => ({
            videoModels: state.videoModels.filter(m => m.id !== id),
          }),
          undefined,
          'deleteVideoModel'
        ),

      updateVideoDraft: patch =>
        set(
          state => ({ videoDraft: { ...state.videoDraft, ...patch } }),
          undefined,
          'updateVideoDraft'
        ),

      createVideoTask: (modelId, params) => {
        const model = get().videoModels.find(m => m.id === modelId)
        const task: VideoTask = {
          id: newId('task'),
          modelId,
          modelName: model?.name ?? modelId,
          params,
          status: 'queued',
          progress: 0,
          createdAt: Date.now(),
          completedAt: null,
          videoUrl: null,
          error: null,
        }
        set(
          state => ({
            videoTasks: [task, ...state.videoTasks],
            selectedVideoTaskId: task.id,
          }),
          undefined,
          'createVideoTask'
        )
        return task.id
      },

      refreshVideoTasks: async taskId => {
        if (get().isRefreshing) return
        set({ isRefreshing: true }, undefined, 'refreshVideoTasks/start')
        await new Promise(resolve => setTimeout(resolve, DEMO_LATENCY_MS))
        const now = Date.now()
        set(
          state => ({
            videoTasks: state.videoTasks.map(task =>
              taskId === undefined || task.id === taskId
                ? advanceDemoTask(task, now)
                : task
            ),
            isRefreshing: false,
            lastRefreshedAt: now,
          }),
          undefined,
          'refreshVideoTasks/done'
        )
      },

      cancelVideoTask: id =>
        set(
          state => ({
            videoTasks: state.videoTasks.map(task =>
              task.id === id && isActiveVideoTask(task)
                ? { ...task, status: 'cancelled', completedAt: Date.now() }
                : task
            ),
          }),
          undefined,
          'cancelVideoTask'
        ),

      deleteVideoTask: id =>
        set(
          state => ({
            videoTasks: state.videoTasks.filter(task => task.id !== id),
            selectedVideoTaskId:
              state.selectedVideoTaskId === id
                ? null
                : state.selectedVideoTaskId,
          }),
          undefined,
          'deleteVideoTask'
        ),

      selectVideoTask: id =>
        set({ selectedVideoTaskId: id }, undefined, 'selectVideoTask'),

      setAutoRefresh: enabled =>
        set({ autoRefresh: enabled }, undefined, 'setAutoRefresh'),

      setRefreshIntervalSec: seconds =>
        set(
          { refreshIntervalSec: seconds },
          undefined,
          'setRefreshIntervalSec'
        ),
    }),
    { name: 'playground-store' }
  )
)
