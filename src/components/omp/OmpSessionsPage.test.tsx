import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { act, fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'

const { commandMocks, showContextMenuMock, listenMock } = vi.hoisted(() => ({
  listenMock: vi.fn(),
  showContextMenuMock: vi.fn(),
  commandMocks: {
    listOmpSessions: vi.fn(),
    getOmpSessionDetail: vi.fn(),
    deleteOmpSession: vi.fn(),
    startOmpSessionsWatcher: vi.fn(),
    stopOmpSessionsWatcher: vi.fn(),
  },
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: listenMock,
}))

vi.mock('@/lib/context-menu', () => ({
  showContextMenu: showContextMenuMock,
}))

vi.mock('@/lib/bindings', () => ({
  commands: commandMocks,
}))

vi.mock('streamdown', () => ({
  Streamdown: ({ children }: { children: string }) => (
    <div data-testid="streamdown">{children}</div>
  ),
}))

import { render, screen, waitFor } from '@/test/test-utils'
import { OmpSessionsPage } from './OmpSessionsPage'

Object.defineProperty(Element.prototype, 'scrollTo', {
  writable: true,
  configurable: true,
  value: vi.fn(),
})

const session = {
  id: 'omp-session-1',
  title: 'Inspect the parser',
  project: '/work/demo',
  model: 'gpt-5',
  modelProvider: 'openai',
  modifiedAt: 1700000000000,
  messageCount: 4,
  tokenUsage: {
    inputTokens: 100,
    outputTokens: 20,
    cacheReadTokens: 0,
    cacheCreationTokens: 0,
    reasoningTokens: 0,
    totalTokens: 120,
    cost: 0.01,
  },
  path: 'C:\\home\\user\\.omp\\agent\\sessions\\project\\session.jsonl',
}

const detail = {
  summary: session,
  messages: [
    {
      id: 'message-1',
      role: 'user',
      content: [{ type: 'text', text: 'Inspect the parser' }],
      timestamp: '2026-01-01T00:00:00Z',
      isActiveBranch: true,
    },
    {
      id: 'message-2',
      role: 'assistant',
      content: [
        { type: 'text', text: 'Checking the parser.' },
        { type: 'toolCall', text: 'bash: Checking the parser' },
      ],
      timestamp: '2026-01-01T00:00:01Z',
      isActiveBranch: true,
    },
    {
      id: 'message-3',
      role: 'toolResult',
      content: [{ type: 'text', text: 'parser output here' }],
      timestamp: '2026-01-01T00:00:02Z',
      isActiveBranch: true,
    },
  ],
}

describe('OmpSessionsPage', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  beforeEach(() => {
    vi.clearAllMocks()
    localStorage.clear()
    listenMock.mockResolvedValue(() => undefined)
    commandMocks.listOmpSessions.mockResolvedValue({
      status: 'ok',
      data: [session],
    })
    commandMocks.getOmpSessionDetail.mockResolvedValue({
      status: 'ok',
      data: detail,
    })
    commandMocks.deleteOmpSession.mockResolvedValue({
      status: 'ok',
      data: null,
    })
    commandMocks.startOmpSessionsWatcher.mockResolvedValue({
      status: 'ok',
      data: null,
    })
    commandMocks.stopOmpSessionsWatcher.mockResolvedValue({
      status: 'ok',
      data: null,
    })
    showContextMenuMock.mockResolvedValue(undefined)
  })

  it('lists sessions and starts the watcher', async () => {
    render(<OmpSessionsPage />)

    expect(await screen.findByText('Inspect the parser')).toBeInTheDocument()
    expect(commandMocks.startOmpSessionsWatcher).toHaveBeenCalled()
  })

  it('reloads the list when the watcher reports a change', async () => {
    render(<OmpSessionsPage />)
    await screen.findByText('Inspect the parser')

    commandMocks.listOmpSessions.mockResolvedValue({
      status: 'ok',
      data: [{ ...session, title: 'Updated session' }],
    })
    const onChange = listenMock.mock.calls[0]?.[1] as (() => void) | undefined
    onChange?.()

    expect(await screen.findByText('Updated session')).toBeInTheDocument()
    expect(commandMocks.listOmpSessions).toHaveBeenCalledTimes(2)
  })

  it('loads the selected session detail and renders tool calls', async () => {
    const user = userEvent.setup()
    render(<OmpSessionsPage />)

    await user.click(await screen.findByText('Inspect the parser'))

    await waitFor(() => {
      expect(commandMocks.getOmpSessionDetail).toHaveBeenCalledWith(
        session.path
      )
    })
    expect(await screen.findByText('Checking the parser.')).toBeInTheDocument()
    // Tool calls are surfaced as a compact summary line.
    expect(
      await screen.findByText('bash: Checking the parser')
    ).toBeInTheDocument()
  })

  it('keeps tool results collapsed until expanded', async () => {
    const user = userEvent.setup()
    render(<OmpSessionsPage />)

    await user.click(await screen.findByText('Inspect the parser'))
    await screen.findByText('Checking the parser.')

    // jsdom doesn't hide `<details>` content, so assert on the open state.
    const resultBody = screen
      .getByText('Tool result')
      .closest('details') as HTMLElement
    expect(resultBody).not.toHaveAttribute('open')
    await user.click(screen.getByText('Tool result'))
    expect(resultBody).toHaveAttribute('open')
    expect(await screen.findByText('parser output here')).toBeInTheDocument()
  })

  it('renders the list in batches and loads more when the sentinel appears', async () => {
    const manySessions = Array.from({ length: 45 }, (_, index) => ({
      ...session,
      id: `omp-session-${index}`,
      title: `Session ${index}`,
    }))
    commandMocks.listOmpSessions.mockResolvedValue({
      status: 'ok',
      data: manySessions,
    })

    // Stub IntersectionObserver and capture the sentinel callback so the
    // test can simulate scrolling to the bottom of the list.
    let triggerSentinel: (() => void) | undefined
    vi.stubGlobal(
      'IntersectionObserver',
      class {
        root = null
        rootMargin = ''
        thresholds: number[] = []
        constructor(
          callback: IntersectionObserverCallback,
          _options?: IntersectionObserverInit
        ) {
          triggerSentinel = () =>
            callback(
              [{ isIntersecting: true } as IntersectionObserverEntry],
              this as unknown as IntersectionObserver
            )
        }
        observe() {
          /* no-op in tests */
        }
        unobserve() {
          /* no-op in tests */
        }
        disconnect() {
          /* no-op in tests */
        }
        takeRecords(): IntersectionObserverEntry[] {
          return []
        }
      }
    )

    render(<OmpSessionsPage />)

    expect(await screen.findByText('Session 0')).toBeInTheDocument()
    expect(screen.getByText('Session 19')).toBeInTheDocument()
    expect(screen.queryByText('Session 20')).not.toBeInTheDocument()

    act(() => triggerSentinel?.())

    expect(await screen.findByText('Session 39')).toBeInTheDocument()
    expect(screen.queryByText('Session 40')).not.toBeInTheDocument()
  })

  it('deletes a session through the context menu', async () => {
    showContextMenuMock.mockImplementation(
      async (items: { id: string; action: () => Promise<void> }[]) => {
        await items.find(item => item.id === 'delete')?.action()
      }
    )
    render(<OmpSessionsPage />)

    const item = await screen.findByText('Inspect the parser')
    fireEvent.contextMenu(item)

    await waitFor(() => {
      expect(commandMocks.deleteOmpSession).toHaveBeenCalledWith(session.path)
    })
    expect(commandMocks.listOmpSessions).toHaveBeenCalledTimes(2)
  })
})
