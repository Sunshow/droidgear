import { create } from 'zustand'
import { devtools, persist } from 'zustand/middleware'

type NavigationView = 'droid' | 'channels' | 'opencode' | 'codex' | 'openclaw'
type ToolView = 'droid' | 'opencode' | 'codex' | 'openclaw'
export type DroidSubView =
  | 'models'
  | 'helpers'
  | 'specs'
  | 'mcp'
  | 'sessions'
  | 'terminal'
export type OpenCodeSubView = 'providers'
export type OpenClawSubView = 'providers' | 'helpers'

export interface PendingUpdate {
  version: string
  body?: string
}

interface UIState {
  leftSidebarVisible: boolean
  rightSidebarVisible: boolean
  commandPaletteOpen: boolean
  preferencesOpen: boolean
  currentView: NavigationView
  lastToolView: ToolView
  droidSubView: DroidSubView
  opencodeSubView: OpenCodeSubView
  openclawSubView: OpenClawSubView
  lastSpecExportPath: string | null
  pendingUpdate: PendingUpdate | null

  toggleLeftSidebar: () => void
  setLeftSidebarVisible: (visible: boolean) => void
  toggleRightSidebar: () => void
  setRightSidebarVisible: (visible: boolean) => void
  toggleCommandPalette: () => void
  setCommandPaletteOpen: (open: boolean) => void
  togglePreferences: () => void
  setPreferencesOpen: (open: boolean) => void
  setCurrentView: (view: NavigationView) => void
  setDroidSubView: (view: DroidSubView) => void
  setOpenCodeSubView: (view: OpenCodeSubView) => void
  setOpenClawSubView: (view: OpenClawSubView) => void
  setLastSpecExportPath: (path: string) => void
  setPendingUpdate: (update: PendingUpdate | null) => void
  clearPendingUpdate: () => void
}

export const useUIStore = create<UIState>()(
  devtools(
    persist(
      set => ({
        leftSidebarVisible: true,
        rightSidebarVisible: false,
        commandPaletteOpen: false,
        preferencesOpen: false,
        currentView: 'droid',
        lastToolView: 'droid',
        droidSubView: 'models',
        opencodeSubView: 'providers',
        openclawSubView: 'providers',
        lastSpecExportPath: null,
        pendingUpdate: null,

        toggleLeftSidebar: () =>
          set(
            state => ({ leftSidebarVisible: !state.leftSidebarVisible }),
            undefined,
            'toggleLeftSidebar'
          ),

        setLeftSidebarVisible: visible =>
          set(
            { leftSidebarVisible: visible },
            undefined,
            'setLeftSidebarVisible'
          ),

        toggleRightSidebar: () =>
          set(
            state => ({ rightSidebarVisible: !state.rightSidebarVisible }),
            undefined,
            'toggleRightSidebar'
          ),

        setRightSidebarVisible: visible =>
          set(
            { rightSidebarVisible: visible },
            undefined,
            'setRightSidebarVisible'
          ),

        toggleCommandPalette: () =>
          set(
            state => ({ commandPaletteOpen: !state.commandPaletteOpen }),
            undefined,
            'toggleCommandPalette'
          ),

        setCommandPaletteOpen: open =>
          set({ commandPaletteOpen: open }, undefined, 'setCommandPaletteOpen'),

        togglePreferences: () =>
          set(
            state => ({ preferencesOpen: !state.preferencesOpen }),
            undefined,
            'togglePreferences'
          ),

        setPreferencesOpen: open =>
          set({ preferencesOpen: open }, undefined, 'setPreferencesOpen'),

        setCurrentView: view =>
          set(
            state => ({
              currentView: view,
              // Update lastToolView when switching tools
              lastToolView:
                view === 'droid' ||
                view === 'opencode' ||
                view === 'codex' ||
                view === 'openclaw'
                  ? view
                  : state.lastToolView,
            }),
            undefined,
            'setCurrentView'
          ),

        setDroidSubView: view =>
          set({ droidSubView: view }, undefined, 'setDroidSubView'),

        setOpenCodeSubView: view =>
          set({ opencodeSubView: view }, undefined, 'setOpenCodeSubView'),

        setOpenClawSubView: view =>
          set({ openclawSubView: view }, undefined, 'setOpenClawSubView'),

        setLastSpecExportPath: path =>
          set({ lastSpecExportPath: path }, undefined, 'setLastSpecExportPath'),

        setPendingUpdate: update =>
          set({ pendingUpdate: update }, undefined, 'setPendingUpdate'),

        clearPendingUpdate: () =>
          set({ pendingUpdate: null }, undefined, 'clearPendingUpdate'),
      }),
      {
        name: 'ui-store',
        partialize: state => ({
          lastSpecExportPath: state.lastSpecExportPath,
          currentView: state.currentView,
          lastToolView: state.lastToolView,
          leftSidebarVisible: state.leftSidebarVisible,
          rightSidebarVisible: state.rightSidebarVisible,
        }),
      }
    ),
    {
      name: 'ui-store',
    }
  )
)
