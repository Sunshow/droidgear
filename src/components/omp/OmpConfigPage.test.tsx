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
      listOmpProfiles: vi.fn(),
      getActiveOmpProfileId: vi.fn(),
      getOmpConfigStatus: vi.fn(),
      readOmpCurrentConfig: vi.fn(),
      readOmpCustomProviders: vi.fn(),
      listOmpRoleOptions: vi.fn(),
      saveOmpProfile: vi.fn(),
      applyOmpProfile: vi.fn(),
      saveOmpCustomProvider: vi.fn(),
      deleteOmpCustomProvider: vi.fn(),
      testOmpProviderConnection: vi.fn(),
    },
  }
})

vi.mock('@/lib/bindings', () => ({
  commands: commandMocks,
}))

vi.mock('sonner', () => ({
  toast: toastMock,
}))

import { render, screen, waitFor } from '@/test/test-utils'
import { OmpConfigPage } from './OmpConfigPage'

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
  modelRoles: {},
}

describe('OmpConfigPage model roles', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    commandMocks.listOmpProfiles.mockResolvedValue({
      status: 'ok',
      data: [profile],
    })
    commandMocks.getActiveOmpProfileId.mockResolvedValue({
      status: 'ok',
      data: 'p1',
    })
    commandMocks.getOmpConfigStatus.mockResolvedValue({
      status: 'ok',
      data: {
        configExists: true,
        configPath: '/home/test/.omp/agent/config.yml',
        modelsDBExists: true,
        agentDBExists: true,
      },
    })
    commandMocks.readOmpCurrentConfig.mockResolvedValue({
      status: 'ok',
      data: {
        agentConfig: { modelRoles: {} },
        providerModels: [
          { providerId: 'proxy', models: [{ id: 'm1' }, { id: 'm2' }] },
        ],
        credentials: [],
      },
    })
    commandMocks.readOmpCustomProviders.mockResolvedValue({
      status: 'ok',
      data: {
        providers: [
          { providerId: 'local', config: { models: [{ id: 'l1' }] } },
        ],
      },
    })
    commandMocks.listOmpRoleOptions.mockResolvedValue({
      status: 'ok',
      data: {
        providers: ['local', 'proxy'],
        models: { local: ['l1'], proxy: ['m1', 'm2'] },
      },
    })
    commandMocks.saveOmpProfile.mockResolvedValue({ status: 'ok', data: null })
    commandMocks.applyOmpProfile.mockResolvedValue({
      status: 'ok',
      data: { id: 'p1', roles: {}, warnings: [] },
    })
  })

  it('offers models.yml and cached ids while keeping free text', async () => {
    const user = userEvent.setup({ delay: null })
    render(<OmpConfigPage />)

    const defaultRoleInput = (
      await screen.findAllByPlaceholderText('provider/model-id')
    )[0]
    expect(defaultRoleInput).toHaveValue('')

    await user.click(
      screen.getByRole('button', {
        name: /从列表选择模型 · default|choose a model from the list · default/i,
      })
    )
    await user.click(await screen.findByRole('option', { name: 'proxy/m2' }))

    await waitFor(() => expect(defaultRoleInput).toHaveValue('proxy/m2'))
    // The picker covers models.yml providers too, not just the cache.
    await user.click(
      screen.getByRole('button', {
        name: /从列表选择模型 · default|choose a model from the list · default/i,
      })
    )
    expect(await screen.findByRole('option', { name: 'local/l1' })).toBeTruthy()
  })

  it('warns about roles the applied profile cannot resolve', async () => {
    commandMocks.applyOmpProfile.mockResolvedValue({
      status: 'ok',
      data: {
        id: 'p1',
        roles: { default: 'gone/m1' },
        warnings: ['modelRoles.default points at "gone/m1"'],
      },
    })
    const user = userEvent.setup({ delay: null })
    render(<OmpConfigPage />)

    await user.click(
      await screen.findByRole('button', { name: /应用配置|apply profile/i })
    )

    await waitFor(() => {
      expect(commandMocks.applyOmpProfile).toHaveBeenCalledWith('p1')
    })
    await waitFor(() => {
      expect(toastMock.warning).toHaveBeenCalledWith(
        expect.stringContaining('gone/m1')
      )
    })
    expect(toastMock.success).not.toHaveBeenCalled()
  })
})
