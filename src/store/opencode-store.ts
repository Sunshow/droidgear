import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import {
  commands,
  type OpenCodeProfile,
  type OpenCodeProviderConfig,
  type OpenCodeConfigStatus,
  type ProviderTemplate,
  type JsonValue,
} from '@/lib/bindings'

interface OpenCodeState {
  profiles: OpenCodeProfile[]
  activeProfileId: string | null
  currentProfile: OpenCodeProfile | null
  originalProfile: OpenCodeProfile | null
  hasChanges: boolean
  isLoading: boolean
  error: string | null
  configStatus: OpenCodeConfigStatus | null
  providerTemplates: ProviderTemplate[]

  loadProfiles: () => Promise<void>
  loadActiveProfileId: () => Promise<void>
  loadConfigStatus: () => Promise<void>
  loadProviderTemplates: () => Promise<void>
  selectProfile: (id: string) => void
  createProfile: (name: string) => Promise<void>
  saveProfile: () => Promise<void>
  deleteProfile: (id: string) => Promise<void>
  duplicateProfile: (id: string, newName: string) => Promise<void>
  applyProfile: (id: string) => Promise<void>
  updateProfileName: (name: string) => void
  updateProfileDescription: (description: string) => void
  addProvider: (
    id: string,
    config: OpenCodeProviderConfig,
    auth?: JsonValue
  ) => void
  updateProvider: (
    id: string,
    config: OpenCodeProviderConfig,
    auth?: JsonValue
  ) => void
  deleteProvider: (id: string) => void
  resetChanges: () => void
  setError: (error: string | null) => void
}

function profilesEqual(
  a: OpenCodeProfile | null,
  b: OpenCodeProfile | null
): boolean {
  if (!a || !b) return a === b
  return JSON.stringify(a) === JSON.stringify(b)
}

