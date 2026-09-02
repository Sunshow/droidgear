import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import {
  commands,
  type DshProviderConfig,
  type DshConfigStatus,
} from '@/lib/bindings'

interface DshState {
  providers: Partial<Record<string, DshProviderConfig>>
  credentials: Partial<Record<string, string>>
  isLoading: boolean
  error: string | null
  configStatus: DshConfigStatus | null

  loadProviders: () => Promise<void>
  loadCredentials: () => Promise<void>
  loadConfigStatus: () => Promise<void>
  saveProvider: (id: string, config: DshProviderConfig) => Promise<void>
  deleteProvider: (id: string) => Promise<void>
  saveCredential: (name: string, value: string) => Promise<void>
  deleteCredential: (name: string) => Promise<void>
  setError: (error: string | null) => void
}

export const useDshStore = create<DshState>()(
  devtools(
    (set, get) => ({
      providers: {},
      credentials: {},
      isLoading: false,
      error: null,
      configStatus: null,

      loadProviders: async () => {
        set(
          { isLoading: true, error: null },
          undefined,
          'dsh/loadProviders/start'
        )
        try {
          const result = await commands.readDshCurrentConfig()
          if (result.status === 'ok') {
            set(
              { providers: result.data.providers, isLoading: false },
              undefined,
              'dsh/loadProviders/success'
            )
          } else {
            set(
              { error: result.error, isLoading: false },
              undefined,
              'dsh/loadProviders/error'
            )
          }
        } catch (e) {
          set(
            { error: String(e), isLoading: false },
            undefined,
            'dsh/loadProviders/exception'
          )
        }
      },

      loadConfigStatus: async () => {
        try {
          const result = await commands.getDshConfigStatus()
          if (result.status === 'ok') {
            set(
              { configStatus: result.data },
              undefined,
              'dsh/loadConfigStatus'
            )
          }
        } catch {
          // ignore
        }
      },

      loadCredentials: async () => {
        try {
          const result = await commands.readDshCredentials()
          if (result.status === 'ok') {
            set(
              { credentials: result.data.refs },
              undefined,
              'dsh/loadCredentials'
            )
          }
        } catch {
          // ignore
        }
      },

      saveCredential: async (name, value) => {
        const result = await commands.saveDshCredentialRef(name, value)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'dsh/saveCredential/error')
          throw new Error(result.error)
        }
        const credentials = value.trim()
          ? { ...get().credentials, [name]: value }
          : (() => {
              const { [name]: _removed, ...rest } = get().credentials
              return rest
            })()
        set({ credentials }, undefined, 'dsh/saveCredential/success')
        await get().loadConfigStatus()
      },

      deleteCredential: async name => {
        const result = await commands.deleteDshCredentialRef(name)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'dsh/deleteCredential/error')
          return
        }
        const { [name]: _removed, ...credentials } = get().credentials
        set({ credentials }, undefined, 'dsh/deleteCredential/success')
        await get().loadConfigStatus()
      },

      saveProvider: async (id, config) => {
        const result = await commands.saveDshProvider(id, config)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'dsh/saveProvider/error')
          throw new Error(result.error)
        }
        set(
          { providers: { ...get().providers, [id]: config } },
          undefined,
          'dsh/saveProvider/success'
        )
        await get().loadConfigStatus()
      },

      deleteProvider: async id => {
        const result = await commands.deleteDshProvider(id)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'dsh/deleteProvider/error')
          return
        }
        const { [id]: _removed, ...providers } = get().providers
        set({ providers }, undefined, 'dsh/deleteProvider/success')
        await get().loadConfigStatus()
      },

      setError: error => set({ error }, undefined, 'dsh/setError'),
    }),
    { name: 'dsh-store' }
  )
)
