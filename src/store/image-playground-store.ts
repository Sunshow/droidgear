import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import { toast } from 'sonner'
import {
  commands,
  type ImageAsset,
  type ImageModel,
  type ImageProtocol,
  type ImageTaskParams,
  type ImageTaskRecord,
  type ImageTaskSnapshot,
  type ImageTaskStatus,
} from '@/lib/bindings'
import i18n from '@/i18n/config'
import { parseImageUrls } from '@/lib/playground-utils'

export type {
  ImageAsset,
  ImageModel,
  ImageProtocol,
  ImageTaskParams,
  ImageTaskStatus,
}

export type ImageModelDraft = Omit<ImageModel, 'id'>
export type ImageTask = ImageTaskRecord

export interface ImageDraft extends ImageTaskParams {
  modelId: string
}

export function isRunningImageTask(task: ImageTask): boolean {
  return task.status === 'running'
}

function newId(prefix: string): string {
  return `${prefix}_${crypto.randomUUID().slice(0, 8)}`
}

const EMPTY_DRAFT: ImageDraft = {
  modelId: '',
  prompt: '',
  size: '',
  quality: '',
  count: 1,
  imageUrl: '',
  extraJson: '',
}

async function persistImageModels(models: ImageModel[]): Promise<boolean> {
  try {
    const result = await commands.saveImageModels(models)
    if (result.status === 'error') {
      toast.error(i18n.t('playground.image.errors.saveModels'), {
        description: result.error,
      })
      return false
    }
    return true
  } catch (error) {
    toast.error(i18n.t('playground.image.errors.saveModels'), {
      description: String(error),
    })
    return false
  }
}

async function saveImageTasks(tasks: ImageTask[]): Promise<boolean> {
  try {
    const result = await commands.saveImageTasks(tasks)
    if (result.status === 'error') {
      toast.error(i18n.t('playground.image.errors.saveTasks'), {
        description: result.error,
      })
      return false
    }
    return true
  } catch (error) {
    toast.error(i18n.t('playground.image.errors.saveTasks'), {
      description: String(error),
    })
    return false
  }
}

// Task writes are serialized and always persist the latest state, so a slow
// write can never overwrite a task that was created meanwhile.
let taskPersistChain: Promise<void> = Promise.resolve()
let taskPersistScheduled = false

// Deduplicate the initial task load. A create that arrives while the history
// is still loading must wait for it, otherwise its first save would overwrite
// the persisted history with just the new task.
let imageTasksLoadPromise: Promise<void> | null = null

function scheduleImageTaskPersistence(): Promise<void> {
  if (!taskPersistScheduled) {
    taskPersistScheduled = true
    taskPersistChain = taskPersistChain.then(async () => {
      taskPersistScheduled = false
      await saveImageTasks(useImagePlaygroundStore.getState().imageTasks)
    })
  }
  return taskPersistChain
}

function taskFromSnapshot(
  task: ImageTask,
  snapshot: ImageTaskSnapshot
): ImageTask {
  return {
    ...task,
    status: 'completed',
    images: snapshot.images,
    revisedPrompt: snapshot.revisedPrompt,
    responseText: snapshot.responseText,
    completedAt: snapshot.completedAt,
    error: null,
  }
}

interface ImagePlaygroundState {
  imageModels: ImageModel[]
  imageModelsLoaded: boolean
  imageModelsLoading: boolean
  imageTasks: ImageTask[]
  imageTasksLoaded: boolean
  imageTasksLoading: boolean
  imageDraft: ImageDraft
  selectedImageTaskId: string | null
  isCreating: boolean

  loadImageModels: () => Promise<void>
  loadImageTasks: () => Promise<void>
  addImageModel: (draft: ImageModelDraft) => Promise<ImageModel | null>
  addImageModels: (drafts: ImageModelDraft[]) => Promise<ImageModel[]>
  updateImageModel: (id: string, draft: ImageModelDraft) => Promise<void>
  deleteImageModel: (id: string) => Promise<void>
  updateImageDraft: (patch: Partial<ImageDraft>) => void
  createImageTask: (
    modelId: string,
    params: ImageTaskParams
  ) => Promise<string | null>
  rerunImageTask: (taskId: string) => Promise<string | null>
  deleteImageTask: (id: string) => Promise<void>
  selectImageTask: (id: string | null) => void
}

