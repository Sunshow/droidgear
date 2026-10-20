import { describe, expect, it, vi } from 'vitest'
import { act } from 'react'

const { TerminalMock, FitAddonMock, spawnMock, layout } = vi.hoisted(() => {
  class TerminalMock {
    static instances: TerminalMock[] = []
    options: Record<string, unknown> = {}
    element: HTMLElement | null = null
    parser = { registerOscHandler: vi.fn(() => ({ dispose: vi.fn() })) }
    focus = vi.fn()
    write = vi.fn()
    dispose = vi.fn()
    loadAddon = vi.fn()
    onData = vi.fn(() => ({ dispose: vi.fn() }))
    onSelectionChange = vi.fn(() => ({ dispose: vi.fn() }))
    getSelection = vi.fn(() => '')
    clearSelection = vi.fn()
    attachCustomKeyEventHandler = vi.fn()

    constructor(options: Record<string, unknown>) {
      this.options = options
      TerminalMock.instances.push(this)
    }

    open(element: HTMLElement) {
      this.element = element
    }
  }

  class FitAddonMock {
    fit = vi.fn()
    proposeDimensions = vi.fn(() => ({ cols: 80, rows: 24 }))
  }

  const pty = {
    onData: vi.fn(() => ({ dispose: vi.fn() })),
    onExit: vi.fn(() => ({ dispose: vi.fn() })),
    write: vi.fn(),
    kill: vi.fn(),
    resize: vi.fn(),
  }

  const spawnMock = vi.fn(() => pty)

  return {
    TerminalMock,
    FitAddonMock,
    spawnMock,
    layout: { width: 800, height: 600 },
  }
})

// jsdom has no layout engine; expose the stubbed container size instead.
Object.defineProperty(HTMLElement.prototype, 'clientWidth', {
  configurable: true,
  get: () => layout.width,
})
Object.defineProperty(HTMLElement.prototype, 'clientHeight', {
  configurable: true,
  get: () => layout.height,
})

class ResizeObserverMock {
  static instances: ResizeObserverMock[] = []
  observe = vi.fn()
  unobserve = vi.fn()
  disconnect = vi.fn()

  constructor(private readonly callback: ResizeObserverCallback) {
    ResizeObserverMock.instances.push(this)
  }

  fire() {
    this.callback([], this as unknown as ResizeObserver)
  }
}
vi.stubGlobal('ResizeObserver', ResizeObserverMock)

vi.mock('@xterm/xterm', () => ({ Terminal: TerminalMock }))
vi.mock('@xterm/addon-fit', () => ({ FitAddon: FitAddonMock }))
vi.mock('tauri-pty', () => ({ spawn: spawnMock }))
vi.mock('@tauri-apps/plugin-os', () => ({
  platform: () => 'windows',
  version: () => '10.0.26300',
}))
vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({
  writeText: vi.fn().mockResolvedValue(undefined),
  readText: vi.fn().mockResolvedValue(''),
}))
vi.mock('@/services/shell-env', () => ({
  getShellEnv: vi.fn().mockResolvedValue({}),
}))
vi.mock('@/services/preferences', () => ({
  usePreferences: () => ({ data: undefined }),
}))
vi.mock('@/lib/notifications', () => ({ notify: vi.fn() }))

import { render, waitFor } from '@/test/test-utils'
import { TerminalView } from './TerminalView'

async function renderView() {
  const spawnedBefore = spawnMock.mock.calls.length
  const { container } = render(<TerminalView terminalId="t1" />)
  const root = container.firstElementChild as HTMLElement
  // Flush shell-env loading and terminal initialization so pointer events
  // are dispatched against a fully initialized terminal.
  await waitFor(() =>
    expect(spawnMock.mock.calls.length).toBeGreaterThan(spawnedBefore)
  )
  return { root }
}

function pointerDown(root: HTMLElement) {
  const event = new MouseEvent('pointerdown', {
    bubbles: true,
    cancelable: true,
  })
  root.dispatchEvent(event)
  return event
}

describe('TerminalView pointer handling', () => {
  it('keeps pointerdown uncancelled so xterm mouse selection works', async () => {
    const { root } = await renderView()

    const event = pointerDown(root)

    expect(event.defaultPrevented).toBe(false)
  })

  it('cancels pointerdown only while an IME composition is active', async () => {
    const { root } = await renderView()

    // xterm marks an active IME composition on its composition view
    const compositionView = document.createElement('div')
    compositionView.className = 'composition-view active'
    root.appendChild(compositionView)

    const whileComposing = pointerDown(root)
    expect(whileComposing.defaultPrevented).toBe(true)

    compositionView.classList.remove('active')
    const afterComposition = pointerDown(root)
    expect(afterComposition.defaultPrevented).toBe(false)
  })
})

describe('TerminalView PTY lifecycle', () => {
  it('tells xterm the Windows PTY is ConPTY so row growth cannot drop lines', async () => {
    TerminalMock.instances.length = 0
    await renderView()

    expect(TerminalMock.instances[0]?.options.windowsPty).toEqual({
      backend: 'conpty',
      buildNumber: 26300,
    })
  })

  it('spawns the PTY only once the container has a real layout', async () => {
    const spawnedBefore = spawnMock.mock.calls.length
    layout.width = 0
    layout.height = 0

    render(<TerminalView terminalId="t1" />)
    await act(async () => {
      await Promise.resolve()
    })
    expect(spawnMock.mock.calls.length).toBe(spawnedBefore)

    // The container becomes measurable (e.g. the hidden terminal page is
    // shown); only then may the PTY spawn with real dimensions.
    layout.width = 800
    layout.height = 600
    act(() => {
      for (const observer of ResizeObserverMock.instances) observer.fire()
    })
    await waitFor(() =>
      expect(spawnMock.mock.calls.length).toBeGreaterThan(spawnedBefore)
    )
  })
})
