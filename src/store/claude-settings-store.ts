import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import {
  commands,
  type ClaudeSettingsFileInfo,
  type JsonValue,
} from '@/lib/bindings'

export type ClaudeSettingsDoc = Record<string, JsonValue>

interface ClaudeSettingsState {
  files: ClaudeSettingsFileInfo[]
  activeFile: ClaudeSettingsFileInfo | null
  currentJson: ClaudeSettingsDoc | null
  hasChanges: boolean
  isLoading: boolean
  isLaunching: boolean
  error: string | null

  loadFiles: () => Promise<void>
  selectFile: (file: ClaudeSettingsFileInfo) => Promise<void>
  createFile: (name: string, copyFromActive: boolean) => Promise<void>
  deleteFile: (name: string) => Promise<void>
  patchJson: (mutator: (draft: ClaudeSettingsDoc) => void) => void
  setJson: (next: ClaudeSettingsDoc) => void
  saveFile: () => Promise<void>
  resetChanges: () => Promise<void>
  launch: (cwd: string, skipDangerous: boolean) => Promise<boolean>
  setError: (message: string | null) => void
}

function cloneJson<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T
}

function ensureObject(value: JsonValue | undefined): ClaudeSettingsDoc {
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    return value as ClaudeSettingsDoc
  }
  return {}
}

export const useClaudeSettingsStore = create<ClaudeSettingsState>()(
  devtools(
    (set, get) => ({
      files: [],
      activeFile: null,
      currentJson: null,
      hasChanges: false,
      isLoading: false,
      isLaunching: false,
      error: null,

      loadFiles: async () => {
        set(
          { isLoading: true, error: null },
          undefined,
          'claudeSettings/loadFiles/start'
        )
        try {
          const filesResult = await commands.listClaudeSettingsFiles()
          if (filesResult.status !== 'ok') {
            set(
              { error: filesResult.error, isLoading: false },
              undefined,
              'claudeSettings/loadFiles/listError'
            )
            return
          }
          const files = filesResult.data
          const active = files.find(f => f.isActive) ?? files[0] ?? null
          let currentJson: ClaudeSettingsDoc | null = null
          if (active) {
            const readResult = await commands.readClaudeSettingsFile(
              active.name
            )
            if (readResult.status === 'ok') {
              currentJson = ensureObject(readResult.data)
            } else {
              set(
                { error: readResult.error },
                undefined,
                'claudeSettings/loadFiles/readError'
              )
            }
          }
          set(
            {
              files,
              activeFile: active,
              currentJson,
              hasChanges: false,
              isLoading: false,
            },
            undefined,
            'claudeSettings/loadFiles/success'
          )
        } catch (err) {
          set(
            { error: String(err), isLoading: false },
            undefined,
            'claudeSettings/loadFiles/exception'
          )
        }
      },

      selectFile: async file => {
        if (file.isActive && get().currentJson) return
        set(
          { isLoading: true, error: null },
          undefined,
          'claudeSettings/selectFile/start'
        )
        const setActive = await commands.setActiveClaudeSettingsFile(
          file.isGlobal ? null : file.name
        )
        if (setActive.status !== 'ok') {
          set(
            { error: setActive.error, isLoading: false },
            undefined,
            'claudeSettings/selectFile/setActiveError'
          )
          return
        }
        await get().loadFiles()
      },

      createFile: async (name, copyFromActive) => {
        const result = await commands.createClaudeSettingsFile(
          name,
          copyFromActive
        )
        if (result.status !== 'ok') {
          set(
            { error: result.error },
            undefined,
            'claudeSettings/createFile/error'
          )
          throw new Error(result.error)
        }
        await get().loadFiles()
      },

      deleteFile: async name => {
        const result = await commands.deleteClaudeSettingsFile(name)
        if (result.status !== 'ok') {
          set(
            { error: result.error },
            undefined,
            'claudeSettings/deleteFile/error'
          )
          throw new Error(result.error)
        }
        await get().loadFiles()
      },

      patchJson: mutator => {
        const current = get().currentJson
        if (!current) return
        const next = cloneJson(current)
        mutator(next)
        set(
          { currentJson: next, hasChanges: true },
          undefined,
          'claudeSettings/patchJson'
        )
      },

      setJson: next => {
        set(
          { currentJson: cloneJson(next), hasChanges: true },
          undefined,
          'claudeSettings/setJson'
        )
      },

      saveFile: async () => {
        const { activeFile, currentJson } = get()
        if (!activeFile || !currentJson) return
        const result = await commands.saveClaudeSettingsFile(
          activeFile.name,
          currentJson as JsonValue
        )
        if (result.status !== 'ok') {
          set(
            { error: result.error },
            undefined,
            'claudeSettings/saveFile/error'
          )
          return
        }
        set({ hasChanges: false }, undefined, 'claudeSettings/saveFile/success')
      },

      resetChanges: async () => {
        const { activeFile } = get()
        if (!activeFile) return
        const readResult = await commands.readClaudeSettingsFile(
          activeFile.name
        )
        if (readResult.status === 'ok') {
          set(
            { currentJson: ensureObject(readResult.data), hasChanges: false },
            undefined,
            'claudeSettings/resetChanges'
          )
        }
      },

      launch: async (cwd, skipDangerous) => {
        set(
          { isLaunching: true, error: null },
          undefined,
          'claudeSettings/launch/start'
        )
        try {
          const { hasChanges } = get()
          if (hasChanges) {
            await get().saveFile()
            if (get().error) {
              return false
            }
          }
          const result = await commands.launchClaudeWithSettings(
            cwd,
            skipDangerous
          )
          if (result.status !== 'ok') {
            set(
              { error: result.error },
              undefined,
              'claudeSettings/launch/error'
            )
            return false
          }
          return true
        } finally {
          set(
            { isLaunching: false },
            undefined,
            'claudeSettings/launch/finally'
          )
        }
      },

      setError: message =>
        set({ error: message }, undefined, 'claudeSettings/setError'),
    }),
    { name: 'claude-settings-store' }
  )
)
