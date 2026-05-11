import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import {
  commands,
  type ClaudeCodeProfile,
  type ClaudeConfigStatus,
  type ClaudeCurrentConfig,
} from '@/lib/bindings'

function cloneClaudeProfile(profile: ClaudeCodeProfile): ClaudeCodeProfile {
  return JSON.parse(JSON.stringify(profile)) as ClaudeCodeProfile
}

function serializeClaudeProfile(profile: ClaudeCodeProfile): string {
  return JSON.stringify(profile)
}

interface ClaudeState {
  profiles: ClaudeCodeProfile[]
  activeProfileId: string | null
  currentProfile: ClaudeCodeProfile | null
  isLoading: boolean
  error: string | null
  configStatus: ClaudeConfigStatus | null

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
  setError: (error: string | null) => void
}

export const useClaudeStore = create<ClaudeState>()(
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
          'claude/loadProfiles/start'
        )
        try {
          const result = await commands.listClaudeProfiles()
          if (result.status === 'ok') {
            let profiles = result.data
            if (profiles.length === 0) {
              const created = await commands.createDefaultClaudeProfile()
              if (created.status !== 'ok') {
                set(
                  { error: created.error, isLoading: false },
                  undefined,
                  'claude/loadProfiles/bootstrapError'
                )
                return
              }

              const refreshed = await commands.listClaudeProfiles()
              profiles =
                refreshed.status === 'ok' ? refreshed.data : [created.data]
            }
            set(
              { profiles, isLoading: false },
              undefined,
              'claude/loadProfiles/success'
            )
          } else {
            set(
              { error: result.error, isLoading: false },
              undefined,
              'claude/loadProfiles/error'
            )
          }
        } catch (e) {
          set(
            { error: String(e), isLoading: false },
            undefined,
            'claude/loadProfiles/exception'
          )
        }
      },

      loadActiveProfileId: async () => {
        try {
          const result = await commands.getActiveClaudeProfileId()
          if (result.status === 'ok') {
            const activeId = result.data
            set(
              { activeProfileId: activeId },
              undefined,
              'claude/loadActiveProfileId'
            )
            if (activeId) {
              get().selectProfile(activeId)
            } else {
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
          const result = await commands.getClaudeConfigStatus()
          if (result.status === 'ok') {
            set(
              { configStatus: result.data },
              undefined,
              'claude/loadConfigStatus'
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
            currentProfile: profile ? cloneClaudeProfile(profile) : null,
          },
          undefined,
          'claude/selectProfile'
        )
      },

      createProfile: async name => {
        const now = new Date().toISOString()
        const profile: ClaudeCodeProfile = {
          id: '',
          name,
          description: null,
          baseUrl: null,
          bearerToken: null,
          model: null,
          smallModelUsesMainModel: true,
          smallModel: null,
          reasoningEffort: null,
          thinkingMode: 'inherit',
          createdAt: now,
          updatedAt: now,
        }
        const result = await commands.saveClaudeProfile(profile)
        if (result.status !== 'ok') throw new Error(result.error)
        await get().loadProfiles()
      },

      saveProfile: async () => {
        const { currentProfile } = get()
        if (!currentProfile) return
        const profileToSave = cloneClaudeProfile(currentProfile)
        const savedSnapshot = serializeClaudeProfile(profileToSave)
        const result = await commands.saveClaudeProfile(profileToSave)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'claude/saveProfile/error')
          return
        }
        await get().loadProfiles()

        const latestState = get()
        if (!latestState.currentProfile) return
        if (latestState.currentProfile.id !== profileToSave.id) return
        if (
          serializeClaudeProfile(latestState.currentProfile) !== savedSnapshot
        ) {
          return
        }

        const refreshedProfile = latestState.profiles.find(
          profile => profile.id === profileToSave.id
        )
        if (!refreshedProfile) return

        set(
          { currentProfile: cloneClaudeProfile(refreshedProfile) },
          undefined,
          'claude/saveProfile/refreshCurrentProfile'
        )
      },

      deleteProfile: async id => {
        const result = await commands.deleteClaudeProfile(id)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'claude/deleteProfile/error')
          return
        }
        await get().loadProfiles()
        const next = get().profiles[0]?.id || null
        if (next) get().selectProfile(next)
      },

      duplicateProfile: async (id, newName) => {
        const result = await commands.duplicateClaudeProfile(id, newName)
        if (result.status !== 'ok') {
          set(
            { error: result.error },
            undefined,
            'claude/duplicateProfile/error'
          )
          return
        }
        await get().loadProfiles()
        get().selectProfile(result.data.id)
      },

      applyProfile: async id => {
        const { currentProfile } = get()
        if (currentProfile && currentProfile.id === id) {
          const saveResult = await commands.saveClaudeProfile(currentProfile)
          if (saveResult.status !== 'ok') {
            set(
              { error: saveResult.error },
              undefined,
              'claude/applyProfile/saveError'
            )
            return
          }
        }
        const result = await commands.applyClaudeProfile(id)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'claude/applyProfile/error')
          return
        }
        set({ activeProfileId: id }, undefined, 'claude/applyProfile/success')
        await get().loadConfigStatus()
      },

      loadFromLiveConfig: async () => {
        const { currentProfile } = get()
        if (!currentProfile) return
        const result = await commands.readClaudeCurrentConfig()
        if (result.status !== 'ok') {
          set(
            { error: result.error },
            undefined,
            'claude/loadFromLiveConfig/error'
          )
          return
        }
        const live: ClaudeCurrentConfig = result.data
        const updated: ClaudeCodeProfile = {
          ...currentProfile,
          baseUrl: live.baseUrl ?? null,
          bearerToken: live.bearerToken ?? null,
          model: live.model ?? null,
          smallModelUsesMainModel: live.smallModelUsesMainModel ?? true,
          smallModel: live.smallModel ?? null,
          reasoningEffort: live.reasoningEffort ?? null,
          thinkingMode: live.thinkingMode ?? 'inherit',
          updatedAt: new Date().toISOString(),
        }
        set(
          { currentProfile: updated },
          undefined,
          'claude/loadFromLiveConfig/success'
        )
        await get().saveProfile()
      },

      setError: error => set({ error }, undefined, 'claude/setError'),
    }),
    { name: 'claude-store' }
  )
)
