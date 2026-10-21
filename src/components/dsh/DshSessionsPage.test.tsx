import { beforeEach, describe, expect, it, vi } from 'vitest'
import { fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'

const { commandMocks, showContextMenuMock, listenMock } = vi.hoisted(() => ({
  listenMock: vi.fn(),
  showContextMenuMock: vi.fn(),
  commandMocks: {
    listDshSessions: vi.fn(),
    getDshSessionDetail: vi.fn(),
    deleteDshSession: vi.fn(),
    startDshSessionsWatcher: vi.fn(),
    stopDshSessionsWatcher: vi.fn(),
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
import { DshSessionsPage } from './DshSessionsPage'

Object.defineProperty(Element.prototype, 'scrollTo', {
  writable: true,
  configurable: true,
  value: vi.fn(),
})

const session = {
  id: 'session-dsh-1',
  title: 'Inspect the parser',
  project: '/work/demo',
  model: 'deepseek-flash',
  modelProvider: 'deepseek-official',
  modifiedAt: 1700000000000,
  tokenUsage: {
    inputTokens: 100,
    outputTokens: 20,
    cacheReadTokens: 0,
    cacheCreationTokens: 0,
    totalTokens: 120,
  },
  path: '/home/user/.dsh/sessions/--work-demo--/session-dsh-1/session.v4.jsonl.zstd',
}

const detail = {
  summary: session,
  messages: [
    {
      id: 'message-1',
      role: 'user',
      content: [{ type: 'text', text: 'Inspect the parser' }],
      timestamp: '2026-01-01T00:00:00Z',
    },
    {
      id: 'message-2',
      role: 'assistant',
      content: [{ type: 'text', text: 'The parser is working.' }],
      timestamp: '2026-01-01T00:00:01Z',
    },
  ],
}

describe('DshSessionsPage', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    localStorage.clear()
    listenMock.mockResolvedValue(() => undefined)
    commandMocks.listDshSessions.mockResolvedValue({
      status: 'ok',
      data: [session],
    })
    commandMocks.getDshSessionDetail.mockResolvedValue({
      status: 'ok',
      data: detail,
    })
    commandMocks.deleteDshSession.mockResolvedValue({
      status: 'ok',
      data: null,
    })
    commandMocks.startDshSessionsWatcher.mockResolvedValue({
      status: 'ok',
      data: null,
    })
    commandMocks.stopDshSessionsWatcher.mockResolvedValue({
      status: 'ok',
      data: null,
    })
    showContextMenuMock.mockResolvedValue(undefined)
  })

  it('lists sessions and starts the watcher', async () => {
    render(<DshSessionsPage />)

    expect(await screen.findByText('Inspect the parser')).toBeInTheDocument()
    expect(commandMocks.startDshSessionsWatcher).toHaveBeenCalled()
  })

  it('reloads the list when the watcher reports a change', async () => {
    render(<DshSessionsPage />)
    await screen.findByText('Inspect the parser')

    commandMocks.listDshSessions.mockResolvedValue({
      status: 'ok',
      data: [{ ...session, title: 'Updated session' }],
    })
    const onChange = listenMock.mock.calls[0]?.[1] as (() => void) | undefined
    onChange?.()

    expect(await screen.findByText('Updated session')).toBeInTheDocument()
    expect(commandMocks.listDshSessions).toHaveBeenCalledTimes(2)
  })

  it('loads the selected session detail', async () => {
    const user = userEvent.setup()
    render(<DshSessionsPage />)

    await user.click(await screen.findByText('Inspect the parser'))

    await waitFor(() => {
      expect(commandMocks.getDshSessionDetail).toHaveBeenCalledWith(
        session.path
      )
    })
    expect(
      await screen.findByText('The parser is working.')
    ).toBeInTheDocument()
  })

  it('groups consecutive thinking-only messages into one collapsible block', async () => {
    const user = userEvent.setup()
    commandMocks.getDshSessionDetail.mockResolvedValue({
      status: 'ok',
      data: {
        summary: session,
        messages: [
          {
            id: 'think-1',
            role: 'assistant',
            content: [{ type: 'thinking', thinking: 'First thought' }],
            timestamp: '2026-01-01T00:00:00Z',
          },
          {
            id: 'think-2',
            role: 'assistant',
            content: [{ type: 'thinking', thinking: 'Second thought' }],
            timestamp: '2026-01-01T00:00:01Z',
          },
          {
            id: 'answer-1',
            role: 'assistant',
            content: [{ type: 'text', text: 'Visible answer' }],
            timestamp: '2026-01-01T00:00:02Z',
          },
        ],
      },
    })
    render(<DshSessionsPage />)

    await user.click(await screen.findByText('Inspect the parser'))

    const groupSummary = await screen.findByText('Thinking process (2)')
    const details = groupSummary.closest('details')
    expect(details).not.toBeNull()
    expect(details?.open).toBe(false)
    expect(screen.queryByText('Thinking process (1)')).not.toBeInTheDocument()

    await user.click(groupSummary)
    expect(details?.open).toBe(true)
    expect(screen.getByText('First thought')).toBeInTheDocument()
    expect(screen.getByText('Second thought')).toBeInTheDocument()
    expect(screen.getByTestId('streamdown')).toHaveTextContent('Visible answer')
  })

  it('deletes a session through the context menu', async () => {
    showContextMenuMock.mockImplementation(
      async (items: { id: string; action: () => Promise<void> }[]) => {
        await items.find(item => item.id === 'delete')?.action()
      }
    )
    render(<DshSessionsPage />)

    const item = await screen.findByText('Inspect the parser')
    fireEvent.contextMenu(item)

    await waitFor(() => {
      expect(commandMocks.deleteDshSession).toHaveBeenCalledWith(session.path)
    })
    expect(commandMocks.listDshSessions).toHaveBeenCalledTimes(2)
  })
})
