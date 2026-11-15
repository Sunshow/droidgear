import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import { toast } from 'sonner'
import {
  commands,
  type VideoModel,
  type VideoProtocol,
  type VideoSource,
  type VideoTaskParams,
  type VideoTaskRecord,
  type VideoTaskSnapshot,
  type VideoTaskStatus,
} from '@/lib/bindings'
import i18n from '@/i18n/config'
import { parseImageUrls } from '@/lib/video-utils'

export type {
  VideoModel,
  VideoProtocol,
  VideoSource,
  VideoTaskParams,
  VideoTaskStatus,
}

export type VideoModelDraft = Omit<VideoModel, 'id'>
export type VideoTask = VideoTaskRecord

export interface VideoDraft extends VideoTaskParams {
  modelId: string
}

export function isActiveVideoTask(task: VideoTask): boolean {
  return task.status === 'queued' || task.status === 'in_progress'
}

function isTerminalVideoStatus(status: VideoTaskStatus): boolean {
  return status === 'completed' || status === 'failed' || status === 'cancelled'
}

function newId(prefix: string): string {
  return `${prefix}_${crypto.randomUUID().slice(0, 8)}`
}

const EMPTY_DRAFT: VideoDraft = {
  modelId: '',
  prompt: '',
  duration: '',
  resolution: '',
  imageUrl: '',
  extraJson: '',
}

async function persistVideoModels(models: VideoModel[]): Promise<boolean> {
  try {
    const result = await commands.saveVideoModels(models)
    if (result.status === 'error') {
      toast.error(i18n.t('playground.video.errors.saveModels'), {
        description: result.error,
      })
      return false
    }
    return true
  } catch (error) {
    toast.error(i18n.t('playground.video.errors.saveModels'), {
      description: String(error),
    })
    return false
  }
}

async function saveVideoTasks(tasks: VideoTask[]): Promise<boolean> {
  try {
    const result = await commands.saveVideoTasks(tasks)
    if (result.status === 'error') {
      toast.error(i18n.t('playground.video.errors.saveTasks'), {
        description: result.error,
      })
      return false
    }
    return true
  } catch (error) {
    toast.error(i18n.t('playground.video.errors.saveTasks'), {
      description: String(error),
    })
    return false
  }
}

// Task writes are serialized and always persist the latest state, so a slow
// refresh can never overwrite a task that was created meanwhile.
let taskPersistChain: Promise<void> = Promise.resolve()
let taskPersistScheduled = false

// Deduplicate the initial task load. A create that arrives while the history
// is still loading must wait for it, otherwise its first save would overwrite
// the persisted history with just the new task.
let videoTasksLoadPromise: Promise<void> | null = null

function scheduleVideoTaskPersistence(): Promise<void> {
  if (!taskPersistScheduled) {
    taskPersistScheduled = true
    taskPersistChain = taskPersistChain.then(async () => {
      taskPersistScheduled = false
      await saveVideoTasks(usePlaygroundStore.getState().videoTasks)
    })
  }
  return taskPersistChain
}

interface PlaygroundState {
  videoModels: VideoModel[]
  videoModelsLoaded: boolean
  videoModelsLoading: boolean
  videoTasks: VideoTask[]
  videoTasksLoaded: boolean
  videoTasksLoading: boolean
  videoDraft: VideoDraft
  selectedVideoTaskId: string | null
  autoRefresh: boolean
  refreshIntervalSec: number
  isRefreshing: boolean
  isCreating: boolean
  lastRefreshedAt: number | null

