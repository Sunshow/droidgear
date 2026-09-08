import { beforeEach, describe, expect, it, vi } from 'vitest'
import { fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'

const { commandMocks, showContextMenuMock } = vi.hoisted(() => ({
  showContextMenuMock: vi.fn(),
  commandMocks: {
    listCodexSessions: vi.fn(),
    listCodexSessionProviders: vi.fn(),
    getCodexSessionDetail: vi.fn(),
    deleteCodexSession: vi.fn(),
    setCodexSessionProvider: vi.fn(),
    startCodexSessionsWatcher: vi.fn(),
    stopCodexSessionsWatcher: vi.fn(),
  },
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => undefined),
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
import { CodexSessionsPage } from './CodexSessionsPage'

// jsdom does not implement element scrolling (used by the follow-mode logic)
Object.defineProperty(Element.prototype, 'scrollTo', {
  writable: true,
  configurable: true,
  value: vi.fn(),
})

const sampleTokenUsage = {
  inputTokens: 100,
  outputTokens: 20,
  cacheCreationTokens: 5,
  cacheReadTokens: 10,
  reasoningTokens: 30,
  totalTokens: 165,
}

const sampleSessions = [
  {
    id: 'sess-1',
    title: 'Fix the bug',
    project: '/work/repo',
    model: 'gpt-5',
    modelProvider: 'openai',
    modifiedAt: 1700000000000,
    tokenUsage: sampleTokenUsage,
    path: '/home/user/.codex/sessions/2026/09/08/rollout-a.jsonl',
  },
  {
    id: 'sess-2',
    title: 'New Session',
    project: '/work/other',
    model: 'gpt-5',
    modelProvider: 'openai',
    modifiedAt: 1600000000000,
    tokenUsage: { ...sampleTokenUsage, inputTokens: 0, outputTokens: 0 },
    path: '/home/user/.codex/sessions/2026/09/07/rollout-b.jsonl',
  },
]

const sampleProviders = [
  { id: 'openai', name: 'OpenAI' },
  { id: 'deepseek', name: 'DeepSeek' },
]

const sampleDetail = {
  id: 'sess-1',
  title: 'Fix the bug',
  cwd: '/work/repo',
  model: 'gpt-5',
  modelProvider: 'openai',
  modifiedAt: 1700000000000,
  tokenUsage: sampleTokenUsage,
  messages: [
    {
      id: 'm1',
      role: 'user',
      content: [{ type: 'text', text: 'Fix the bug' }],
      timestamp: '2026-09-08T00:00:04Z',
    },
    {
      id: 'm2',
      role: 'assistant',
      content: [
        { type: 'thinking', thinking: 'I should check the parser.' },
        { type: 'text', text: 'Done **now**.' },
      ],
      timestamp: '2026-09-08T00:00:07Z',
    },
  ],
}

describe('CodexSessionsPage', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    localStorage.clear()

    commandMocks.listCodexSessions.mockResolvedValue({
      status: 'ok',
      data: sampleSessions,
    })
    commandMocks.listCodexSessionProviders.mockResolvedValue({
      status: 'ok',
      data: sampleProviders,
    })
    commandMocks.getCodexSessionDetail.mockResolvedValue({
      status: 'ok',
      data: sampleDetail,
    })
    commandMocks.deleteCodexSession.mockResolvedValue({
      status: 'ok',
      data: null,
    })
    commandMocks.setCodexSessionProvider.mockResolvedValue({
      status: 'ok',
      data: null,
    })
    commandMocks.startCodexSessionsWatcher.mockResolvedValue({
      status: 'ok',
      data: null,
    })
    commandMocks.stopCodexSessionsWatcher.mockResolvedValue({
      status: 'ok',
      data: null,
    })
    showContextMenuMock.mockResolvedValue(undefined)
  })

  it('lists sessions and hides empty sessions by default', async () => {
    render(<CodexSessionsPage />)

    await screen.findByText('Fix the bug')
    expect(screen.queryByText('New Session')).not.toBeInTheDocument()
    expect(commandMocks.startCodexSessionsWatcher).toHaveBeenCalled()
  })

  it('shows empty sessions when the hide toggle is off', async () => {
    localStorage.setItem('codex-sessions-hide-empty', 'false')
    render(<CodexSessionsPage />)

    await screen.findByText('New Session')
    expect(screen.getByText('Fix the bug')).toBeInTheDocument()
  })

  it('loads and renders the session detail when selected', async () => {
    const user = userEvent.setup()
    render(<CodexSessionsPage />)

    const item = await screen.findByText('Fix the bug')
    await user.click(item)

    await waitFor(() => {
      expect(commandMocks.getCodexSessionDetail).toHaveBeenCalledWith(
        '/home/user/.codex/sessions/2026/09/08/rollout-a.jsonl'
      )
    })

    // Detail header + message content
    expect(
      await screen.findByText('Fix the bug', { selector: 'h2' })
    ).toBeInTheDocument()
    expect(screen.getByText('Done **now**.')).toBeInTheDocument()
    // Thinking stays collapsed by default
    const thinking = screen.getByText('I should check the parser.')
    expect(thinking).toBeInTheDocument()
    expect(thinking.closest('details')).not.toHaveAttribute('open')
  })

  it('expands thinking blocks when the preference is set', async () => {
    localStorage.setItem('codex-sessions-expand-thinking', 'true')
    const user = userEvent.setup()
    render(<CodexSessionsPage />)

    await user.click(await screen.findByText('Fix the bug'))
    const thinking = await screen.findByText('I should check the parser.')
    await waitFor(() => {
      expect(thinking.closest('details')).toHaveAttribute('open')
    })
  })

  it('deletes a session via the context menu and reloads the list', async () => {
    showContextMenuMock.mockImplementation(
      async (items: { id: string; action: () => Promise<void> }[]) => {
        const deleteItem = items.find(item => item.id === 'delete')
        await deleteItem?.action()
      }
    )
    render(<CodexSessionsPage />)

    const item = await screen.findByText('Fix the bug')
    fireEvent.contextMenu(item)

    await waitFor(() => {
      expect(commandMocks.deleteCodexSession).toHaveBeenCalledWith(
        '/home/user/.codex/sessions/2026/09/08/rollout-a.jsonl'
      )
    })
    expect(commandMocks.listCodexSessions).toHaveBeenCalledTimes(2)
  })

  it('shows an error from the listing command', async () => {
    commandMocks.listCodexSessions.mockResolvedValue({
      status: 'error',
      error: 'cannot read sessions dir',
    })
    render(<CodexSessionsPage />)

    expect(
      await screen.findByText('cannot read sessions dir')
    ).toBeInTheDocument()
  })

  it('shows the session provider in the detail header', async () => {
    const user = userEvent.setup()
    render(<CodexSessionsPage />)

    await user.click(await screen.findByText('Fix the bug'))

    const providerButton = await screen.findByRole('button', {
      name: /OpenAI/,
    })
    expect(providerButton).toBeInTheDocument()
  })

  it('switches the session provider from the detail header', async () => {
    const user = userEvent.setup()
    render(<CodexSessionsPage />)

    await user.click(await screen.findByText('Fix the bug'))
    // Radix menus open on pointerdown (userEvent.click does not trigger it in jsdom)
    fireEvent.pointerDown(await screen.findByRole('button', { name: /OpenAI/ }))

    // Pick DeepSeek from the dropdown menu
    const deepseekItem = await screen.findByRole('menuitem', {
      name: /DeepSeek/,
    })
    await user.click(deepseekItem)

    await waitFor(() => {
      expect(commandMocks.setCodexSessionProvider).toHaveBeenCalledWith(
        '/home/user/.codex/sessions/2026/09/08/rollout-a.jsonl',
        'deepseek'
      )
    })
    // Detail and list reload after the switch
    expect(commandMocks.getCodexSessionDetail).toHaveBeenCalledTimes(2)
    expect(commandMocks.listCodexSessions).toHaveBeenCalledTimes(2)
  })

  it('does not call the switch command for the current provider', async () => {
    const user = userEvent.setup()
    render(<CodexSessionsPage />)

    await user.click(await screen.findByText('Fix the bug'))
    fireEvent.pointerDown(await screen.findByRole('button', { name: /OpenAI/ }))
    const openaiItem = await screen.findByRole('menuitem', { name: /OpenAI/ })
    await user.click(openaiItem)

    expect(commandMocks.setCodexSessionProvider).not.toHaveBeenCalled()
  })
})
