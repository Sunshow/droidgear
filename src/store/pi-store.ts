import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import {
  commands,
  type PiProfile,
  type PiProviderConfig,
  type PiConfigStatus,
  type PiCurrentConfig,
} from '@/lib/bindings'

interface PiState {
  profiles: PiProfile[]
  activeProfileId: string | null
  currentProfile: PiProfile | null
  isLoading: boolean
  error: string | null
  configStatus: PiConfigStatus | null

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
  addProvider: (id: string, config: PiProviderConfig) => Promise<void>
  updateProvider: (id: string, config: PiProviderConfig) => Promise<void>
  deleteProvider: (id: string) => Promise<void>
  setError: (error: string | null) => void
}

export const usePiStore = create<PiState>()(
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
          'pi/loadProfiles/start'
        )
        try {
          const result = await commands.listPiProfiles()
          if (result.status === 'ok') {
            let profiles = result.data
            if (profiles.length === 0) {
              const created = await commands.createDefaultPiProfile()
              if (created.status === 'ok') {
                const refreshed = await commands.listPiProfiles()
                profiles =
                  refreshed.status === 'ok' ? refreshed.data : [created.data]
              }
            }
            set(
              { profiles, isLoading: false },
              undefined,
              'pi/loadProfiles/success'
            )
          } else {
            set(
              { error: result.error, isLoading: false },
              undefined,
              'pi/loadProfiles/error'
            )
          }
        } catch (e) {
          set(
            { error: String(e), isLoading: false },
            undefined,
            'pi/loadProfiles/exception'
          )
        }
      },

      loadActiveProfileId: async () => {
        try {
          const result = await commands.getActivePiProfileId()
          if (result.status === 'ok') {
            const activeId = result.data
            set(
              { activeProfileId: activeId },
              undefined,
              'pi/loadActiveProfileId'
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
          const result = await commands.getPiConfigStatus()
          if (result.status === 'ok') {
            set({ configStatus: result.data }, undefined, 'pi/loadConfigStatus')
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
          'pi/selectProfile'
        )
      },

      createProfile: async name => {
        const now = new Date().toISOString()
        const profile: PiProfile = {
          id: '',
          name,
          description: null,
          createdAt: now,
          updatedAt: now,
          providers: {},
        }
        const result = await commands.savePiProfile(profile)
        if (result.status !== 'ok') throw new Error(result.error)
        await get().loadProfiles()
      },

      saveProfile: async () => {
        const { currentProfile } = get()
        if (!currentProfile) return
        const result = await commands.savePiProfile(currentProfile)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'pi/saveProfile/error')
          return
        }
        await get().loadProfiles()
        get().selectProfile(currentProfile.id)
      },

      deleteProfile: async id => {
        const result = await commands.deletePiProfile(id)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'pi/deleteProfile/error')
          return
        }
        await get().loadProfiles()
        const next = get().profiles[0]?.id || null
        if (next) get().selectProfile(next)
      },

      duplicateProfile: async (id, newName) => {
        const result = await commands.duplicatePiProfile(id, newName)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'pi/duplicateProfile/error')
          return
        }
        await get().loadProfiles()
        get().selectProfile(result.data.id)
      },

      applyProfile: async id => {
        // Ensure the current profile is saved to disk before applying
        const { currentProfile } = get()
        if (currentProfile && currentProfile.id === id) {
          const saveResult = await commands.savePiProfile(currentProfile)
          if (saveResult.status !== 'ok') {
            set(
              { error: saveResult.error },
              undefined,
              'pi/applyProfile/saveError'
            )
            return
          }
        }
        const result = await commands.applyPiProfile(id)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'pi/applyProfile/error')
          return
        }
        set({ activeProfileId: id }, undefined, 'pi/applyProfile/success')
        await get().loadConfigStatus()
      },

      loadFromLiveConfig: async () => {
        const { currentProfile } = get()
        if (!currentProfile) return
        const result = await commands.readPiCurrentConfig()
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'pi/loadFromLiveConfig/error')
          return
        }
        const live: PiCurrentConfig = result.data
        const updated: PiProfile = {
          ...currentProfile,
          providers: live.providers || {},
          updatedAt: new Date().toISOString(),
        }
        set(
          { currentProfile: updated },
          undefined,
          'pi/loadFromLiveConfig/success'
        )
        await get().saveProfile()
      },

      addProvider: async (id, config) => {
        const { currentProfile } = get()
        if (!currentProfile) return
        const updated: PiProfile = {
          ...currentProfile,
          providers: { ...(currentProfile.providers ?? {}), [id]: config },
          updatedAt: new Date().toISOString(),
        }
        set({ currentProfile: updated }, undefined, 'pi/addProvider')
        await get().saveProfile()
      },

      updateProvider: async (id, config) => {
        const { currentProfile } = get()
        if (!currentProfile) return
        const updated: PiProfile = {
          ...currentProfile,
          providers: { ...(currentProfile.providers ?? {}), [id]: config },
          updatedAt: new Date().toISOString(),
        }
        set({ currentProfile: updated }, undefined, 'pi/updateProvider')
        await get().saveProfile()
      },

      deleteProvider: async id => {
        const { currentProfile } = get()
        if (!currentProfile) return
        const { [id]: _removed, ...providers } = currentProfile.providers ?? {}
        const updated: PiProfile = {
          ...currentProfile,
          providers,
          updatedAt: new Date().toISOString(),
        }
        set({ currentProfile: updated }, undefined, 'pi/deleteProvider')
        await get().saveProfile()
      },

      setError: error => set({ error }, undefined, 'pi/setError'),
    }),
    { name: 'pi-store' }
  )
)
