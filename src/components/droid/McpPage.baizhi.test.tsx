import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from '@testing-library/react'
import { McpPage } from './McpPage'
import { commands, type McpServer } from '@/lib/bindings'

const { toastError } = vi.hoisted(() => ({ toastError: vi.fn() }))
vi.mock('sonner', () => ({
  toast: { error: toastError, success: vi.fn() },
}))

vi.mock('react-i18next', () => {
  const t = (key: string) => key
  return { useTranslation: () => ({ t }) }
})
vi.mock('@/lib/bindings', () => ({
  commands: {
    loadMcpServers: vi.fn(),
    saveMcpServer: vi.fn(),
    saveBaizhiPreset: vi.fn(),
    toggleMcpServer: vi.fn(),
    deleteMcpServer: vi.fn(),
  },
}))

const existing: McpServer = {
  name: 'baizhi',
  config: { type: 'http', disabled: false, url: 'https://example.test/custom' },
}

function presetButton(title: HTMLElement) {
  const card = title.parentElement?.parentElement
  if (!card) throw new Error('Expected a preset card')
  return within(card).getByRole('button')
}

async function openBaizhi() {
  render(<McpPage />)
  const title = await screen.findByText('百智工具包（Baizhi MCP）')
  fireEvent.mouseDown(presetButton(title))
  return screen.findByRole('dialog')
}

