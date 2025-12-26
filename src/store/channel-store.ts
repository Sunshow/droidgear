import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import { commands, type Channel, type ChannelToken } from '@/lib/bindings'

interface ChannelState {
  channels: Channel[]
  originalChannels: Channel[]
  selectedChannelId: string | null
  tokens: Record<string, ChannelToken[]>
  hasChanges: boolean
  isLoading: boolean
  error: string | null

  // Actions
  loadChannels: () => Promise<void>
  saveChannels: () => Promise<void>
  addChannel: (channel: Channel) => void
  updateChannel: (id: string, channel: Partial<Channel>) => void
  deleteChannel: (id: string) => Promise<void>
  selectChannel: (id: string | null) => void
  fetchTokens: (channelId: string, baseUrl: string) => Promise<void>
  resetChanges: () => void
  setError: (error: string | null) => void
}

function channelsEqual(a: Channel[], b: Channel[]): boolean {
  if (a.length !== b.length) return false
  return JSON.stringify(a) === JSON.stringify(b)
}

export const useChannelStore = create<ChannelState>()(
  devtools(
    (set, get) => ({
      channels: [],
      originalChannels: [],
      selectedChannelId: null,
      tokens: {},
      hasChanges: false,
      isLoading: false,
      error: null,

      loadChannels: async () => {
        set({ isLoading: true, error: null }, undefined, 'loadChannels/start')
        try {
          const result = await commands.loadChannels()
          if (result.status === 'ok') {
            set(
              {
                channels: result.data,
                originalChannels: JSON.parse(JSON.stringify(result.data)),
                hasChanges: false,
                isLoading: false,
              },
              undefined,
              'loadChannels/success'
            )
          } else {
            set(
              { error: result.error, isLoading: false },
              undefined,
              'loadChannels/error'
            )
          }
        } catch (e) {
          set(
            { error: String(e), isLoading: false },
            undefined,
            'loadChannels/exception'
          )
        }
      },

      saveChannels: async () => {
        const { channels } = get()
        set({ isLoading: true, error: null }, undefined, 'saveChannels/start')
        try {
          const result = await commands.saveChannels(channels)
          if (result.status === 'ok') {
            set(
              {
                originalChannels: JSON.parse(JSON.stringify(channels)),
                hasChanges: false,
                isLoading: false,
              },
              undefined,
              'saveChannels/success'
            )
          } else {
            set(
              { error: result.error, isLoading: false },
              undefined,
              'saveChannels/error'
            )
          }
        } catch (e) {
          set(
            { error: String(e), isLoading: false },
            undefined,
            'saveChannels/exception'
          )
        }
      },

      addChannel: channel => {
        set(
          state => {
            const newChannels = [...state.channels, channel]
            return {
              channels: newChannels,
              hasChanges: !channelsEqual(newChannels, state.originalChannels),
            }
          },
          undefined,
          'addChannel'
        )
      },

      updateChannel: (id, updates) => {
        set(
          state => {
            const newChannels = state.channels.map(ch =>
              ch.id === id ? { ...ch, ...updates } : ch
            )
            return {
              channels: newChannels,
              hasChanges: !channelsEqual(newChannels, state.originalChannels),
            }
          },
          undefined,
          'updateChannel'
        )
      },

      deleteChannel: async id => {
        // Delete credentials from storage
        try {
          await commands.deleteChannelCredentials(id)
        } catch {
          // Ignore errors if credentials don't exist
        }

        set(
          state => {
            const newChannels = state.channels.filter(ch => ch.id !== id)
            const { [id]: _removed, ...remainingTokens } = state.tokens
            return {
              channels: newChannels,
              tokens: remainingTokens,
              selectedChannelId:
                state.selectedChannelId === id ? null : state.selectedChannelId,
              hasChanges: !channelsEqual(newChannels, state.originalChannels),
            }
          },
          undefined,
          'deleteChannel'
        )
      },

      selectChannel: id => {
        set({ selectedChannelId: id }, undefined, 'selectChannel')
      },

      fetchTokens: async (channelId, baseUrl) => {
        set({ isLoading: true, error: null }, undefined, 'fetchTokens/start')
        try {
          // Get credentials from storage
          const credResult = await commands.getChannelCredentials(channelId)
          if (credResult.status !== 'ok' || !credResult.data) {
            set(
              {
                error:
                  'Credentials not found. Please set the username and password first.',
                isLoading: false,
              },
              undefined,
              'fetchTokens/noCredentials'
            )
            return
          }

          const [username, password] = credResult.data
          const result = await commands.fetchChannelTokens(
            baseUrl,
            username,
            password
          )
          if (result.status === 'ok') {
            set(
              state => ({
                tokens: { ...state.tokens, [channelId]: result.data },
                isLoading: false,
              }),
              undefined,
              'fetchTokens/success'
            )
          } else {
            set(
              { error: result.error, isLoading: false },
              undefined,
              'fetchTokens/error'
            )
          }
        } catch (e) {
          set(
            { error: String(e), isLoading: false },
            undefined,
            'fetchTokens/exception'
          )
        }
      },

      resetChanges: () => {
        set(
          state => ({
            channels: JSON.parse(JSON.stringify(state.originalChannels)),
            hasChanges: false,
          }),
          undefined,
          'resetChanges'
        )
      },

      setError: error => set({ error }, undefined, 'setError'),
    }),
    { name: 'channel-store' }
  )
)
