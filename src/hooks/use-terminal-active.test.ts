import { describe, it, expect, beforeEach } from 'vitest'
import { renderHook } from '@testing-library/react'
import { useUIStore } from '@/store/ui-store'
import { useTerminalActive } from './use-terminal-active'

describe('useTerminalActive', () => {
  beforeEach(() => {
    useUIStore.getState().setCurrentView('droid')
    useUIStore.getState().setDroidSubView('models')
    useUIStore.getState().setCodexSubView('providers')
    useUIStore.getState().setOpenCodeSubView('providers')
    useUIStore.getState().setOpenClawSubView('providers')
    useUIStore.getState().setClaudeSubView('settings')
    useUIStore.getState().setHermesSubView('model')
    useUIStore.getState().setPiSubView('providers')
    useUIStore.getState().setOmpSubView('config')
    useUIStore.getState().setDshSubView('providers')
  })

  it('is false on a regular Droid sub-view', () => {
    const { result } = renderHook(() => useTerminalActive())
    expect(result.current).toBe(false)
  })

  it('is true when the Droid terminal sub-view is selected', () => {
    useUIStore.getState().setDroidSubView('terminal')
    const { result } = renderHook(() => useTerminalActive())
    expect(result.current).toBe(true)
  })

  it.each([
    ['codex', () => useUIStore.getState().setCodexSubView('terminal')],
    ['opencode', () => useUIStore.getState().setOpenCodeSubView('terminal')],
    ['openclaw', () => useUIStore.getState().setOpenClawSubView('terminal')],
    ['claude', () => useUIStore.getState().setClaudeSubView('terminal')],
    ['hermes', () => useUIStore.getState().setHermesSubView('terminal')],
    ['pi', () => useUIStore.getState().setPiSubView('terminal')],
    ['omp', () => useUIStore.getState().setOmpSubView('terminal')],
    ['dsh', () => useUIStore.getState().setDshSubView('terminal')],
  ] as const)('is true on the %s terminal sub-view', (_tool, select) => {
    useUIStore.getState().setCurrentView(_tool)
    select()
    const { result } = renderHook(() => useTerminalActive())
    expect(result.current).toBe(true)
  })

  it('is false when the terminal sub-view belongs to another tool', () => {
    useUIStore.getState().setCodexSubView('terminal')
    const { result } = renderHook(() => useTerminalActive())
    expect(result.current).toBe(false)
  })

  it('is false on non-tool views', () => {
    useUIStore.getState().setCurrentView('channels')
    const { result } = renderHook(() => useTerminalActive())
    expect(result.current).toBe(false)
  })
})
