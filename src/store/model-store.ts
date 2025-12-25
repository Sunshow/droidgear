import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import { commands, type CustomModel } from '@/lib/bindings'

interface ModelState {
  models: CustomModel[]
  originalModels: CustomModel[]
  configPath: string
  hasChanges: boolean
  isLoading: boolean
  error: string | null

  // Actions
  loadModels: () => Promise<void>
  saveModels: () => Promise<void>
  addModel: (model: CustomModel) => void
  updateModel: (index: number, model: CustomModel) => void
  deleteModel: (index: number) => void
  reorderModels: (fromIndex: number, toIndex: number) => void
  resetChanges: () => void
  setError: (error: string | null) => void
}

function modelsEqual(a: CustomModel[], b: CustomModel[]): boolean {
  if (a.length !== b.length) return false
  return JSON.stringify(a) === JSON.stringify(b)
}

export const useModelStore = create<ModelState>()(
  devtools(
    (set, get) => ({
      models: [],
      originalModels: [],
      configPath: '~/.factory/config.json',
      hasChanges: false,
      isLoading: false,
      error: null,

      loadModels: async () => {
        set({ isLoading: true, error: null }, undefined, 'loadModels/start')
        try {
          const [pathResult, modelsResult] = await Promise.all([
            commands.getConfigPath(),
            commands.loadCustomModels(),
          ])

          if (pathResult.status === 'ok') {
            set({ configPath: pathResult.data }, undefined, 'loadModels/setPath')
          }

          if (modelsResult.status === 'ok') {
            set(
              {
                models: modelsResult.data,
                originalModels: JSON.parse(JSON.stringify(modelsResult.data)),
                hasChanges: false,
                isLoading: false,
              },
              undefined,
              'loadModels/success'
            )
          } else {
            set(
              { error: modelsResult.error, isLoading: false },
              undefined,
              'loadModels/error'
            )
          }
        } catch (e) {
          set(
            { error: String(e), isLoading: false },
            undefined,
            'loadModels/exception'
          )
        }
      },

      saveModels: async () => {
        const { models } = get()
        set({ isLoading: true, error: null }, undefined, 'saveModels/start')
        try {
          const result = await commands.saveCustomModels(models)
          if (result.status === 'ok') {
            set(
              {
                originalModels: JSON.parse(JSON.stringify(models)),
                hasChanges: false,
                isLoading: false,
              },
              undefined,
              'saveModels/success'
            )
          } else {
            set(
              { error: result.error, isLoading: false },
              undefined,
              'saveModels/error'
            )
          }
        } catch (e) {
          set(
            { error: String(e), isLoading: false },
            undefined,
            'saveModels/exception'
          )
        }
      },

      addModel: model => {
        set(
          state => {
            const newModels = [...state.models, model]
            return {
              models: newModels,
              hasChanges: !modelsEqual(newModels, state.originalModels),
            }
          },
          undefined,
          'addModel'
        )
      },

      updateModel: (index, model) => {
        set(
          state => {
            const newModels = [...state.models]
            newModels[index] = model
            return {
              models: newModels,
              hasChanges: !modelsEqual(newModels, state.originalModels),
            }
          },
          undefined,
          'updateModel'
        )
      },

      deleteModel: index => {
        set(
          state => {
            const newModels = state.models.filter((_, i) => i !== index)
            return {
              models: newModels,
              hasChanges: !modelsEqual(newModels, state.originalModels),
            }
          },
          undefined,
          'deleteModel'
        )
      },

      reorderModels: (fromIndex, toIndex) => {
        set(
          state => {
            const newModels = [...state.models]
            const removed = newModels.splice(fromIndex, 1)[0]
            if (removed) {
              newModels.splice(toIndex, 0, removed)
            }
            return {
              models: newModels,
              hasChanges: !modelsEqual(newModels, state.originalModels),
            }
          },
          undefined,
          'reorderModels'
        )
      },

      resetChanges: () => {
        set(
          state => ({
            models: JSON.parse(JSON.stringify(state.originalModels)),
            hasChanges: false,
          }),
          undefined,
          'resetChanges'
        )
      },

      setError: error => set({ error }, undefined, 'setError'),
    }),
    { name: 'model-store' }
  )
)
