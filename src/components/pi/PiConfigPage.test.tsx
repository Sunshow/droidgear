import { beforeEach, describe, expect, it, vi } from 'vitest'
import userEvent from '@testing-library/user-event'

const { commandMocks, toastMock } = vi.hoisted(() => {
  const toast = Object.assign(vi.fn(), {
    success: vi.fn(),
    error: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
    dismiss: vi.fn(),
    loading: vi.fn(),
  })
  return {
    toastMock: toast,
    commandMocks: {
      listPiProfiles: vi.fn(),
      createDefaultPiProfile: vi.fn(),
      getActivePiProfileId: vi.fn(),
      listPiSessions: vi.fn(),
      getPiConfigStatus: vi.fn(),
      readPiSettings: vi.fn(),
      readPiCurrentConfig: vi.fn(),
      savePiSettings: vi.fn(),
      savePiProfile: vi.fn(),
      applyPiProfile: vi.fn(),
    },
  }
})

vi.mock('@/lib/bindings', () => ({
  commands: commandMocks,
}))

vi.mock('sonner', () => ({
  toast: toastMock,
}))

import { render, screen, waitFor, within } from '@/test/test-utils'
import { PiConfigPage } from './PiConfigPage'

vi.stubGlobal(
  'ResizeObserver',
  class ResizeObserver {
    observe = vi.fn()
    unobserve = vi.fn()
    disconnect = vi.fn()
  }
)

const profile = {
  id: 'p1',
  name: 'Default',
  description: null,
  createdAt: '2024-01-01T00:00:00Z',
  updatedAt: '2024-01-01T00:00:00Z',
  providers: {},
}

describe('PiConfigPage default model', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    commandMocks.listPiProfiles.mockResolvedValue({
      status: 'ok',
      data: [profile],
    })
    commandMocks.createDefaultPiProfile.mockResolvedValue({
      status: 'ok',
      data: profile,
    })
    commandMocks.getActivePiProfileId.mockResolvedValue({
      status: 'ok',
      data: 'p1',
    })
    commandMocks.getPiConfigStatus.mockResolvedValue({
      status: 'ok',
      data: {
        configExists: true,
        configPath: '/home/test/.pi/agent/models.json',
      },
    })
    commandMocks.readPiSettings.mockResolvedValue({
      status: 'ok',
      data: {
        defaultProvider: null,
        defaultModel: null,
        defaultThinkingLevel: null,
        settingsExists: false,
        settingsPath: '/home/test/.pi/agent/settings.json',
      },
    })
    commandMocks.readPiCurrentConfig.mockResolvedValue({
      status: 'ok',
      data: { providers: {} },
    })
    commandMocks.savePiSettings.mockResolvedValue({ status: 'ok', data: null })
    commandMocks.savePiProfile.mockResolvedValue({ status: 'ok', data: null })
    commandMocks.applyPiProfile.mockResolvedValue({
      status: 'ok',
      data: {
        id: 'p1',
        appliedProviders: [],
        startup: {
          defaultProvider: null,
          defaultModel: null,
          defaultThinkingLevel: null,
        },
        startupAdjusted: false,
        startupReasons: [],
      },
    })
  })

  it('writes the provider/model pair into settings.json', async () => {
    const user = userEvent.setup({ delay: null })
    render(<PiConfigPage />)

    const providerInput = await screen.findByPlaceholderText(/wududu/)
    const modelInput = screen.getByPlaceholderText(/deepseek-v4\.1-flash/)
    await user.type(providerInput, 'wududu')
    await user.type(modelInput, 'deepseek-v4.1-flash')

    await user.click(
      screen.getByRole('button', { name: /默认模型|default model/i })
    )

    await waitFor(() => {
      expect(commandMocks.savePiSettings).toHaveBeenCalledWith({
        defaultProvider: 'wududu',
        defaultModel: 'deepseek-v4.1-flash',
        defaultThinkingLevel: null,
      })
    })
  })

  it('clears the keys when both fields are emptied (pi picks automatically)', async () => {
    commandMocks.readPiSettings.mockResolvedValue({
      status: 'ok',
      data: {
        defaultProvider: 'wududu',
        defaultModel: 'deepseek-v4.1-flash',
        defaultThinkingLevel: 'high',
        settingsExists: true,
        settingsPath: '/home/test/.pi/agent/settings.json',
      },
    })
    const user = userEvent.setup({ delay: null })
    render(<PiConfigPage />)

    const providerInput = await screen.findByPlaceholderText(/wududu/)
    await waitFor(() => expect(providerInput).toHaveValue('wududu'))
    await user.clear(providerInput)
    await user.clear(screen.getByPlaceholderText(/deepseek-v4\.1-flash/))

    await user.click(
      screen.getByRole('button', { name: /默认模型|default model/i })
    )

    await waitFor(() => {
      expect(commandMocks.savePiSettings).toHaveBeenCalledWith({
        defaultProvider: null,
        defaultModel: null,
        defaultThinkingLevel: 'high',
      })
    })
  })
  it('offers the profile/live ids in a picker while keeping free text', async () => {
    commandMocks.readPiCurrentConfig.mockResolvedValue({
      status: 'ok',
      data: {
        providers: {
          wududu: { models: [{ id: 'deepseek-v4.1-flash' }] },
        },
      },
    })
    const user = userEvent.setup({ delay: null })
    render(<PiConfigPage />)

    const providerInput = await screen.findByPlaceholderText(/wududu/)
    await user.click(
      screen.getByRole('button', {
        name: /从列表选择 Provider|choose a provider from the list/i,
      })
    )
    await user.click(await screen.findByRole('option', { name: 'wududu' }))
    await waitFor(() => expect(providerInput).toHaveValue('wududu'))

    await user.click(
      screen.getByRole('button', {
        name: /从列表选择模型 id|choose a model id from the list/i,
      })
    )
    await user.click(
      await screen.findByRole('option', { name: 'deepseek-v4.1-flash' })
    )
    await waitFor(() =>
      expect(screen.getByPlaceholderText(/deepseek-v4\.1-flash/)).toHaveValue(
        'deepseek-v4.1-flash'
      )
    )
  })

  it('reports a startup model repaired by the apply', async () => {
    commandMocks.applyPiProfile.mockResolvedValue({
      status: 'ok',
      data: {
        id: 'p1',
        appliedProviders: ['corp-proxy'],
        startup: {
          defaultProvider: 'corp-proxy',
          defaultModel: 'corp-large',
          defaultThinkingLevel: null,
        },
        startupAdjusted: true,
        startupReasons: [
          'provider "wududu" is no longer supplied by models.json',
        ],
      },
    })
    const user = userEvent.setup({ delay: null })
    render(<PiConfigPage />)

    await user.click(
      await screen.findByRole('button', { name: /应用配置|apply profile/i })
    )
    const dialog = await screen.findByRole('alertdialog')
    await user.click(
      within(dialog).getByRole('button', { name: /应用配置|apply profile/i })
    )

    await waitFor(() => {
      expect(commandMocks.applyPiProfile).toHaveBeenCalledWith('p1')
    })
    await waitFor(() => {
      expect(toastMock.success).toHaveBeenCalledWith(
        expect.stringContaining('corp-proxy / corp-large')
      )
    })
  })
})