export const useOpenCodeStore = create<OpenCodeState>()(
  devtools(
    (set, get) => ({
      profiles: [],
      activeProfileId: null,
      currentProfile: null,
      originalProfile: null,
      hasChanges: false,
      isLoading: false,
      error: null,
      configStatus: null,
      providerTemplates: [],

      loadProfiles: async () => {
        set({ isLoading: true, error: null }, undefined, 'loadProfiles/start')
        try {
          const result = await commands.listOpencodeProfiles()
          if (result.status === 'ok') {
            let profiles = result.data
            // Create default profile if none exists
            if (profiles.length === 0) {
              const createResult = await commands.createDefaultProfile()
              if (createResult.status === 'ok') {
                profiles = [createResult.data]
              }
            }
            set(
              { profiles, isLoading: false },
              undefined,
              'loadProfiles/success'
            )
          } else {
            set(
              { error: result.error, isLoading: false },
              undefined,
              'loadProfiles/error'
            )
          }
        } catch (e) {
          set(
            { error: String(e), isLoading: false },
            undefined,
            'loadProfiles/exception'
          )
        }
      },

      loadActiveProfileId: async () => {
        try {
          const result = await commands.getActiveOpencodeProfileId()
          if (result.status === 'ok') {
            const activeId = result.data
            set({ activeProfileId: activeId }, undefined, 'loadActiveProfileId')
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
          // Silently ignore
        }
      },

      loadConfigStatus: async () => {
        try {
          const result = await commands.getOpencodeConfigStatus()
          if (result.status === 'ok') {
            set({ configStatus: result.data }, undefined, 'loadConfigStatus')
          }
        } catch {
          // Silently ignore
        }
      },

      loadProviderTemplates: async () => {
        try {
          const result = await commands.getOpencodeProviderTemplates()
          if (result.status === 'ok') {
            set(
              { providerTemplates: result.data },
              undefined,
              'loadProviderTemplates'
            )
          }
        } catch {
          // Silently ignore
        }
      },

      selectProfile: id => {
        const { profiles } = get()
        const profile = profiles.find(p => p.id === id)
        if (profile) {
          const copy = JSON.parse(JSON.stringify(profile))
          set(
            {
              currentProfile: copy,
              originalProfile: JSON.parse(JSON.stringify(profile)),
              hasChanges: false,
            },
            undefined,
            'selectProfile'
          )
        }
      },

      createProfile: async name => {
        const now = new Date().toISOString()
        const newProfile: OpenCodeProfile = {
          id: crypto.randomUUID(),
          name,
          description: null,
          createdAt: now,
          updatedAt: now,
          providers: {},
          auth: {},
        }
        try {
          const result = await commands.saveOpencodeProfile(newProfile)
          if (result.status === 'ok') {
            await get().loadProfiles()
            get().selectProfile(newProfile.id)
          } else {
            set({ error: result.error }, undefined, 'createProfile/error')
          }
        } catch (e) {
          set({ error: String(e) }, undefined, 'createProfile/exception')
        }
      },

      saveProfile: async () => {
        const { currentProfile } = get()
        if (!currentProfile) return

        set({ isLoading: true, error: null }, undefined, 'saveProfile/start')
        try {
          const result = await commands.saveOpencodeProfile(currentProfile)
          if (result.status === 'ok') {
            set(
              {
                originalProfile: JSON.parse(JSON.stringify(currentProfile)),
                hasChanges: false,
                isLoading: false,
              },
              undefined,
              'saveProfile/success'
            )
            await get().loadProfiles()
          } else {
            set(
              { error: result.error, isLoading: false },
              undefined,
              'saveProfile/error'
            )
          }
        } catch (e) {
          set(
            { error: String(e), isLoading: false },
            undefined,
            'saveProfile/exception'
          )
        }
      },

      deleteProfile: async id => {
        try {
          const result = await commands.deleteOpencodeProfile(id)
          if (result.status === 'ok') {
            const { currentProfile } = get()
            await get().loadProfiles()
            if (currentProfile?.id === id) {
              const { profiles } = get()
              if (profiles.length > 0 && profiles[0]) {
                get().selectProfile(profiles[0].id)
              } else {
                set(
                  { currentProfile: null, originalProfile: null },
                  undefined,
                  'deleteProfile/clear'
                )
              }
            }
          } else {
            set({ error: result.error }, undefined, 'deleteProfile/error')
          }
        } catch (e) {
          set({ error: String(e) }, undefined, 'deleteProfile/exception')
        }
      },

      duplicateProfile: async (id, newName) => {
        try {
          const result = await commands.duplicateOpencodeProfile(id, newName)
          if (result.status === 'ok') {
            await get().loadProfiles()
            get().selectProfile(result.data.id)
          } else {
            set({ error: result.error }, undefined, 'duplicateProfile/error')
          }
        } catch (e) {
          set({ error: String(e) }, undefined, 'duplicateProfile/exception')
        }
      },

      applyProfile: async id => {
        set({ isLoading: true, error: null }, undefined, 'applyProfile/start')
        try {
          const result = await commands.applyOpencodeProfile(id)
          if (result.status === 'ok') {
            set(
              { activeProfileId: id, isLoading: false },
              undefined,
              'applyProfile/success'
            )
            await get().loadConfigStatus()
          } else {
            set(
              { error: result.error, isLoading: false },
              undefined,
              'applyProfile/error'
            )
          }
        } catch (e) {
          set(
            { error: String(e), isLoading: false },
            undefined,
            'applyProfile/exception'
          )
        }
      },

      updateProfileName: name => {
        set(
          state => {
            if (!state.currentProfile) return state
            const updated = { ...state.currentProfile, name }
            return {
              currentProfile: updated,
              hasChanges: !profilesEqual(updated, state.originalProfile),
            }
          },
          undefined,
          'updateProfileName'
        )
      },

      updateProfileDescription: description => {
        set(
          state => {
            if (!state.currentProfile) return state
            const updated = {
              ...state.currentProfile,
              description: description || null,
            }
            return {
              currentProfile: updated,
              hasChanges: !profilesEqual(updated, state.originalProfile),
            }
          },
          undefined,
          'updateProfileDescription'
        )
      },

      addProvider: (id, config, auth) => {
        set(
          state => {
            if (!state.currentProfile) return state
            const updated = {
              ...state.currentProfile,
              providers: { ...state.currentProfile.providers, [id]: config },
              auth: auth
                ? { ...state.currentProfile.auth, [id]: auth }
                : state.currentProfile.auth,
            }
            return {
              currentProfile: updated,
              hasChanges: !profilesEqual(updated, state.originalProfile),
            }
          },
          undefined,
          'addProvider'
        )
      },

      updateProvider: (id, config, auth) => {
        set(
          state => {
            if (!state.currentProfile) return state
            const updated = {
              ...state.currentProfile,
              providers: { ...state.currentProfile.providers, [id]: config },
              auth: auth
                ? { ...state.currentProfile.auth, [id]: auth }
                : state.currentProfile.auth,
            }
            return {
              currentProfile: updated,
              hasChanges: !profilesEqual(updated, state.originalProfile),
            }
          },
          undefined,
          'updateProvider'
        )
      },

      deleteProvider: id => {
        set(
          state => {
            if (!state.currentProfile) return state
            const { [id]: _removed, ...providers } =
              state.currentProfile.providers
            const { [id]: _removedAuth, ...auth } = state.currentProfile.auth
            const updated = { ...state.currentProfile, providers, auth }
            return {
              currentProfile: updated,
              hasChanges: !profilesEqual(updated, state.originalProfile),
            }
          },
          undefined,
          'deleteProvider'
        )
      },

      resetChanges: () => {
        set(
          state => ({
            currentProfile: state.originalProfile
              ? JSON.parse(JSON.stringify(state.originalProfile))
              : null,
            hasChanges: false,
          }),
          undefined,
          'resetChanges'
        )
      },

      setError: error => set({ error }, undefined, 'setError'),
    }),
    { name: 'opencode-store' }
  )
)
