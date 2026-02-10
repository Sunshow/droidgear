import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import {
  commands,
  type CodexProfile,
  type CodexConfigStatus,
  type CodexCurrentConfig,
  type CodexProviderConfig,
} from '@/lib/bindings'

interface CodexState {
  profiles: CodexProfile[]
  activeProfileId: string | null
  currentProfile: CodexProfile | null
  isLoading: boolean
  error: string | null
  configStatus: CodexConfigStatus | null

  loadProfiles: () => Promise<void>
  loadActiveProfileId: () => Promise<void>
  loadConfigStatus: () => Promise<void>
  selectProfile: (id: string) => void
  createProfile: (name: string) => Promise<void>
  saveProfile: () => Promise<void>
  deleteProfile: (id: string) => Promise<void>
  duplicateProfile: (id: string, newName: string) => Promise<void>
  applyProfile: (id: string) => Promise<void>
  loadFromLiveConfig: () => Promise<void>
  updateProfileName: (name: string) => Promise<void>
  updateProfileDescription: (description: string) => Promise<void>

  // Provider management
  addProvider: (id: string, config: CodexProviderConfig) => Promise<void>
  updateProvider: (id: string, config: CodexProviderConfig) => Promise<void>
  deleteProvider: (id: string) => Promise<void>
  setActiveProvider: (providerId: string) => Promise<void>

  setError: (error: string | null) => void
}