export const useImagePlaygroundStore = create<ImagePlaygroundState>()(
  devtools(
    (set, get) => {
      async function submitImageTask(
        model: ImageModel,
        params: ImageTaskParams
      ): Promise<string | null> {
        if (get().isCreating) return null
        set({ isCreating: true }, undefined, 'createImageTask/start')

        const taskId = newId('img')
        const startedAt = Date.now()
        const task: ImageTask = {
          id: taskId,
          modelId: model.id,
          modelName: model.name,
          model,
          params,
          status: 'running',
          images: [] as ImageAsset[],
          revisedPrompt: null,
          responseText: null,
          createdAt: startedAt,
          completedAt: null,
          error: null,
        }

        try {
          // Wait for the persisted history before the first save; otherwise
          // this task's save would replace the history with a single record.
          if (!get().imageTasksLoaded) await get().loadImageTasks()

          const count = Math.min(
            8,
            Math.max(1, Math.trunc(Number(params.count) || 1))
          )
          set(
            state => ({
              imageTasks: [task, ...state.imageTasks],
              selectedImageTaskId: taskId,
            }),
            undefined,
            'createImageTask/insert'
          )
          await scheduleImageTaskPersistence()

          const result = await commands.createImageTask(
            model,
            taskId,
            params.prompt.trim(),
            params.size.trim(),
            params.quality.trim(),
            count,
            parseImageUrls(params.imageUrl),
            params.extraJson
          )

          if (result.status === 'error') {
            set(
              state => ({
                imageTasks: state.imageTasks.map(item =>
                  item.id === taskId && item.status === 'running'
                    ? {
                        ...item,
                        status: 'failed',
                        completedAt: Date.now(),
                        error: result.error,
                      }
                    : item
                ),
              }),
              undefined,
              'createImageTask/error'
            )
            toast.error(i18n.t('playground.image.errors.createTask'), {
              description: result.error,
            })
            return taskId
          }

          if (
            useImagePlaygroundStore
              .getState()
              .imageTasks.some(
                item => item.id === taskId && item.status === 'running'
              )
          ) {
            set(
              state => ({
                imageTasks: state.imageTasks.map(item =>
                  item.id === taskId && item.status === 'running'
                    ? taskFromSnapshot(item, result.data)
                    : item
                ),
              }),
              undefined,
              'createImageTask/success'
            )
            toast.success(
              i18n.t('playground.image.create.generated', {
                count: result.data.images.length,
              })
            )
          }
          return taskId
        } catch (error) {
          set(
            state => ({
              imageTasks: state.imageTasks.map(item =>
                item.id === taskId && item.status === 'running'
                  ? {
                      ...item,
                      status: 'failed',
                      completedAt: Date.now(),
                      error: String(error),
                    }
                  : item
              ),
            }),
            undefined,
            'createImageTask/exception'
          )
          toast.error(i18n.t('playground.image.errors.createTask'), {
            description: String(error),
          })
          return taskId
        } finally {
          set({ isCreating: false }, undefined, 'createImageTask/done')
          await scheduleImageTaskPersistence()
        }
      }

      return {
        imageModels: [],
        imageModelsLoaded: false,
        imageModelsLoading: false,
        imageTasks: [],
        imageTasksLoaded: false,
        imageTasksLoading: false,
        imageDraft: EMPTY_DRAFT,
        selectedImageTaskId: null,
        isCreating: false,

        loadImageModels: async () => {
          if (get().imageModelsLoading) return
          set({ imageModelsLoading: true }, undefined, 'loadImageModels/start')
          try {
            const result = await commands.loadImageModels()
            if (result.status === 'error') {
              set(
                { imageModelsLoading: false },
                undefined,
                'loadImageModels/error'
              )
              toast.error(i18n.t('playground.image.errors.loadModels'), {
                description: result.error,
              })
              return
            }

            const models = result.data
            set(
              state => {
                const selectedStillExists = models.some(
                  model => model.id === state.imageDraft.modelId
                )
                return {
                  imageModels: models,
                  imageModelsLoading: false,
                  imageModelsLoaded: true,
                  imageDraft: selectedStillExists
                    ? state.imageDraft
                    : {
                        ...state.imageDraft,
                        modelId: models[0]?.id ?? '',
                        size: '',
                        quality: '',
                      },
                }
              },
              undefined,
              'loadImageModels/success'
            )
          } catch (error) {
            set(
              { imageModelsLoading: false },
              undefined,
              'loadImageModels/exception'
            )
            toast.error(i18n.t('playground.image.errors.loadModels'), {
              description: String(error),
            })
          }
        },

        loadImageTasks: async () => {
          if (get().imageTasksLoaded) return
          if (imageTasksLoadPromise) {
            await imageTasksLoadPromise
            return
          }

          imageTasksLoadPromise = (async () => {
            set({ imageTasksLoading: true }, undefined, 'loadImageTasks/start')
            try {
              const result = await commands.loadImageTasks()
              if (result.status === 'error') {
                set(
                  { imageTasksLoading: false },
                  undefined,
                  'loadImageTasks/error'
                )
                toast.error(i18n.t('playground.image.errors.loadTasks'), {
                  description: result.error,
                })
                return
              }

              // Generation is a single request, so a `running` record can only
              // come from an app that closed before the answer arrived.
              const interruptedAt = Date.now()
              let sawInterrupted = false
              const persisted = result.data.map(task => {
                if (task.status !== 'running') return task
                sawInterrupted = true
                return {
                  ...task,
                  status: 'failed' as const,
                  completedAt: task.completedAt ?? interruptedAt,
                  error: i18n.t('playground.image.errors.interrupted'),
                }
              })
              const current = get().imageTasks
              const merged = [
                ...current,
                ...persisted.filter(
                  task => !current.some(item => item.id === task.id)
                ),
              ]
              set(
                {
                  imageTasks: merged,
                  imageTasksLoading: false,
                  imageTasksLoaded: true,
                  selectedImageTaskId:
                    get().selectedImageTaskId ?? merged[0]?.id ?? null,
                },
                undefined,
                'loadImageTasks/success'
              )
              if (sawInterrupted) await scheduleImageTaskPersistence()
            } catch (error) {
              set(
                { imageTasksLoading: false },
                undefined,
                'loadImageTasks/exception'
              )
              toast.error(i18n.t('playground.image.errors.loadTasks'), {
                description: String(error),
              })
            }
          })()

          try {
            await imageTasksLoadPromise
          } finally {
            imageTasksLoadPromise = null
          }
        },

        addImageModels: async drafts => {
          if (drafts.length === 0) return []
          const previousModels = get().imageModels
          const previousDraft = get().imageDraft
          const created = drafts.map(draft => ({ ...draft, id: newId('im') }))
          const first = created[0]
          set(
            {
              imageModels: [...previousModels, ...created],
              imageDraft: first
                ? {
                    ...previousDraft,
                    modelId: first.id,
                    size: '',
                    quality: '',
                    count: 1,
                  }
                : previousDraft,
            },
            undefined,
            'addImageModels'
          )

          if (!(await persistImageModels(get().imageModels))) {
            set(
              { imageModels: previousModels, imageDraft: previousDraft },
              undefined,
              'addImageModels/rollback'
            )
            return []
          }
          return created
        },

        addImageModel: async draft => {
          const created = await get().addImageModels([draft])
          return created[0] ?? null
        },

        updateImageModel: async (id, draft) => {
          const previousModels = get().imageModels
          const models = previousModels.map(model =>
            model.id === id ? { ...draft, id } : model
          )
          set({ imageModels: models }, undefined, 'updateImageModel')
          if (!(await persistImageModels(models))) {
            set(
              { imageModels: previousModels },
              undefined,
              'updateImageModel/rollback'
            )
          }
        },

        deleteImageModel: async id => {
          const previousModels = get().imageModels
          const previousDraft = get().imageDraft
          const models = previousModels.filter(model => model.id !== id)
          const draft =
            previousDraft.modelId === id
              ? {
                  ...previousDraft,
                  modelId: models[0]?.id ?? '',
                  size: '',
                  quality: '',
                }
              : previousDraft
          set(
            { imageModels: models, imageDraft: draft },
            undefined,
            'deleteImageModel'
          )
          if (!(await persistImageModels(models))) {
            set(
              { imageModels: previousModels, imageDraft: previousDraft },
              undefined,
              'deleteImageModel/rollback'
            )
          }
        },

        updateImageDraft: patch =>
          set(
            state => ({ imageDraft: { ...state.imageDraft, ...patch } }),
            undefined,
            'updateImageDraft'
          ),

        createImageTask: async (modelId, params) => {
          const model = get().imageModels.find(item => item.id === modelId)
          if (!model) {
            toast.error(i18n.t('playground.image.errors.modelMissing'))
            return null
          }
          return submitImageTask(model, params)
        },

        rerunImageTask: async taskId => {
          const task = get().imageTasks.find(item => item.id === taskId)
          if (!task) return null
          const model =
            get().imageModels.find(item => item.id === task.modelId) ??
            task.model
          return submitImageTask(model, task.params)
        },

        deleteImageTask: async id => {
          set(
            state => ({
              imageTasks: state.imageTasks.filter(task => task.id !== id),
              selectedImageTaskId:
                state.selectedImageTaskId === id
                  ? null
                  : state.selectedImageTaskId,
            }),
            undefined,
            'deleteImageTask'
          )
          await scheduleImageTaskPersistence()
        },

        selectImageTask: id =>
          set({ selectedImageTaskId: id }, undefined, 'selectImageTask'),
      }
    },
    { name: 'image-playground-store' }
  )
)
