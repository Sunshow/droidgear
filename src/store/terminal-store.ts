import { create } from 'zustand'
import { devtools, persist } from 'zustand/middleware'

// Terminal instance managed by frontend
export interface TerminalInstance {
  id: string
  name: string
  cwd: string
  status: 'running' | 'completed'
  hasNotification: boolean
  createdAt: number
}

interface TerminalState {
  terminals: TerminalInstance[]
  selectedTerminalId: string | null
  terminalForceDark: boolean
  terminalCopyOnSelect: boolean

  // Actions
  createTerminal: (name?: string, cwd?: string) => string
  closeTerminal: (id: string) => void
  renameTerminal: (id: string, name: string) => void
  selectTerminal: (id: string | null) => void
  updateTerminalStatus: (id: string, status: 'running' | 'completed') => void
  setTerminalNotification: (id: string, hasNotification: boolean) => void
  clearNotification: (id: string) => void
  setTerminalForceDark: (forceDark: boolean) => void
  setTerminalCopyOnSelect: (enabled: boolean) => void
}

let terminalCounter = 0

export const useTerminalStore = create<TerminalState>()(
  devtools(
    persist(
      (set, get) => ({
        terminals: [],
        selectedTerminalId: null,
        terminalForceDark: true,
        terminalCopyOnSelect: false,

        createTerminal: (name?: string, cwd?: string) => {
          // Sync counter with existing terminals to avoid ID conflicts
          const existingIds = get().terminals.map(t => {
            const match = t.id.match(/terminal-\d+-(\d+)/)
            return match?.[1] ? parseInt(match[1], 10) : 0
          })
          const maxExisting = Math.max(0, ...existingIds)
          if (terminalCounter <= maxExisting) {
            terminalCounter = maxExisting
          }
          terminalCounter++
          const id = `terminal-${Date.now()}-${terminalCounter}`
          const terminal: TerminalInstance = {
            id,
            name: name || `Terminal ${terminalCounter}`,
            cwd: cwd || '',
            status: 'running',
            hasNotification: false,
            createdAt: Date.now(),
          }
          set(
            state => ({
              terminals: [...state.terminals, terminal],
              selectedTerminalId: id,
            }),
            undefined,
            'createTerminal'
          )
          return id
        },

        closeTerminal: (id: string) => {
          set(
            state => {
              const newTerminals = state.terminals.filter(t => t.id !== id)
              const newSelectedId =
                state.selectedTerminalId === id
                  ? (newTerminals[0]?.id ?? null)
                  : state.selectedTerminalId
              return {
                terminals: newTerminals,
                selectedTerminalId: newSelectedId,
              }
            },
            undefined,
            'closeTerminal'
          )
        },

        renameTerminal: (id: string, name: string) => {
          set(
            state => ({
              terminals: state.terminals.map(t =>
                t.id === id ? { ...t, name } : t
              ),
            }),
            undefined,
            'renameTerminal'
          )
        },

        selectTerminal: (id: string | null) => {
          set({ selectedTerminalId: id }, undefined, 'selectTerminal')
          // Clear notification when selecting
          if (id) {
            const terminal = get().terminals.find(t => t.id === id)
            if (terminal?.hasNotification) {
              get().clearNotification(id)
            }
          }
        },

        updateTerminalStatus: (id: string, status: 'running' | 'completed') => {
          set(
            state => ({
              terminals: state.terminals.map(t =>
                t.id === id ? { ...t, status } : t
              ),
            }),
            undefined,
            'updateTerminalStatus'
          )
        },

        setTerminalNotification: (id: string, hasNotification: boolean) => {
          set(
            state => ({
              terminals: state.terminals.map(t =>
                t.id === id ? { ...t, hasNotification } : t
              ),
            }),
            undefined,
            'setTerminalNotification'
          )
        },

        clearNotification: (id: string) => {
          set(
            state => ({
              terminals: state.terminals.map(t =>
                t.id === id ? { ...t, hasNotification: false } : t
              ),
            }),
            undefined,
            'clearNotification'
          )
        },

        setTerminalForceDark: (forceDark: boolean) => {
          set(
            { terminalForceDark: forceDark },
            undefined,
            'setTerminalForceDark'
          )
        },

        setTerminalCopyOnSelect: (enabled: boolean) => {
          set(
            { terminalCopyOnSelect: enabled },
            undefined,
            'setTerminalCopyOnSelect'
          )
        },
      }),
      {
        name: 'terminal-store',
        partialize: state => ({
          terminals: state.terminals.map(t => ({
            id: t.id,
            name: t.name,
            cwd: t.cwd,
            status: 'completed' as const,
            hasNotification: false,
            createdAt: t.createdAt,
          })),
          selectedTerminalId: state.selectedTerminalId,
          terminalForceDark: state.terminalForceDark,
          terminalCopyOnSelect: state.terminalCopyOnSelect,
        }),
      }
    ),
    { name: 'terminal-store' }
  )
)
