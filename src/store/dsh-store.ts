import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import i18n from '@/i18n/config'
import {
  commands,
  type DshProviderConfig,
  type DshConfigSnapshot,
  type DshConfigStatus,
  type JsonValue,
} from '@/lib/bindings'

/** Turn backend config write codes into user-facing copy. */
export function friendlyConfigWriteError(error: string): string {
  if (error.startsWith('DSH_CONFIG_CONFLICT'))
    return i18n.t('dsh.config.conflict')
  if (error.startsWith('DSH_CONFIG_SHADOWED')) {
    return i18n.t('dsh.config.homePatchShadow')
  }
  return error
}

interface DshState {
  providers: Partial<Record<string, DshProviderConfig>>
  credentials: Partial<Record<string, string>>
  /** Official runtime profile the current page manages (`desktop` | `web`);
   * `null` only when no official profile exists (legacy layout). Set by the
   * page on mount — there is no profile switching UI. */
  profile: string | null
  isLoading: boolean
  error: string | null
  configStatus: DshConfigStatus | null
  /** Composed configuration for the active profile (all layers). */
  snapshot: DshConfigSnapshot | null
  isLoadingSnapshot: boolean
  /** Entry whose write is in flight, for per-card spinners. */
  savingEntryId: string | null

  setProfile: (profile: string | null) => void
  loadProviders: () => Promise<void>
  loadCredentials: () => Promise<void>
  loadConfigStatus: () => Promise<void>
  loadSnapshot: () => Promise<void>
  /**
   * Replace one entry's complete config in the profile patch.
   * `config: null` removes the override so the bundle defaults apply again.
   */
  saveEntry: (entryId: string, config: JsonValue | null) => Promise<void>
  toggleEntryDisabled: (entryId: string, disabled: boolean) => Promise<void>
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
      profile: null,
      isLoading: false,
      error: null,
      configStatus: null,
      snapshot: null,
      isLoadingSnapshot: false,
      savingEntryId: null,

      setProfile: profile => {
        if (get().profile === profile) return
        set(
          { profile, snapshot: null, savingEntryId: null },
          undefined,
          'dsh/setProfile'
        )
      },

      loadProviders: async () => {
        set(
          { isLoading: true, error: null },
          undefined,
          'dsh/loadProviders/start'
        )
        try {
          const result = await commands.readDshCurrentConfig(get().profile)
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
          const result = await commands.getDshConfigStatus(get().profile)
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

      loadSnapshot: async () => {
        set({ isLoadingSnapshot: true }, undefined, 'dsh/loadSnapshot/start')
        try {
          const result = await commands.readDshConfigSnapshot(get().profile)
          if (result.status === 'ok') {
            set(
              { snapshot: result.data, isLoadingSnapshot: false },
              undefined,
              'dsh/loadSnapshot/success'
            )
          } else {
            set(
              {
                error: friendlyConfigWriteError(result.error),
                isLoadingSnapshot: false,
              },
              undefined,
              'dsh/loadSnapshot/error'
            )
          }
        } catch (e) {
          set(
            { error: String(e), isLoadingSnapshot: false },
            undefined,
            'dsh/loadSnapshot/exception'
          )
        }
      },

      saveEntry: async (entryId, config) => {
        set({ savingEntryId: entryId }, undefined, 'dsh/saveEntry/start')
        try {
          const expectedHash = get().snapshot?.patchHash ?? null
          const result = await commands.saveDshConfigEntry(
            get().profile,
            entryId,
            config,
            expectedHash
          )
          if (result.status !== 'ok') {
            const error = friendlyConfigWriteError(result.error)
            set({ error }, undefined, 'dsh/saveEntry/error')
            throw new Error(error)
          }
          set({ error: null }, undefined, 'dsh/saveEntry/success')
          await get().loadSnapshot()
        } finally {
          if (get().savingEntryId === entryId) {
            set({ savingEntryId: null }, undefined, 'dsh/saveEntry/finish')
          }
        }
      },

      toggleEntryDisabled: async (entryId, disabled) => {
        set(
          { savingEntryId: entryId },
          undefined,
          'dsh/toggleEntryDisabled/start'
        )
        try {
          const expectedHash = get().snapshot?.patchHash ?? null
          const result = await commands.setDshConfigEntryDisabled(
            get().profile,
            entryId,
            disabled,
            expectedHash
          )
          if (result.status !== 'ok') {
            const error = friendlyConfigWriteError(result.error)
            set({ error }, undefined, 'dsh/toggleEntryDisabled/error')
            throw new Error(error)
          }
          set({ error: null }, undefined, 'dsh/toggleEntryDisabled/success')
          await get().loadSnapshot()
        } finally {
          if (get().savingEntryId === entryId) {
            set(
              { savingEntryId: null },
              undefined,
              'dsh/toggleEntryDisabled/finish'
            )
          }
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
        const result = await commands.saveDshProvider(get().profile, id, config)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'dsh/saveProvider/error')
          throw new Error(result.error)
        }
        set(
          { providers: { ...get().providers, [id]: config } },
          undefined,
          'dsh/saveProvider/success'
        )
        // 重新读取，展示后端补齐（注册表元数据、协议兼容处理）后的结果
        await get().loadConfigStatus()
        await get().loadProviders()
        // Provider writes change the profile patch hash; keep the snapshot
        // usable for a follow-up `saveEntry` (default model) without conflicts.
        await get().loadSnapshot()
      },

      deleteProvider: async id => {
        const result = await commands.deleteDshProvider(get().profile, id)
        if (result.status !== 'ok') {
          set({ error: result.error }, undefined, 'dsh/deleteProvider/error')
          return
        }
        const { [id]: _removed, ...providers } = get().providers
        set({ providers }, undefined, 'dsh/deleteProvider/success')
        await get().loadConfigStatus()
        await get().loadSnapshot()
      },

      setError: error => set({ error }, undefined, 'dsh/setError'),
    }),
    { name: 'dsh-store' }
  )
)
