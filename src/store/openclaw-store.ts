import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import {
  commands,
  type OpenClawProfile,
  type OpenClawProviderConfig,
  type OpenClawConfigStatus,
} from '@/lib/bindings'

interface OpenClawState {
  profiles: OpenClawProfile[]
  activeProfileId: string | null
  currentProfile: OpenClawProfile | null
  isLoading: boolean
  error: string | null
  configStatus: OpenClawConfigStatus | null

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
  updateDefaultModel: (model: string) => Promise<void>
  addProvider: (id: string, config: OpenClawProviderConfig) => Promise<void>
  updateProvider: (id: string, config: OpenClawProviderConfig) => Promise<void>
  deleteProvider: (id: string) => Promise<void>
  setError: (error: string | null) => void
}

export const useOpenClawStore = create<OpenClawState>()(
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
          'openclaw/loadProfiles/start'
        )
        try {
          const result = await commands.listOpenclawProfiles()
          if (result.status === 'ok') {
            let profiles = result.data
            if (profiles.length === 0) {
              const created = await commands.createDefaultOpenclawProfile()
              if (created.status === 'ok') {
                profiles = [created.data]
              }
            }
            set(
              { profiles, isLoading: false },
              undefined,
              'openclaw/loadProfiles/success'
            )
          } else {
            set(
              { error: result.error, isLoading: false },
              undefined,
              'openclaw/loadProfiles/error'
            )
          }
        } catch (e) {
          set(
            { error: String(e), isLoading: false },
            undefined,
            'openclaw/loadProfiles/exception'
          )
        }
      },

      loadActiveProfileId: async () => {
        try {
          const result = await commands.getActiveOpenclawProfileId()
          if (result.status === 'ok') {
            set(
              { activeProfileId: result.data },
              undefined,
              'openclaw/loadActiveProfileId'
            )
            if (result.data) get().selectProfile(result.data)
          }
        } catch {
          // ignore
        }
      },

      loadConfigStatus: async () => {
        try {
          const result = await commands.getOpenclawConfigStatus()
          if (result.status === 'ok') {
            set(
              { configStatus: result.data },
              undefined,
              'openclaw/loadConfigStatus'
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
          'openclaw/selectProfile'
        )
      },

      createProfile: async name => {
        const now = new Date().toISOString()
        const profile: OpenClawProfile = {
          id: '',
          name,
          description: null,
          createdAt: now,
          updatedAt: now,
          defaultModel: null,
          providers: {},
        }
        const result = await commands.saveOpenclawProfile(profile)
        if (result.status !== 'ok') throw new Error(result.error)
        await get().loadProfiles()
      },

      saveProfile: async () => {
        const { currentProfile } = get()
        if (!currentProfile) return
        const result = await commands.saveOpenclawProfile(currentProfile)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'openclaw/saveProfile/error')
          return
        }
        await get().loadProfiles()
        get().selectProfile(currentProfile.id)
      },

      deleteProfile: async id => {
        const result = await commands.deleteOpenclawProfile(id)
        if (result.status !== 'ok') {
          set(
            { error: result.error },
            undefined,
            'openclaw/deleteProfile/error'
          )
          return
        }
        await get().loadProfiles()
        const next = get().profiles[0]?.id || null
        if (next) get().selectProfile(next)
      },

      duplicateProfile: async (id, newName) => {
        const result = await commands.duplicateOpenclawProfile(id, newName)
        if (result.status !== 'ok') {
          set(
            { error: result.error },
            undefined,
            'openclaw/duplicateProfile/error'
          )
          return
        }
        await get().loadProfiles()
        get().selectProfile(result.data.id)
      },

      applyProfile: async id => {
        const result = await commands.applyOpenclawProfile(id)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'openclaw/applyProfile/error')
          return
        }
        set({ activeProfileId: id }, undefined, 'openclaw/applyProfile/success')
        await get().loadConfigStatus()
      },

      loadFromLiveConfig: async () => {
        const { currentProfile } = get()
        if (!currentProfile) return
        const result = await commands.readOpenclawCurrentConfig()
        if (result.status !== 'ok') {
          set(
            { error: result.error },
            undefined,
            'openclaw/loadFromLiveConfig/error'
          )
          return
        }
        const live = result.data
        const updated: OpenClawProfile = {
          ...currentProfile,
          defaultModel: live.defaultModel ?? currentProfile.defaultModel,
          providers: live.providers as Record<string, OpenClawProviderConfig>,
          updatedAt: new Date().toISOString(),
        }
        set(
          { currentProfile: updated },
          undefined,
          'openclaw/loadFromLiveConfig/success'
        )
        // Auto-save after loading from live config
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
        set(
          { currentProfile: updated },
          undefined,
          'openclaw/updateProfileName'
        )
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
          'openclaw/updateProfileDescription'
        )
        await get().saveProfile()
      },

      updateDefaultModel: async model => {
        const { currentProfile } = get()
        if (!currentProfile) return
        const updated = {
          ...currentProfile,
          defaultModel: model || null,
          updatedAt: new Date().toISOString(),
        }
        set(
          { currentProfile: updated },
          undefined,
          'openclaw/updateDefaultModel'
        )
        await get().saveProfile()
      },

      addProvider: async (id, config) => {
        const { currentProfile } = get()
        if (!currentProfile) return
        const updated = {
          ...currentProfile,
          providers: { ...currentProfile.providers, [id]: config },
          updatedAt: new Date().toISOString(),
        }
        set({ currentProfile: updated }, undefined, 'openclaw/addProvider')
        await get().saveProfile()
      },

      updateProvider: async (id, config) => {
        const { currentProfile } = get()
        if (!currentProfile) return
        const updated = {
          ...currentProfile,
          providers: { ...currentProfile.providers, [id]: config },
          updatedAt: new Date().toISOString(),
        }
        set({ currentProfile: updated }, undefined, 'openclaw/updateProvider')
        await get().saveProfile()
      },

      deleteProvider: async id => {
        const { currentProfile } = get()
        if (!currentProfile) return
        const { [id]: _removed, ...providers } = currentProfile.providers
        const updated = {
          ...currentProfile,
          providers,
          updatedAt: new Date().toISOString(),
        }
        set({ currentProfile: updated }, undefined, 'openclaw/deleteProvider')
        await get().saveProfile()
      },

      setError: error => set({ error }, undefined, 'openclaw/setError'),
    }),
    { name: 'openclaw-store' }
  )
)
