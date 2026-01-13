import { create } from 'zustand'
import { devtools, persist } from 'zustand/middleware'

// Terminal snippet for quick input
export interface TerminalSnippet {
  id: string
  name: string
  content: string
  autoExecute: boolean
}

// Derived terminal (child of a main terminal)
export interface DerivedTerminal {
  id: string
  name: string
  command: string
  status: 'running' | 'completed'
  hasNotification: boolean
  createdAt: number
}

// Terminal instance managed by frontend
export interface TerminalInstance {
  id: string
  name: string
  cwd: string
  status: 'running' | 'completed'
  hasNotification: boolean
  createdAt: number
  derivedTerminals: DerivedTerminal[]
  selectedDerivedId: string | null // null = main terminal
}

interface TerminalState {
  terminals: TerminalInstance[]
  selectedTerminalId: string | null
  terminalForceDark: boolean
  terminalCopyOnSelect: boolean
  snippets: TerminalSnippet[]

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

  // Derived terminal actions
  createDerivedTerminal: (
    parentId: string,
    command: string,
    name?: string
  ) => string
  closeDerivedTerminal: (parentId: string, derivedId: string) => void
  selectDerivedTerminal: (parentId: string, derivedId: string | null) => void
  renameDerivedTerminal: (
    parentId: string,
    derivedId: string,
    name: string
  ) => void
  updateDerivedTerminalStatus: (
    parentId: string,
    derivedId: string,
    status: 'running' | 'completed'
  ) => void
  setDerivedTerminalNotification: (
    parentId: string,
    derivedId: string,
    hasNotification: boolean
  ) => void

  // Snippet actions
  addSnippet: (snippet: Omit<TerminalSnippet, 'id'>) => string
  updateSnippet: (
    id: string,
    updates: Partial<Omit<TerminalSnippet, 'id'>>
  ) => void
  removeSnippet: (id: string) => void
  reorderSnippets: (ids: string[]) => void
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
        snippets: [],

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
            derivedTerminals: [],
            selectedDerivedId: null,
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

        // Derived terminal actions
        createDerivedTerminal: (
          parentId: string,
          command: string,
          name?: string
        ) => {
          const derivedId = `derived-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
          const derivedName =
            name || command.split(' ')[0] || `Derived ${derivedId.slice(-4)}`
          const derived: DerivedTerminal = {
            id: derivedId,
            name: derivedName,
            command,
            status: 'running',
            hasNotification: false,
            createdAt: Date.now(),
          }
          set(
            state => ({
              terminals: state.terminals.map(t =>
                t.id === parentId
                  ? {
                      ...t,
                      derivedTerminals: [...t.derivedTerminals, derived],
                      selectedDerivedId: derivedId,
                    }
                  : t
              ),
            }),
            undefined,
            'createDerivedTerminal'
          )
          return derivedId
        },

        closeDerivedTerminal: (parentId: string, derivedId: string) => {
          set(
            state => ({
              terminals: state.terminals.map(t => {
                if (t.id !== parentId) return t
                const newDerived = t.derivedTerminals.filter(
                  d => d.id !== derivedId
                )
                const newSelectedId =
                  t.selectedDerivedId === derivedId
                    ? null // Go back to main terminal
                    : t.selectedDerivedId
                return {
                  ...t,
                  derivedTerminals: newDerived,
                  selectedDerivedId: newSelectedId,
                }
              }),
            }),
            undefined,
            'closeDerivedTerminal'
          )
        },

        selectDerivedTerminal: (parentId: string, derivedId: string | null) => {
          set(
            state => ({
              terminals: state.terminals.map(t =>
                t.id === parentId ? { ...t, selectedDerivedId: derivedId } : t
              ),
            }),
            undefined,
            'selectDerivedTerminal'
          )
          // Clear notification when selecting
          if (derivedId) {
            const terminal = get().terminals.find(t => t.id === parentId)
            const derived = terminal?.derivedTerminals.find(
              d => d.id === derivedId
            )
            if (derived?.hasNotification) {
              get().setDerivedTerminalNotification(parentId, derivedId, false)
            }
          }
        },

        renameDerivedTerminal: (
          parentId: string,
          derivedId: string,
          name: string
        ) => {
          set(
            state => ({
              terminals: state.terminals.map(t =>
                t.id === parentId
                  ? {
                      ...t,
                      derivedTerminals: t.derivedTerminals.map(d =>
                        d.id === derivedId ? { ...d, name } : d
                      ),
                    }
                  : t
              ),
            }),
            undefined,
            'renameDerivedTerminal'
          )
        },

        updateDerivedTerminalStatus: (
          parentId: string,
          derivedId: string,
          status: 'running' | 'completed'
        ) => {
          set(
            state => ({
              terminals: state.terminals.map(t =>
                t.id === parentId
                  ? {
                      ...t,
                      derivedTerminals: t.derivedTerminals.map(d =>
                        d.id === derivedId ? { ...d, status } : d
                      ),
                    }
                  : t
              ),
            }),
            undefined,
            'updateDerivedTerminalStatus'
          )
        },

        setDerivedTerminalNotification: (
          parentId: string,
          derivedId: string,
          hasNotification: boolean
        ) => {
          set(
            state => ({
              terminals: state.terminals.map(t =>
                t.id === parentId
                  ? {
                      ...t,
                      derivedTerminals: t.derivedTerminals.map(d =>
                        d.id === derivedId ? { ...d, hasNotification } : d
                      ),
                    }
                  : t
              ),
            }),
            undefined,
            'setDerivedTerminalNotification'
          )
        },

        // Snippet actions
        addSnippet: (snippet: Omit<TerminalSnippet, 'id'>) => {
          const id = `snippet-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
          set(
            state => ({
              snippets: [...state.snippets, { ...snippet, id }],
            }),
            undefined,
            'addSnippet'
          )
          return id
        },

        updateSnippet: (
          id: string,
          updates: Partial<Omit<TerminalSnippet, 'id'>>
        ) => {
          set(
            state => ({
              snippets: state.snippets.map(s =>
                s.id === id ? { ...s, ...updates } : s
              ),
            }),
            undefined,
            'updateSnippet'
          )
        },

        removeSnippet: (id: string) => {
          set(
            state => ({
              snippets: state.snippets.filter(s => s.id !== id),
            }),
            undefined,
            'removeSnippet'
          )
        },

        reorderSnippets: (ids: string[]) => {
          set(
            state => {
              const snippetMap = new Map(state.snippets.map(s => [s.id, s]))
              const reordered = ids
                .map(id => snippetMap.get(id))
                .filter((s): s is TerminalSnippet => s !== undefined)
              return { snippets: reordered }
            },
            undefined,
            'reorderSnippets'
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
            derivedTerminals: t.derivedTerminals.map(d => ({
              ...d,
              status: 'completed' as const,
              hasNotification: false,
            })),
            selectedDerivedId: t.selectedDerivedId,
          })),
          selectedTerminalId: state.selectedTerminalId,
          terminalForceDark: state.terminalForceDark,
          terminalCopyOnSelect: state.terminalCopyOnSelect,
          snippets: state.snippets,
        }),
        merge: (persistedState, currentState) => {
          const persisted = persistedState as Partial<TerminalState>
          return {
            ...currentState,
            ...persisted,
            terminals: (persisted.terminals ?? []).map(t => ({
              ...t,
              derivedTerminals: t.derivedTerminals ?? [],
              selectedDerivedId: t.selectedDerivedId ?? null,
            })),
            snippets: persisted.snippets ?? [],
          }
        },
      }
    ),
    { name: 'terminal-store' }
  )
)
