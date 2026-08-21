import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import {
  commands,
  type OmpProfile,
  type OmpConfigStatus,
  type OmpCurrentConfig,
  type OmpProviderTestResult,
} from '@/lib/bindings'

interface OmpState {
  profiles: OmpProfile[]
  activeProfileId: string | null
  currentProfile: OmpProfile | null
  isLoading: boolean
  error: string | null
  configStatus: OmpConfigStatus | null
  liveConfig: OmpCurrentConfig | null

  loadProfiles: () => Promise<void>
  loadActiveProfileId: () => Promise<void>
  loadConfigStatus: () => Promise<void>
  loadLiveConfig: () => Promise<void>
  selectProfile: (id: string) => void
  createProfile: (name: string) => Promise<void>
  saveProfile: () => Promise<void>
  deleteProfile: (id: string) => Promise<void>
  duplicateProfile: (id: string, newName: string) => Promise<void>
  applyProfile: (id: string) => Promise<void>
  testProvider: (providerId: string) => Promise<OmpProviderTestResult | null>
  setError: (error: string | null) => void
}

export const useOmpStore = create<OmpState>()(
  devtools(
    (set, get) => ({
      profiles: [],
      activeProfileId: null,
      currentProfile: null,
      isLoading: false,
      error: null,
      configStatus: null,
      liveConfig: null,

      loadProfiles: async () => {
        set({ isLoading: true }, undefined, 'omp/loadProfiles/start')
        try {
          const result = await commands.listOmpProfiles()
          if (result.status === 'ok') {
            set(
              { profiles: result.data, isLoading: false },
              undefined,
              'omp/loadProfiles/success'
            )
          } else {
            set(
              { error: result.error, isLoading: false },
              undefined,
              'omp/loadProfiles/error'
            )
          }
        } catch (e) {
          set(
            { error: String(e), isLoading: false },
            undefined,
            'omp/loadProfiles/exception'
          )
        }
      },

      loadActiveProfileId: async () => {
        try {
          const result = await commands.getActiveOmpProfileId()
          if (result.status === 'ok') {
            const activeId = result.data
            set(
              { activeProfileId: activeId },
              undefined,
              'omp/loadActiveProfileId'
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
          const result = await commands.getOmpConfigStatus()
          if (result.status === 'ok') {
            set(
              { configStatus: result.data },
              undefined,
              'omp/loadConfigStatus'
            )
          }
        } catch {
          // ignore
        }
      },

      loadLiveConfig: async () => {
        try {
          const result = await commands.readOmpCurrentConfig()
          if (result.status === 'ok') {
            set({ liveConfig: result.data }, undefined, 'omp/loadLiveConfig')
          }
        } catch {
          // ignore
        }
      },

      selectProfile: id => {
        const profile = get().profiles.find(p => p.id === id) ?? null
        set(
          { currentProfile: profile, activeProfileId: id },
          undefined,
          'omp/selectProfile'
        )
      },

      createProfile: async name => {
        const profile: OmpProfile = {
          id: '',
          name,
          createdAt: '',
          updatedAt: '',
          modelRoles: {},
        }
        const result = await commands.saveOmpProfile(profile)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'omp/createProfile/error')
          return
        }
        await get().loadProfiles()
      },

      saveProfile: async () => {
        const profile = get().currentProfile
        if (!profile) return

        const result = await commands.saveOmpProfile(profile)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'omp/saveProfile/error')
          return
        }
        await get().loadProfiles()
      },

      deleteProfile: async id => {
        const result = await commands.deleteOmpProfile(id)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'omp/deleteProfile/error')
          return
        }
        await get().loadProfiles()
      },

      duplicateProfile: async (id, newName) => {
        const result = await commands.duplicateOmpProfile(id, newName)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'omp/duplicateProfile/error')
          return
        }
        await get().loadProfiles()
      },

      applyProfile: async id => {
        const result = await commands.applyOmpProfile(id)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'omp/applyProfile/error')
          return
        }
        set({ activeProfileId: id }, undefined, 'omp/applyProfile/success')
        await get().loadConfigStatus()
        await get().loadLiveConfig()
      },

      testProvider: async providerId => {
        try {
          const result = await commands.testOmpProviderConnection(providerId)
          if (result.status === 'ok') {
            return result.data
          }
          set({ error: result.error }, undefined, 'omp/testProvider/error')
          return null
        } catch (e) {
          set(
            { error: String(e), isLoading: false },
            undefined,
            'omp/testProvider/exception'
          )
          return null
        }
      },

      setError: error => set({ error }, undefined, 'omp/setError'),
    }),
    { name: 'omp-store' }
  )
)
