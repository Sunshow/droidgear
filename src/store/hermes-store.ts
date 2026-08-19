import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import {
  commands,
  type HermesProfile,
  type HermesConfigStatus,
  type HermesCurrentConfig,
} from '@/lib/bindings'

interface HermesState {
  profiles: HermesProfile[]
  activeProfileId: string | null
  currentProfile: HermesProfile | null
  isLoading: boolean
  error: string | null
  configStatus: HermesConfigStatus | null

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
  importFromChannel: (params: {
    baseUrl: string
    apiKey: string
    provider: string
    defaultModel?: string
  }) => Promise<void>

  setError: (error: string | null) => void
}

export const useHermesStore = create<HermesState>()(
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
          'hermes/loadProfiles/start'
        )
        try {
          const result = await commands.listHermesProfiles()
          if (result.status === 'ok') {
            let profiles = result.data
            if (profiles.length === 0) {
              const created = await commands.createDefaultHermesProfile()
              if (created.status === 'ok') {
                const refreshed = await commands.listHermesProfiles()
                profiles =
                  refreshed.status === 'ok' ? refreshed.data : [created.data]
              }
            }
            set(
              { profiles, isLoading: false },
              undefined,
              'hermes/loadProfiles/success'
            )
          } else {
            set(
              { error: result.error, isLoading: false },
              undefined,
              'hermes/loadProfiles/error'
            )
          }
        } catch (e) {
          set(
            { error: String(e), isLoading: false },
            undefined,
            'hermes/loadProfiles/exception'
          )
        }
      },

      loadActiveProfileId: async () => {
        try {
          const result = await commands.getActiveHermesProfileId()
          if (result.status === 'ok') {
            const activeId = result.data
            set(
              { activeProfileId: activeId },
              undefined,
              'hermes/loadActiveProfileId'
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
          const result = await commands.getHermesConfigStatus()
          if (result.status === 'ok') {
            set(
              { configStatus: result.data },
              undefined,
              'hermes/loadConfigStatus'
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
          'hermes/selectProfile'
        )
      },

      createProfile: async name => {
        const now = new Date().toISOString()
        const profile: HermesProfile = {
          id: '',
          name,
          description: null,
          createdAt: now,
          updatedAt: now,
          model: {
            default: null,
            provider: null,
            baseUrl: null,
            apiKey: null,
          },
        }
        const result = await commands.saveHermesProfile(profile)
        if (result.status !== 'ok') throw new Error(result.error)
        await get().loadProfiles()
      },

      saveProfile: async () => {
        const { currentProfile } = get()
        if (!currentProfile) return
        const result = await commands.saveHermesProfile(currentProfile)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'hermes/saveProfile/error')
          return
        }
        await get().loadProfiles()
        get().selectProfile(currentProfile.id)
      },

      deleteProfile: async id => {
        const result = await commands.deleteHermesProfile(id)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'hermes/deleteProfile/error')
          return
        }
        await get().loadProfiles()
        const next = get().profiles[0]?.id || null
        if (next) get().selectProfile(next)
      },

      duplicateProfile: async (id, newName) => {
        const result = await commands.duplicateHermesProfile(id, newName)
        if (result.status !== 'ok') {
          set(
            { error: result.error },
            undefined,
            'hermes/duplicateProfile/error'
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
          const saveResult = await commands.saveHermesProfile(currentProfile)
          if (saveResult.status !== 'ok') {
            set(
              { error: saveResult.error },
              undefined,
              'hermes/applyProfile/saveError'
            )
            return
          }
        }
        const result = await commands.applyHermesProfile(id)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'hermes/applyProfile/error')
          return
        }
        set({ activeProfileId: id }, undefined, 'hermes/applyProfile/success')
        await get().loadConfigStatus()
      },

      loadFromLiveConfig: async () => {
        const { currentProfile } = get()
        if (!currentProfile) return
        const result = await commands.readHermesCurrentConfig()
        if (result.status !== 'ok') {
          set(
            { error: result.error },
            undefined,
            'hermes/loadFromLiveConfig/error'
          )
          return
        }
        const live: HermesCurrentConfig = result.data
        const updated: HermesProfile = {
          ...currentProfile,
          model: live.model,
          reasoningEffort:
            live.reasoningEffort ?? currentProfile.reasoningEffort,
          updatedAt: new Date().toISOString(),
        }
        set(
          { currentProfile: updated },
          undefined,
          'hermes/loadFromLiveConfig/success'
        )
        await get().saveProfile()
      },

      importFromChannel: async ({
        baseUrl,
        apiKey,
        provider,
        defaultModel,
      }) => {
        const { currentProfile } = get()
        if (!currentProfile) return
        const updated: HermesProfile = {
          ...currentProfile,
          model: {
            default: defaultModel ?? null,
            provider: provider || null,
            baseUrl: baseUrl || null,
            apiKey: apiKey || null,
          },
          updatedAt: new Date().toISOString(),
        }
        set({ currentProfile: updated }, undefined, 'hermes/importFromChannel')
        await get().saveProfile()
      },

      setError: error => set({ error }, undefined, 'hermes/setError'),
    }),
    { name: 'hermes-store' }
  )
)
