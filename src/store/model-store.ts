import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import { commands, type CustomModel } from '@/lib/bindings'

const CONFIG_PARSE_ERROR_PREFIX = 'CONFIG_PARSE_ERROR:'

interface ModelState {
  models: CustomModel[]
  originalModels: CustomModel[]
  configPath: string
  hasChanges: boolean
  isLoading: boolean
  error: string | null
  configParseError: string | null
  defaultModelId: string | null

  // Actions
  loadModels: () => Promise<void>
  saveModels: () => Promise<void>
  resetConfigAndSave: () => Promise<void>
  addModel: (model: CustomModel) => void
  updateModel: (index: number, model: CustomModel) => void
  deleteModel: (index: number) => void
  deleteModels: (indices: number[]) => void
  reorderModels: (fromIndex: number, toIndex: number) => void
  resetChanges: () => void
  setError: (error: string | null) => void
  clearConfigParseError: () => void
  loadDefaultModel: () => Promise<void>
  setDefaultModel: (index: number) => Promise<void>
}

function modelsEqual(a: CustomModel[], b: CustomModel[]): boolean {
  if (a.length !== b.length) return false
  return JSON.stringify(a) === JSON.stringify(b)
}

function isSameModelConfig(a: CustomModel, b: CustomModel): boolean {
  return a.model === b.model && a.baseUrl === b.baseUrl && a.apiKey === b.apiKey
}

function generateUniqueDisplayName(
  baseDisplayName: string,
  existingModels: CustomModel[],
  newModel: CustomModel
): string {
  const hasConflict = existingModels.some(
    m =>
      (m.displayName || m.model) === baseDisplayName &&
      !isSameModelConfig(m, newModel)
  )

  if (!hasConflict) {
    return baseDisplayName
  }

  let suffix = 1
  while (
    existingModels.some(
      m => (m.displayName || m.model) === `${baseDisplayName}-${suffix}`
    )
  ) {
    suffix++
  }
  return `${baseDisplayName}-${suffix}`
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
      configParseError: null,
      defaultModelId: null,

      loadModels: async () => {
        set({ isLoading: true, error: null }, undefined, 'loadModels/start')
        try {
          const [pathResult, modelsResult] = await Promise.all([
            commands.getConfigPath(),
            commands.loadCustomModels(),
          ])

          if (pathResult.status === 'ok') {
            set(
              { configPath: pathResult.data },
              undefined,
              'loadModels/setPath'
            )
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
        set(
          { isLoading: true, error: null, configParseError: null },
          undefined,
          'saveModels/start'
        )
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
            if (result.error.startsWith(CONFIG_PARSE_ERROR_PREFIX)) {
              set(
                { configParseError: result.error, isLoading: false },
                undefined,
                'saveModels/configParseError'
              )
            } else {
              set(
                { error: result.error, isLoading: false },
                undefined,
                'saveModels/error'
              )
            }
          }
        } catch (e) {
          set(
            { error: String(e), isLoading: false },
            undefined,
            'saveModels/exception'
          )
        }
      },

      resetConfigAndSave: async () => {
        const { models } = get()
        set(
          { isLoading: true, error: null, configParseError: null },
          undefined,
          'resetConfigAndSave/start'
        )
        try {
          const resetResult = await commands.resetConfigFile()
          if (resetResult.status !== 'ok') {
            set(
              { error: resetResult.error, isLoading: false },
              undefined,
              'resetConfigAndSave/resetError'
            )
            return
          }

          const saveResult = await commands.saveCustomModels(models)
          if (saveResult.status === 'ok') {
            set(
              {
                originalModels: JSON.parse(JSON.stringify(models)),
                hasChanges: false,
                isLoading: false,
              },
              undefined,
              'resetConfigAndSave/success'
            )
          } else {
            set(
              { error: saveResult.error, isLoading: false },
              undefined,
              'resetConfigAndSave/saveError'
            )
          }
        } catch (e) {
          set(
            { error: String(e), isLoading: false },
            undefined,
            'resetConfigAndSave/exception'
          )
        }
      },

      addModel: model => {
        set(
          state => {
            const baseDisplayName = model.displayName || model.model
            const uniqueDisplayName = generateUniqueDisplayName(
              baseDisplayName,
              state.models,
              model
            )
            const modelWithUniqueDisplayName = {
              ...model,
              displayName: uniqueDisplayName,
            }
            const newModels = [...state.models, modelWithUniqueDisplayName]
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

      deleteModels: indices => {
        set(
          state => {
            const indexSet = new Set(indices)
            const newModels = state.models.filter((_, i) => !indexSet.has(i))
            return {
              models: newModels,
              hasChanges: !modelsEqual(newModels, state.originalModels),
            }
          },
          undefined,
          'deleteModels'
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

      clearConfigParseError: () =>
        set({ configParseError: null }, undefined, 'clearConfigParseError'),

      loadDefaultModel: async () => {
        try {
          const result = await commands.getDefaultModel()
          if (result.status === 'ok') {
            set({ defaultModelId: result.data }, undefined, 'loadDefaultModel')
          }
        } catch {
          // Silently ignore errors when loading default model
        }
      },

      setDefaultModel: async index => {
        const { models } = get()
        const model = models[index]
        if (!model || !model.id) return

        const modelId = model.id

        try {
          const result = await commands.saveDefaultModel(modelId)
          if (result.status === 'ok') {
            set({ defaultModelId: modelId }, undefined, 'setDefaultModel')
          } else {
            set({ error: result.error }, undefined, 'setDefaultModel/error')
          }
        } catch (e) {
          set({ error: String(e) }, undefined, 'setDefaultModel/exception')
        }
      },
    }),
    { name: 'model-store' }
  )
)
