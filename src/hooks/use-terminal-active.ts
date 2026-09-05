import { useUIStore } from '@/store/ui-store'

/**
 * Whether the shared in-app terminal page is the active view.
 *
 * The terminal page is reachable from every tool panel (Droid, OpenCode,
 * Codex, Claude, OpenClaw, Hermes, Pi, Omp, Dsh). Each tool's sidebar has a
 * "Terminal" entry that selects its `terminal` sub-view.
 */
export function useTerminalActive(): boolean {
  const currentView = useUIStore(state => state.currentView)
  const droidSubView = useUIStore(state => state.droidSubView)
  const codexSubView = useUIStore(state => state.codexSubView)
  const opencodeSubView = useUIStore(state => state.opencodeSubView)
  const openclawSubView = useUIStore(state => state.openclawSubView)
  const claudeSubView = useUIStore(state => state.claudeSubView)
  const hermesSubView = useUIStore(state => state.hermesSubView)
  const piSubView = useUIStore(state => state.piSubView)
  const ompSubView = useUIStore(state => state.ompSubView)
  const dshSubView = useUIStore(state => state.dshSubView)

  switch (currentView) {
    case 'droid':
      return droidSubView === 'terminal'
    case 'codex':
      return codexSubView === 'terminal'
    case 'opencode':
      return opencodeSubView === 'terminal'
    case 'openclaw':
      return openclawSubView === 'terminal'
    case 'claude':
      return claudeSubView === 'terminal'
    case 'hermes':
      return hermesSubView === 'terminal'
    case 'pi':
      return piSubView === 'terminal'
    case 'omp':
      return ompSubView === 'terminal'
    case 'dsh':
      return dshSubView === 'terminal'
    default:
      return false
  }
}
