import { beforeEach, describe, expect, it, vi } from 'vitest'
import { fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'

const { commandMocks, showContextMenuMock, listenMock } = vi.hoisted(() => ({
  listenMock: vi.fn(),
  showContextMenuMock: vi.fn(),
  commandMocks: {
    listPiSessions: vi.fn(),
    getPiSessionDetail: vi.fn(),
    deletePiSession: vi.fn(),
    startPiSessionsWatcher: vi.fn(),
    stopPiSessionsWatcher: vi.fn(),
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
import { PiSessionsPage } from './PiSessionsPage'

Object.defineProperty(Element.prototype, 'scrollTo', {
  writable: true,
  configurable: true,
  value: vi.fn(),
})

const session = {
  id: 'pi-session-1',
  title: 'Inspect the parser',
  project: '/work/demo',
  model: 'claude-sonnet',
  modelProvider: 'anthropic',
  modifiedAt: 1700000000000,
  messageCount: 3,
  tokenUsage: {
    inputTokens: 100,
    outputTokens: 20,
    cacheReadTokens: 0,
    cacheCreationTokens: 0,
    reasoningTokens: 0,
    totalTokens: 120,
    cost: 0.01,
  },
  path: '/home/user/.pi/agent/sessions/project/session.jsonl',
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
      content: [{ type: 'text', text: 'The parser is working.' }],
      timestamp: '2026-01-01T00:00:01Z',
      isActiveBranch: true,
    },
  ],
}

describe('PiSessionsPage', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    localStorage.clear()
    listenMock.mockResolvedValue(() => undefined)
    commandMocks.listPiSessions.mockResolvedValue({
      status: 'ok',
      data: [session],
    })
    commandMocks.getPiSessionDetail.mockResolvedValue({
      status: 'ok',
      data: detail,
    })
    commandMocks.deletePiSession.mockResolvedValue({
      status: 'ok',
      data: null,
    })
    commandMocks.startPiSessionsWatcher.mockResolvedValue({
      status: 'ok',
      data: null,
    })
    commandMocks.stopPiSessionsWatcher.mockResolvedValue({
      status: 'ok',
      data: null,
    })
    showContextMenuMock.mockResolvedValue(undefined)
  })

  it('lists sessions and starts the watcher', async () => {
    render(<PiSessionsPage />)

    expect(await screen.findByText('Inspect the parser')).toBeInTheDocument()
    expect(commandMocks.startPiSessionsWatcher).toHaveBeenCalled()
  })

  it('reloads the list when the watcher reports a change', async () => {
    render(<PiSessionsPage />)
    await screen.findByText('Inspect the parser')

    commandMocks.listPiSessions.mockResolvedValue({
      status: 'ok',
      data: [{ ...session, title: 'Updated session' }],
    })
    const onChange = listenMock.mock.calls[0]?.[1] as (() => void) | undefined
    onChange?.()

    expect(await screen.findByText('Updated session')).toBeInTheDocument()
    expect(commandMocks.listPiSessions).toHaveBeenCalledTimes(2)
  })

  it('loads the selected session detail', async () => {
    const user = userEvent.setup()
    render(<PiSessionsPage />)

    await user.click(await screen.findByText('Inspect the parser'))

    await waitFor(() => {
      expect(commandMocks.getPiSessionDetail).toHaveBeenCalledWith(session.path)
    })
    expect(
      await screen.findByText('The parser is working.')
    ).toBeInTheDocument()
  })

  it('deletes a session through the context menu', async () => {
    showContextMenuMock.mockImplementation(
      async (items: { id: string; action: () => Promise<void> }[]) => {
        await items.find(item => item.id === 'delete')?.action()
      }
    )
    render(<PiSessionsPage />)

    const item = await screen.findByText('Inspect the parser')
    fireEvent.contextMenu(item)

    await waitFor(() => {
      expect(commandMocks.deletePiSession).toHaveBeenCalledWith(session.path)
    })
    expect(commandMocks.listPiSessions).toHaveBeenCalledTimes(2)
  })
})