export const useCodexStore = create<CodexState>()(
  devtools(
    (set, get) => ({
      profiles: [],
      activeProfileId: null,
      currentProfile: null,
      isLoading: false,
      error: null,
      configStatus: null,

      loadProfiles: async () => {
        set(
          { isLoading: true, error: null },
          undefined,
          'codex/loadProfiles/start'
        )
        try {
          const result = await commands.listCodexProfiles()
          if (result.status === 'ok') {
            let profiles = result.data
            if (profiles.length === 0) {
              const created = await commands.createDefaultCodexProfile()
              if (created.status === 'ok') {
                profiles = [created.data]
              }
            }
            set(
              { profiles, isLoading: false },
              undefined,
              'codex/loadProfiles/success'
            )
          } else {
            set(
              { error: result.error, isLoading: false },
              undefined,
              'codex/loadProfiles/error'
            )
          }
        } catch (e) {
          set(
            { error: String(e), isLoading: false },
            undefined,
            'codex/loadProfiles/exception'
          )
        }
      },

      loadActiveProfileId: async () => {
        try {
          const result = await commands.getActiveCodexProfileId()
          if (result.status === 'ok') {
            const activeId = result.data
            set(
              { activeProfileId: activeId },
              undefined,
              'codex/loadActiveProfileId'
            )
            // Auto-select active profile
            if (activeId) {
              get().selectProfile(activeId)
            } else {
              // Select first profile if no active
              const { profiles } = get()
              if (profiles.length > 0 && profiles[0]) {
                get().selectProfile(profiles[0].id)
              }
            }
          }
        } catch {
          // ignore
        }
      },

      loadConfigStatus: async () => {
        try {
          const result = await commands.getCodexConfigStatus()
          if (result.status === 'ok') {
            set(
              { configStatus: result.data },
              undefined,
              'codex/loadConfigStatus'
            )
          }
        } catch {
          // ignore
        }
      },

      selectProfile: id => {
        const profile = get().profiles.find(p => p.id === id) || null
        set(
          {
            currentProfile: profile
              ? JSON.parse(JSON.stringify(profile))
              : null,
          },
          undefined,
          'codex/selectProfile'
        )
      },

      createProfile: async name => {
        const now = new Date().toISOString()
        const profile: CodexProfile = {
          id: '',
          name,
          description: '',
          createdAt: now,
          updatedAt: now,
          providers: {},
          modelProvider: 'custom',
          model: '',
          modelReasoningEffort: null,
          apiKey: '',
        }
        const result = await commands.saveCodexProfile(profile)
        if (result.status !== 'ok') throw new Error(result.error)
        await get().loadProfiles()
      },

      saveProfile: async () => {
        const { currentProfile } = get()
        if (!currentProfile) return
        const result = await commands.saveCodexProfile(currentProfile)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'codex/saveProfile/error')
          return
        }
        await get().loadProfiles()
        get().selectProfile(currentProfile.id)
      },

      deleteProfile: async id => {
        const result = await commands.deleteCodexProfile(id)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'codex/deleteProfile/error')
          return
        }
        await get().loadProfiles()
        const next = get().profiles[0]?.id || null
        if (next) get().selectProfile(next)
      },

      duplicateProfile: async (id, newName) => {
        const result = await commands.duplicateCodexProfile(id, newName)
        if (result.status !== 'ok') {
          set(
            { error: result.error },
            undefined,
            'codex/duplicateProfile/error'
          )
          return
        }
        await get().loadProfiles()
        get().selectProfile(result.data.id)
      },

      applyProfile: async id => {
        // Ensure the current profile is saved to disk before applying
        const { currentProfile } = get()
        if (currentProfile && currentProfile.id === id) {
          const saveResult = await commands.saveCodexProfile(currentProfile)
          if (saveResult.status !== 'ok') {
            set(
              { error: saveResult.error },
              undefined,
              'codex/applyProfile/saveError'
            )
            return
          }
        }
        const result = await commands.applyCodexProfile(id)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'codex/applyProfile/error')
          return
        }
        set({ activeProfileId: id }, undefined, 'codex/applyProfile/success')
        await get().loadConfigStatus()
      },

      loadFromLiveConfig: async () => {
        const { currentProfile } = get()
        if (!currentProfile) return
        const result = await commands.readCodexCurrentConfig()
        if (result.status !== 'ok') {
          set(
            { error: result.error },
            undefined,
            'codex/loadFromLiveConfig/error'
          )
          return
        }
        const live: CodexCurrentConfig = result.data
        const updated: CodexProfile = {
          ...currentProfile,
          providers: (live.providers ?? {}) as Record<
            string,
            CodexProviderConfig
          >,
          modelProvider: live.modelProvider,
          model: live.model,
          modelReasoningEffort: live.modelReasoningEffort ?? null,
          apiKey: live.apiKey ?? null,
          updatedAt: new Date().toISOString(),
        }
        set(
          { currentProfile: updated },
          undefined,
          'codex/loadFromLiveConfig/success'
        )
        await get().saveProfile()
      },

      updateProfileName: async name => {
        const { currentProfile } = get()
        if (!currentProfile) return
        const updated = {
          ...currentProfile,
          name,
          updatedAt: new Date().toISOString(),
        }
        set({ currentProfile: updated }, undefined, 'codex/updateProfileName')
        await get().saveProfile()
      },

      updateProfileDescription: async description => {
        const { currentProfile } = get()
        if (!currentProfile) return
        const updated = {
          ...currentProfile,
          description: description || null,
          updatedAt: new Date().toISOString(),
        }
        set(
          { currentProfile: updated },
          undefined,
          'codex/updateProfileDescription'
        )
        await get().saveProfile()
      },

      // Provider management
      addProvider: async (id, config) => {
        const { currentProfile } = get()
        if (!currentProfile) return
        const providers = {
          ...((currentProfile.providers ?? {}) as Record<
            string,
            CodexProviderConfig
          >),
        }
        providers[id] = config
        const updated = {
          ...currentProfile,
          providers,
          updatedAt: new Date().toISOString(),
        }
        set({ currentProfile: updated }, undefined, 'codex/addProvider')
        await get().saveProfile()
      },

      updateProvider: async (id, config) => {
        const { currentProfile } = get()
        if (!currentProfile) return
        const providers = {
          ...((currentProfile.providers ?? {}) as Record<
            string,
            CodexProviderConfig
          >),
        }
        providers[id] = config
        const isActiveProvider = currentProfile.modelProvider === id
        const updated = {
          ...currentProfile,
          providers,
          ...(isActiveProvider && {
            model: config.model ?? currentProfile.model,
            modelReasoningEffort:
              config.modelReasoningEffort ??
              currentProfile.modelReasoningEffort,
            apiKey: config.apiKey ?? currentProfile.apiKey,
          }),
          updatedAt: new Date().toISOString(),
        }
        set({ currentProfile: updated }, undefined, 'codex/updateProvider')
        await get().saveProfile()
      },

      deleteProvider: async id => {
        const { currentProfile } = get()
        if (!currentProfile) return
        const oldProviders = (currentProfile.providers ?? {}) as Record<
          string,
          CodexProviderConfig
        >
        const providers = Object.fromEntries(
          Object.entries(oldProviders).filter(([key]) => key !== id)
        )
        const updated = {
          ...currentProfile,
          providers,
          updatedAt: new Date().toISOString(),
        }
        set({ currentProfile: updated }, undefined, 'codex/deleteProvider')
        await get().saveProfile()
      },

      setActiveProvider: async providerId => {
        const { currentProfile } = get()
        if (!currentProfile) return
        const providers = (currentProfile.providers ?? {}) as Record<
          string,
          CodexProviderConfig
        >
        const providerConfig = providers[providerId]
        const updated = {
          ...currentProfile,
          modelProvider: providerId,
          model: providerConfig?.model ?? currentProfile.model,
          modelReasoningEffort:
            providerConfig?.modelReasoningEffort ??
            currentProfile.modelReasoningEffort,
          apiKey: providerConfig?.apiKey ?? currentProfile.apiKey,
          updatedAt: new Date().toISOString(),
        }
        set({ currentProfile: updated }, undefined, 'codex/setActiveProvider')
        await get().saveProfile()
      },

      setError: error => set({ error }, undefined, 'codex/setError'),
    }),
    { name: 'codex-store' }
  )
)