describe('Baizhi Quick Add', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(commands.loadMcpServers).mockResolvedValue({
      status: 'ok',
      data: [],
    })
    vi.mocked(commands.saveMcpServer).mockResolvedValue({
      status: 'ok',
      data: null,
    })
    vi.mocked(commands.saveBaizhiPreset).mockResolvedValue({
      status: 'ok',
      data: null,
    })
  })

  it('selection and cancel never save or ask for a key', async () => {
    const dialog = await openBaizhi()
    expect(
      within(dialog).getByText('https://agent-toolkit.app.baizhi.cloud/mcp')
    ).toBeInTheDocument()
    expect(
      within(dialog).getByText('Authorization: Bearer ${BAIZHI_API_KEY}')
    ).toBeInTheDocument()
    expect(within(dialog).queryByRole('textbox')).not.toBeInTheDocument()
    expect(commands.saveBaizhiPreset).not.toHaveBeenCalled()
    fireEvent.mouseDown(
      within(dialog).getByRole('button', { name: 'common.cancel' })
    )
    expect(commands.saveBaizhiPreset).not.toHaveBeenCalled()
  })

  it('explicit confirmation invokes only the no-argument brand command', async () => {
    const dialog = await openBaizhi()
    fireEvent.mouseDown(
      within(dialog).getByRole('button', {
        name: 'mcp.presets.baizhi.saveDisabled',
      })
    )
    await waitFor(() =>
      expect(commands.saveBaizhiPreset).toHaveBeenCalledWith()
    )
    expect(commands.saveMcpServer).not.toHaveBeenCalled()
    expect(commands.toggleMcpServer).not.toHaveBeenCalled()
  })

  it.each(['Enter', ' '])(
    'opens and cancels with %s without saving',
    async key => {
      render(<McpPage />)
      const title = await screen.findByText('百智工具包（Baizhi MCP）')
      expect(fireEvent.keyDown(presetButton(title), { key })).toBe(false)
      const dialog = await screen.findByRole('dialog')
      expect(commands.saveBaizhiPreset).not.toHaveBeenCalled()
      expect(within(dialog).queryByRole('textbox')).not.toBeInTheDocument()
      expect(
        fireEvent.keyDown(
          within(dialog).getByRole('button', { name: 'common.cancel' }),
          { key }
        )
      ).toBe(false)
      await waitFor(() =>
        expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
      )
      expect(commands.saveBaizhiPreset).not.toHaveBeenCalled()
      expect(commands.saveMcpServer).not.toHaveBeenCalled()
    }
  )

  it.each(['Enter', ' '])(
    'confirms with %s through only the no-argument brand command',
    async key => {
      const dialog = await openBaizhi()
      expect(
        fireEvent.keyDown(
          within(dialog).getByRole('button', {
            name: 'mcp.presets.baizhi.saveDisabled',
          }),
          { key }
        )
      ).toBe(false)
      await waitFor(() =>
        expect(commands.saveBaizhiPreset).toHaveBeenCalledExactlyOnceWith()
      )
      expect(commands.saveMcpServer).not.toHaveBeenCalled()
      expect(commands.toggleMcpServer).not.toHaveBeenCalled()
    }
  )

  it.each(['load', 'save'])(
    'handles a %s IPC rejection without exposing its contents',
    async failingCommand => {
      const dialog = await openBaizhi()
      const error = new Error('synthetic-sensitive-error-detail')
      if (failingCommand === 'load') {
        vi.mocked(commands.loadMcpServers).mockRejectedValueOnce(error)
      } else {
        vi.mocked(commands.saveBaizhiPreset).mockRejectedValueOnce(error)
      }
      fireEvent.mouseDown(
        within(dialog).getByRole('button', {
          name: 'mcp.presets.baizhi.saveDisabled',
        })
      )
      await waitFor(() =>
        expect(toastError).toHaveBeenCalledExactlyOnceWith(
          'mcp.presets.baizhi.saveFailed'
        )
      )
      expect(screen.queryByText(error.message)).not.toBeInTheDocument()
      expect(
        within(dialog).getByRole('button', {
          name: 'mcp.presets.baizhi.saveDisabled',
        })
      ).not.toBeDisabled()
      expect(commands.saveMcpServer).not.toHaveBeenCalled()
      expect(commands.toggleMcpServer).not.toHaveBeenCalled()
      if (failingCommand === 'load') {
        expect(commands.saveBaizhiPreset).not.toHaveBeenCalled()
      }
    }
  )

  it.each(['load', 'save'])(
    'handles a %s Rust error status without exposing its contents',
    async failingCommand => {
      const dialog = await openBaizhi()
      const error = 'synthetic-sensitive-rust-error-detail'
      if (failingCommand === 'load') {
        vi.mocked(commands.loadMcpServers).mockResolvedValueOnce({
          status: 'error',
          error,
        })
      } else {
        vi.mocked(commands.saveBaizhiPreset).mockResolvedValueOnce({
          status: 'error',
          error,
        })
      }
      fireEvent.mouseDown(
        within(dialog).getByRole('button', {
          name: 'mcp.presets.baizhi.saveDisabled',
        })
      )
      await waitFor(() =>
        expect(toastError).toHaveBeenCalledExactlyOnceWith(
          'mcp.presets.baizhi.saveFailed'
        )
      )
      expect(screen.queryByText(error)).not.toBeInTheDocument()
      expect(
        within(dialog).getByRole('button', {
          name: 'mcp.presets.baizhi.saveDisabled',
        })
      ).not.toBeDisabled()
      expect(commands.saveMcpServer).not.toHaveBeenCalled()
      if (failingCommand === 'load') {
        expect(commands.saveBaizhiPreset).not.toHaveBeenCalled()
      }
    }
  )

  it.each(['reject', 'error-status'])(
    'handles refresh %s after saving once without claiming a rollback',
    async failure => {
      const dialog = await openBaizhi()
      const error = 'synthetic-sensitive-refresh-detail'
      vi.mocked(commands.loadMcpServers).mockResolvedValueOnce({
        status: 'ok',
        data: [],
      })
      if (failure === 'reject') {
        vi.mocked(commands.loadMcpServers).mockRejectedValueOnce(
          new Error(error)
        )
      } else {
        vi.mocked(commands.loadMcpServers).mockResolvedValueOnce({
          status: 'error',
          error,
        })
      }
      fireEvent.mouseDown(
        within(dialog).getByRole('button', {
          name: 'mcp.presets.baizhi.saveDisabled',
        })
      )
      await waitFor(() =>
        expect(toastError).toHaveBeenCalledExactlyOnceWith(
          'mcp.presets.baizhi.saveFailed'
        )
      )
      expect(commands.saveBaizhiPreset).toHaveBeenCalledExactlyOnceWith()
      expect(commands.loadMcpServers).toHaveBeenCalledTimes(3)
      expect(commands.saveMcpServer).not.toHaveBeenCalled()
      expect(screen.queryByText(error)).not.toBeInTheDocument()
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
      fireEvent.keyDown(
        presetButton(screen.getByText('百智工具包（Baizhi MCP）')),
        { key: 'Enter' }
      )
      const reopened = await screen.findByRole('dialog')
      expect(
        within(reopened).getByRole('button', {
          name: 'mcp.presets.baizhi.saveDisabled',
        })
      ).not.toBeDisabled()
      fireEvent.keyDown(
        within(reopened).getByRole('button', { name: 'common.cancel' }),
        { key: 'Escape' }
      )
      expect(commands.saveBaizhiPreset).toHaveBeenCalledTimes(1)
    }
  )

  it('does not overwrite an entry added after the card was displayed', async () => {
    const dialog = await openBaizhi()
    vi.mocked(commands.loadMcpServers).mockResolvedValue({
      status: 'ok',
      data: [existing],
    })
    fireEvent.mouseDown(
      within(dialog).getByRole('button', {
        name: 'mcp.presets.baizhi.saveDisabled',
      })
    )
    await waitFor(() =>
      expect(commands.loadMcpServers).toHaveBeenCalledTimes(2)
    )
    expect(commands.saveBaizhiPreset).not.toHaveBeenCalled()
  })

  it('keeps the existing Playwright preset on the ordinary save command', async () => {
    render(<McpPage />)
    const title = await screen.findByText('Playwright')
    fireEvent.click(presetButton(title))
    await waitFor(() =>
      expect(commands.saveMcpServer).toHaveBeenCalledWith({
        name: 'playwright',
        config: {
          type: 'stdio',
          disabled: false,
          command: 'npx',
          args: ['@playwright/mcp@latest'],
        },
      })
    )
    expect(commands.saveBaizhiPreset).not.toHaveBeenCalled()
  })

  it('keeps Context7 key input on the existing ordinary preset path', async () => {
    render(<McpPage />)
    const title = await screen.findByText('Context7')
    fireEvent.click(presetButton(title))
    const dialog = await screen.findByRole('dialog')
    fireEvent.change(within(dialog).getByRole('textbox'), {
      target: { value: 'synthetic-context7' },
    })
    fireEvent.click(within(dialog).getByRole('button', { name: 'common.save' }))
    await waitFor(() =>
      expect(commands.saveMcpServer).toHaveBeenCalledWith({
        name: 'context7',
        config: {
          type: 'stdio',
          disabled: false,
          command: 'npx',
          args: [
            '-y',
            '@upstash/context7-mcp',
            '--api-key',
            'synthetic-context7',
          ],
        },
      })
    )
    expect(commands.saveBaizhiPreset).not.toHaveBeenCalled()
  })
})
