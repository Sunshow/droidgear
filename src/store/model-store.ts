import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import {
  commands,
  type CustomModel,
  type SessionDefaultSettings,
} from '@/lib/bindings'

const CONFIG_PARSE_ERROR_PREFIX = 'CONFIG_PARSE_ERROR:'

interface ModelState {
  models: CustomModel[]
  originalModels: CustomModel[]
  modelFavorites: string[]
  configPath: string
  hasChanges: boolean
  isLoading: boolean
  error: string | null
  configParseError: string | null
  defaultModelId: string | null
  sessionDefaultSettings: SessionDefaultSettings | null

  // Actions
  loadModels: () => Promise<void>
  loadModelFavorites: () => Promise<void>
  saveModels: () => Promise<void>
  saveModelFavorites: (favorites: string[]) => Promise<void>
  toggleModelFavorite: (modelId: string) => Promise<void>
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
  saveSessionDefaultSettings: (
    settings: SessionDefaultSettings
  ) => Promise<void>
}

function modelsEqual(a: CustomModel[], b: CustomModel[]): boolean {
  if (a.length !== b.length) return false
  return JSON.stringify(a) === JSON.stringify(b)
}

function normalizeFavorites(favorites: string[]): string[] {
  return Array.from(
    new Set(
      favorites
        .map(favorite => favorite.trim())
        .filter(favorite => favorite.length > 0)
    )
  )
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

function generateModelId(displayName: string, index: number): string {
  return `custom:${displayName}-${index}`
}

function updateModelsIndexAndId(models: CustomModel[]): CustomModel[] {
  return models.map((m, idx) => ({
    ...m,
    index: idx,
    id: generateModelId(m.displayName || m.model, idx),
  }))
}

export const useModelStore = create<ModelState>()(
  devtools(
    (set, get) => ({
      models: [],
      originalModels: [],
      modelFavorites: [],
      configPath: '~/.factory/config.json',
      hasChanges: false,
      isLoading: false,
      error: null,
      configParseError: null,
      defaultModelId: null,
      sessionDefaultSettings: null,

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
            const modelsWithIds = updateModelsIndexAndId(modelsResult.data)
            set(
              {
                models: modelsWithIds,
                originalModels: JSON.parse(JSON.stringify(modelsWithIds)),
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

      loadModelFavorites: async () => {
        try {
          const result = await commands.getModelFavorites()
          if (result.status === 'ok') {
            set(
              { modelFavorites: normalizeFavorites(result.data) },
              undefined,
              'loadModelFavorites/success'
            )
          } else {
            set({ error: result.error }, undefined, 'loadModelFavorites/error')
          }
        } catch (e) {
          set({ error: String(e) }, undefined, 'loadModelFavorites/exception')
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
            await get().loadModelFavorites()
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

      saveModelFavorites: async favorites => {
        const previousFavorites = get().modelFavorites
        const normalizedFavorites = normalizeFavorites(favorites)
        set(
          { modelFavorites: normalizedFavorites, error: null },
          undefined,
          'saveModelFavorites/start'
        )
        try {
          const result = await commands.saveModelFavorites(normalizedFavorites)
          if (result.status === 'ok') {
            await get().loadModelFavorites()
          } else {
            set(
              { modelFavorites: previousFavorites, error: result.error },
              undefined,
              'saveModelFavorites/error'
            )
          }
        } catch (e) {
          set(
            { modelFavorites: previousFavorites, error: String(e) },
            undefined,
            'saveModelFavorites/exception'
          )
        }
      },

      toggleModelFavorite: async modelId => {
        const favorites = new Set(get().modelFavorites)
        if (favorites.has(modelId)) {
          favorites.delete(modelId)
        } else {
          favorites.add(modelId)
        }
        await get().saveModelFavorites(Array.from(favorites))
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
            await get().loadModelFavorites()
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
            const newIndex = state.models.length
            const modelWithIdAndIndex = {
              ...model,
              displayName: uniqueDisplayName,
              id: generateModelId(uniqueDisplayName, newIndex),
              index: newIndex,
            }
            const newModels = [...state.models, modelWithIdAndIndex]
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
            const oldModel = state.models[index]
            const oldId = oldModel?.id
            const newModels = [...state.models]
            newModels[index] = model
            const updatedModels = updateModelsIndexAndId(newModels)
            const newModel = updatedModels[index]
            const newId = newModel?.id

            // Update default model references if the model's ID changed
            // (e.g., display name was edited, which changes the derived ID)
            let defaultModelId = state.defaultModelId
            let sessionDefaultSettings = state.sessionDefaultSettings
              ? { ...state.sessionDefaultSettings }
              : null

            if (oldId && newId && oldId !== newId) {
              if (defaultModelId === oldId) {
                defaultModelId = newId
              }
              if (sessionDefaultSettings?.model === oldId) {
                sessionDefaultSettings = {
                  ...sessionDefaultSettings,
                  model: newId,
                }
              }
              if (sessionDefaultSettings?.specModeModel === oldId) {
                sessionDefaultSettings = {
                  ...sessionDefaultSettings,
                  specModeModel: newId,
                }
              }
            }

            return {
              models: updatedModels,
              hasChanges: !modelsEqual(updatedModels, state.originalModels),
              defaultModelId,
              sessionDefaultSettings,
            }
          },
          undefined,
          'updateModel'
        )
      },

      deleteModel: index => {
        set(
          state => {
            const filteredModels = state.models.filter((_, i) => i !== index)
            const newModels = updateModelsIndexAndId(filteredModels)
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
            const filteredModels = state.models.filter(
              (_, i) => !indexSet.has(i)
            )
            const newModels = updateModelsIndexAndId(filteredModels)
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
            const reorderedModels = [...state.models]
            const removed = reorderedModels.splice(fromIndex, 1)[0]
            if (removed) {
              reorderedModels.splice(toIndex, 0, removed)
            }
            const newModels = updateModelsIndexAndId(reorderedModels)
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
          const result = await commands.getSessionDefaultSettings()
          if (result.status === 'ok') {
            set(
              {
                defaultModelId: result.data.model ?? null,
                sessionDefaultSettings: result.data,
              },
              undefined,
              'loadDefaultModel'
            )
          }
        } catch {
          // Silently ignore errors when loading default model
        }
      },

      saveSessionDefaultSettings: async settings => {
        try {
          const result = await commands.saveSessionDefaultSettings(settings)
          if (result.status === 'ok') {
            set(
              {
                defaultModelId: settings.model ?? null,
                sessionDefaultSettings: settings,
              },
              undefined,
              'saveSessionDefaultSettings'
            )
          } else {
            set(
              { error: result.error },
              undefined,
              'saveSessionDefaultSettings/error'
            )
          }
        } catch (e) {
          set(
            { error: String(e) },
            undefined,
            'saveSessionDefaultSettings/exception'
          )
        }
      },
    }),
    { name: 'model-store' }
  )
)