  loadVideoModels: () => Promise<void>
  loadVideoTasks: () => Promise<void>
  addVideoModel: (draft: VideoModelDraft) => Promise<VideoModel | null>
  addVideoModels: (drafts: VideoModelDraft[]) => Promise<VideoModel[]>
  updateVideoModel: (id: string, draft: VideoModelDraft) => Promise<void>
  deleteVideoModel: (id: string) => Promise<void>
  updateVideoDraft: (patch: Partial<VideoDraft>) => void
  createVideoTask: (
    modelId: string,
    params: VideoTaskParams
  ) => Promise<string | null>
  rerunVideoTask: (taskId: string) => Promise<string | null>
  refreshVideoTasks: (taskId?: string) => Promise<void>
  cancelVideoTask: (id: string) => Promise<void>
  deleteVideoTask: (id: string) => Promise<void>
  selectVideoTask: (id: string | null) => void
  setAutoRefresh: (enabled: boolean) => void
  setRefreshIntervalSec: (seconds: number) => void
}

export const usePlaygroundStore = create<PlaygroundState>()(
  devtools(
    (set, get) => {
      async function submitVideoTask(
        model: VideoModel,
        params: VideoTaskParams
      ): Promise<string | null> {
        if (get().isCreating) return null
        set({ isCreating: true }, undefined, 'createVideoTask/start')
        try {
          // Wait for the persisted history before the first save; otherwise
          // this task's save would replace the history with a single record.
          if (!get().videoTasksLoaded) await get().loadVideoTasks()

          const rawDuration = params.duration.trim().replace(/s$/i, '')
          const duration = rawDuration === '' ? null : Number(rawDuration)
          if (
            duration !== null &&
            (!Number.isInteger(duration) || duration <= 0 || duration > 3600)
          ) {
            toast.error(i18n.t('playground.video.errors.invalidDuration'))
            return null
          }

          const imageUrls = parseImageUrls(params.imageUrl)

          const result = await commands.createVideoTask(
            model,
            params.prompt,
            duration,
            params.resolution.trim() || null,
            imageUrls,
            params.extraJson
          )
          if (result.status === 'error') {
            toast.error(i18n.t('playground.video.errors.createTask'), {
              description: result.error,
            })
            return null
          }

          const snapshot = result.data
          const now = Date.now()
          const task: VideoTask = {
            id: snapshot.id,
            modelId: model.id,
            modelName: model.name,
            model,
            params,
            status: snapshot.status,
            progress: snapshot.progress,
            createdAt: now,
            completedAt:
              snapshot.completedAt ??
              (isTerminalVideoStatus(snapshot.status) ? now : null),
            videoUrl: snapshot.videoUrl,
            localPath: snapshot.localPath,
            error: snapshot.error,
          }
          set(
            state => ({
              videoTasks: [task, ...state.videoTasks],
              selectedVideoTaskId: task.id,
            }),
            undefined,
            'createVideoTask/success'
          )
          await scheduleVideoTaskPersistence()
          return task.id
        } catch (error) {
          toast.error(i18n.t('playground.video.errors.createTask'), {
            description: String(error),
          })
          return null
        } finally {
          set({ isCreating: false }, undefined, 'createVideoTask/done')
        }
      }

      function applySnapshots(
        snapshots: Map<string, VideoTaskSnapshot>,
        now: number
      ): boolean {
        const state = get()
        if (snapshots.size === 0) {
          set(
            { isRefreshing: false, lastRefreshedAt: now },
            undefined,
            'refreshVideoTasks/none'
          )
          return false
        }

        const nextTasks = state.videoTasks.map(task => {
          const snapshot = snapshots.get(task.id)
          if (!snapshot) return task
          return {
            ...task,
            status: snapshot.status,
            progress: snapshot.progress,
            completedAt:
              snapshot.completedAt ??
              (isTerminalVideoStatus(snapshot.status)
                ? (task.completedAt ?? now)
                : null),
            videoUrl: snapshot.videoUrl ?? task.videoUrl,
            localPath: snapshot.localPath ?? task.localPath,
            error: snapshot.error,
          }
        })
        const changed = nextTasks.some((task, index) => {
          const previous = state.videoTasks[index]
          return (
            previous !== undefined &&
            (task.status !== previous.status ||
              task.completedAt !== previous.completedAt ||
              task.videoUrl !== previous.videoUrl ||
              task.localPath !== previous.localPath ||
              task.error !== previous.error)
          )
        })

        set(
          { videoTasks: nextTasks, isRefreshing: false, lastRefreshedAt: now },
          undefined,
          'refreshVideoTasks/done'
        )
        return changed
      }

      return {
        videoModels: [],
        videoModelsLoaded: false,
        videoModelsLoading: false,
        videoTasks: [],
        videoTasksLoaded: false,
        videoTasksLoading: false,
        videoDraft: EMPTY_DRAFT,
        selectedVideoTaskId: null,
        autoRefresh: true,
        refreshIntervalSec: 5,
        isRefreshing: false,
        isCreating: false,
        lastRefreshedAt: null,

        loadVideoModels: async () => {
          if (get().videoModelsLoading) return
          set({ videoModelsLoading: true }, undefined, 'loadVideoModels/start')
          try {
            const result = await commands.loadVideoModels()
            if (result.status === 'error') {
              set(
                { videoModelsLoading: false },
                undefined,
                'loadVideoModels/error'
              )
              toast.error(i18n.t('playground.video.errors.loadModels'), {
                description: result.error,
              })
              return
            }

            const models = result.data
            set(
              state => {
                const selectedStillExists = models.some(
                  model => model.id === state.videoDraft.modelId
                )
                return {
                  videoModels: models,
                  videoModelsLoading: false,
                  videoModelsLoaded: true,
                  videoDraft: selectedStillExists
                    ? state.videoDraft
                    : {
                        ...state.videoDraft,
                        modelId: models[0]?.id ?? '',
                        duration: '',
                        resolution: '',
                      },
                }
              },
              undefined,
              'loadVideoModels/success'
            )
          } catch (error) {
            set(
              { videoModelsLoading: false },
              undefined,
              'loadVideoModels/exception'
            )
            toast.error(i18n.t('playground.video.errors.loadModels'), {
              description: String(error),
            })
          }
        },

        loadVideoTasks: async () => {
          if (get().videoTasksLoaded) return
          if (videoTasksLoadPromise) {
            await videoTasksLoadPromise
            return
          }

          videoTasksLoadPromise = (async () => {
            set({ videoTasksLoading: true }, undefined, 'loadVideoTasks/start')
            try {
              const result = await commands.loadVideoTasks()
              if (result.status === 'error') {
                set(
                  { videoTasksLoading: false },
                  undefined,
                  'loadVideoTasks/error'
                )
                toast.error(i18n.t('playground.video.errors.loadTasks'), {
                  description: result.error,
                })
                return
              }

              const current = get().videoTasks
              const merged = [
                ...current,
                ...result.data.filter(
                  task => !current.some(item => item.id === task.id)
                ),
              ]
              set(
                {
                  videoTasks: merged,
                  videoTasksLoading: false,
                  videoTasksLoaded: true,
                  selectedVideoTaskId:
                    get().selectedVideoTaskId ?? merged[0]?.id ?? null,
                },
                undefined,
                'loadVideoTasks/success'
              )
            } catch (error) {
              set(
                { videoTasksLoading: false },
                undefined,
                'loadVideoTasks/exception'
              )
              toast.error(i18n.t('playground.video.errors.loadTasks'), {
                description: String(error),
              })
            }
          })()

          try {
            await videoTasksLoadPromise
          } finally {
            videoTasksLoadPromise = null
          }
        },

        addVideoModels: async drafts => {
          if (drafts.length === 0) return []
          const previousModels = get().videoModels
          const previousDraft = get().videoDraft
          const created = drafts.map(draft => ({ ...draft, id: newId('vm') }))
          const first = created[0]
          set(
            {
              videoModels: [...previousModels, ...created],
              videoDraft: first
                ? {
                    ...previousDraft,
                    modelId: first.id,
                    duration: '',
                    resolution: '',
                  }
                : previousDraft,
            },
            undefined,
            'addVideoModels'
          )

          if (!(await persistVideoModels(get().videoModels))) {
            set(
              { videoModels: previousModels, videoDraft: previousDraft },
              undefined,
              'addVideoModels/rollback'
            )
            return []
          }
          return created
        },

        addVideoModel: async draft => {
          const created = await get().addVideoModels([draft])
          return created[0] ?? null
        },

        updateVideoModel: async (id, draft) => {
          const previousModels = get().videoModels
          const models = previousModels.map(model =>
            model.id === id ? { ...draft, id } : model
          )
          set({ videoModels: models }, undefined, 'updateVideoModel')
          if (!(await persistVideoModels(models))) {
            set(
              { videoModels: previousModels },
              undefined,
              'updateVideoModel/rollback'
            )
          }
        },

        deleteVideoModel: async id => {
          const previousModels = get().videoModels
          const previousDraft = get().videoDraft
          const models = previousModels.filter(model => model.id !== id)
          const draft =
            previousDraft.modelId === id
              ? {
                  ...previousDraft,
                  modelId: models[0]?.id ?? '',
                  duration: '',
                  resolution: '',
                }
              : previousDraft
          set(
            { videoModels: models, videoDraft: draft },
            undefined,
            'deleteVideoModel'
          )
          if (!(await persistVideoModels(models))) {
            set(
              { videoModels: previousModels, videoDraft: previousDraft },
              undefined,
              'deleteVideoModel/rollback'
            )
          }
        },

        updateVideoDraft: patch =>
          set(
            state => ({ videoDraft: { ...state.videoDraft, ...patch } }),
            undefined,
            'updateVideoDraft'
          ),

        createVideoTask: async (modelId, params) => {
          const model = get().videoModels.find(item => item.id === modelId)
          if (!model) {
            toast.error(i18n.t('playground.video.errors.modelMissing'))
            return null
          }
          return submitVideoTask(model, params)
        },

        rerunVideoTask: async taskId => {
          const task = get().videoTasks.find(item => item.id === taskId)
          if (!task) return null
          const model =
            get().videoModels.find(item => item.id === task.modelId) ??
            task.model
          return submitVideoTask(model, task.params)
        },

        refreshVideoTasks: async taskId => {
          if (get().isRefreshing) return
          const targets = get().videoTasks.filter(task =>
            taskId === undefined ? isActiveVideoTask(task) : task.id === taskId
          )
          if (targets.length === 0) {
            set(
              { lastRefreshedAt: Date.now() },
              undefined,
              'refreshVideoTasks/none'
            )
            return
          }

          set({ isRefreshing: true }, undefined, 'refreshVideoTasks/start')
          const snapshots = new Map<string, VideoTaskSnapshot>()
          let failure: string | null = null
          let changed = false
          try {
            for (const task of targets) {
              try {
                const result = await commands.queryVideoTask(
                  task.model,
                  task.id
                )
                if (result.status === 'error') {
                  failure = result.error
                  continue
                }
                snapshots.set(task.id, result.data)
              } catch (error) {
                failure = String(error)
              }
            }
          } finally {
            changed = applySnapshots(snapshots, Date.now())
          }
          if (changed) await scheduleVideoTaskPersistence()

          if (failure && taskId !== undefined) {
            toast.error(i18n.t('playground.video.errors.queryTask'), {
              description: failure,
            })
          }
        },

        cancelVideoTask: async id => {
          set(
            state => ({
              videoTasks: state.videoTasks.map(task =>
                task.id === id && isActiveVideoTask(task)
                  ? {
                      ...task,
                      status: 'cancelled',
                      completedAt: Date.now(),
                    }
                  : task
              ),
            }),
            undefined,
            'cancelVideoTask'
          )
          await scheduleVideoTaskPersistence()
        },

        deleteVideoTask: async id => {
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
          )
          await scheduleVideoTaskPersistence()
        },

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
      }
    },
    { name: 'playground-store' }
  )
)
