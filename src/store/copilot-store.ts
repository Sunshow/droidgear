import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import {
  commands,
  type CopilotProfile,
  type CopilotChannelSelection,
} from '@/lib/bindings'

interface CopilotState {
  profiles: CopilotProfile[]
  activeProfileId: string | null
  currentProfile: CopilotProfile | null
  isLoading: boolean
  error: string | null
  configStatus: { configExists: boolean; configPath: string } | null

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
  importFromChannel: (selection: CopilotChannelSelection) => Promise<void>
  setError: (error: string | null) => void
}

function cloneProfile(profile: CopilotProfile): CopilotProfile {
  return { ...profile }
}

async function performMutation(operation: () => Promise<void>): Promise<void> {
  useCopilotStore.setState({ error: null, isLoading: true })
  try {
    await operation()
  } catch (error) {
    useCopilotStore.setState({ error: String(error) })
  } finally {
    useCopilotStore.setState({ isLoading: false })
  }
}

export const useCopilotStore = create<CopilotState>()(
  devtools((set, get) => ({
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
        'copilot/loadProfiles/start'
      )
      try {
        const result = await commands.listCopilotProfiles()
        if (result.status !== 'ok') {
          set(
            { error: result.error, isLoading: false },
            undefined,
            'copilot/loadProfiles/error'
          )
          return
        }

        let profiles = result.data
        if (profiles.length === 0) {
          const created = await commands.createDefaultCopilotProfile()
          if (created.status !== 'ok') throw new Error(created.error)
          if (created.status === 'ok') {
            const refreshed = await commands.listCopilotProfiles()
            profiles =
              refreshed.status === 'ok' ? refreshed.data : [created.data]
          }
        }
        set(
          { profiles, isLoading: false },
          undefined,
          'copilot/loadProfiles/success'
        )
        if (!get().currentProfile && profiles[0])
          get().selectProfile(profiles[0].id)
      } catch (error) {
        set(
          { error: String(error), isLoading: false },
          undefined,
          'copilot/loadProfiles/exception'
        )
      }
    },

    loadActiveProfileId: async () => {
      try {
        const result = await commands.getActiveCopilotProfileId()
        if (result.status !== 'ok') return
        set(
          { activeProfileId: result.data },
          undefined,
          'copilot/loadActiveProfileId'
        )
        if (
          result.data &&
          get().profiles.some(profile => profile.id === result.data)
        ) {
          get().selectProfile(result.data)
          return
        }
        const first = get().profiles[0]
        if (first) get().selectProfile(first.id)
        else set({ currentProfile: null })
      } catch {
        // Loading the active marker is best effort; the profile list remains usable.
      }
    },

    loadConfigStatus: async () => {
      try {
        const result = await commands.getCopilotConfigStatus()
        if (result.status === 'ok') {
          set(
            { configStatus: result.data },
            undefined,
            'copilot/loadConfigStatus'
          )
        }
      } catch {
        // The status card is optional and must not block profile editing.
      }
    },

    selectProfile: id => {
      const profile = get().profiles.find(item => item.id === id)
      set(
        { currentProfile: profile ? cloneProfile(profile) : null },
        undefined,
        'copilot/selectProfile'
      )
    },

    createProfile: name =>
      performMutation(async () => {
        set({ error: null })
        const id = crypto.randomUUID()
        const now = new Date().toISOString()
        const result = await commands.saveCopilotProfile({
          id,
          name,
          description: null,
          createdAt: now,
          updatedAt: now,
          useOfficialAuth: false,
          baseUrl: null,
          providerType: 'openai',
          apiKey: null,
          model: null,
          maxPromptTokens: null,
          maxOutputTokens: null,
        })
        if (result.status !== 'ok') throw new Error(result.error)
        await get().loadProfiles()
        get().selectProfile(id)
      }),

    saveProfile: () =>
      performMutation(async () => {
        set({ error: null })
        const profile = get().currentProfile
        if (!profile) return
        const result = await commands.saveCopilotProfile(profile)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'copilot/saveProfile/error')
          return
        }
        await get().loadProfiles()
        get().selectProfile(profile.id)
      }),

    deleteProfile: id =>
      performMutation(async () => {
        set({ error: null })
        const result = await commands.deleteCopilotProfile(id)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'copilot/deleteProfile/error')
          return
        }
        await get().loadProfiles()
        await get().loadActiveProfileId()
        await get().loadConfigStatus()
      }),

    duplicateProfile: (id, newName) =>
      performMutation(async () => {
        set({ error: null })
        const result = await commands.duplicateCopilotProfile(id, newName)
        if (result.status !== 'ok') {
          set(
            { error: result.error },
            undefined,
            'copilot/duplicateProfile/error'
          )
          return
        }
        await get().loadProfiles()
        get().selectProfile(result.data.id)
      }),

    applyProfile: id =>
      performMutation(async () => {
        set({ error: null })
        const current = get().currentProfile
        if (current?.id === id) {
          const saveResult = await commands.saveCopilotProfile(current)
          if (saveResult.status !== 'ok') {
            set(
              { error: saveResult.error },
              undefined,
              'copilot/applyProfile/saveError'
            )
            return
          }
        }
        const result = await commands.applyCopilotProfile(id)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'copilot/applyProfile/error')
          return
        }
        set({ activeProfileId: id }, undefined, 'copilot/applyProfile/success')
        await get().loadProfiles()
        get().selectProfile(id)
        await get().loadConfigStatus()
      }),

    loadFromLiveConfig: () =>
      performMutation(async () => {
        set({ error: null })
        const current = get().currentProfile
        if (!current) return
        const result = await commands.readCopilotCurrentConfig()
        if (result.status !== 'ok') {
          set(
            { error: result.error },
            undefined,
            'copilot/loadFromLiveConfig/error'
          )
          return
        }
        const live = result.data
        set(
          {
            currentProfile: {
              ...current,
              useOfficialAuth: !live.isByok,
              baseUrl: live.baseUrl ?? null,
              providerType: live.providerType ?? null,
              apiKey: live.apiKey ?? null,
              model: live.model ?? null,
              maxPromptTokens: live.maxPromptTokens ?? null,
              maxOutputTokens: live.maxOutputTokens ?? null,
              updatedAt: new Date().toISOString(),
            },
          },
          undefined,
          'copilot/loadFromLiveConfig/success'
        )
        await get().saveProfile()
      }),

    importFromChannel: selection =>
      performMutation(async () => {
        const current = get().currentProfile
        if (!current) return
        const result = await commands.prepareCopilotChannelImport(
          current,
          selection
        )
        if (result.status !== 'ok') throw new Error(result.error)
        if (get().currentProfile?.id === current.id) {
          set(
            { currentProfile: result.data },
            undefined,
            'copilot/importFromChannel'
          )
        }
      }),

    setError: error => set({ error }, undefined, 'copilot/setError'),
  }))
)
