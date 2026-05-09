import { beforeEach, describe, expect, it, vi } from 'vitest'
import userEvent from '@testing-library/user-event'

const { toastMock, writeTextMock } = vi.hoisted(() => {
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
    writeTextMock: vi.fn().mockResolvedValue(undefined),
  }
})

vi.mock('sonner', () => ({
  toast: toastMock,
}))

vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({
  writeText: writeTextMock,
}))

import { render, screen, waitFor } from '@/test/test-utils'
import { commands } from '@/lib/tauri-bindings'
import { useModelStore } from '@/store/model-store'
import { useUIStore } from '@/store/ui-store'
import { DroidFeatureList } from './DroidFeatureList'

describe('DroidFeatureList', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    localStorage.clear()

    useUIStore.setState({
      currentView: 'droid',
      lastToolView: 'droid',
      droidSubView: 'models',
      leftSidebarVisible: true,
      rightSidebarVisible: false,
      commandPaletteOpen: false,
      preferencesOpen: false,
      lastSpecExportPath: null,
      pendingUpdate: null,
      droidSettingsScrollTarget: null,
      droidRefreshKey: 0,
    })
    useModelStore.setState(useModelStore.getInitialState())

    vi.mocked(commands.listDroidSettingsFiles).mockResolvedValue({
      status: 'ok',
      data: [
        {
          name: 'Global',
          path: '/home/user/.factory/settings.json',
          isGlobal: true,
          isActive: true,
          exists: true,
        },
      ],
    })
    vi.mocked(commands.launchDroid).mockResolvedValue({
      status: 'ok',
      data: null,
    })
    vi.mocked(commands.getDroidLaunchCommand).mockResolvedValue({
      status: 'ok',
      data: ['droid --settings "/home/user/.factory/settings.json"', ''],
    })
  })

  it('launches Droid directly when the terminal launch succeeds', async () => {
    const user = userEvent.setup()
    render(<DroidFeatureList />)

    const launchButton = await screen.findByTitle(
      'Open Droid CLI in a new terminal window'
    )
    await user.click(launchButton)

    await waitFor(() => {
      expect(commands.launchDroid).toHaveBeenCalledTimes(1)
    })
    expect(commands.getDroidLaunchCommand).not.toHaveBeenCalled()
    expect(writeTextMock).not.toHaveBeenCalled()
    expect(toastMock.info).not.toHaveBeenCalled()
    expect(toastMock.error).not.toHaveBeenCalled()
  })

  it('saves pending model changes before launching Droid', async () => {
    const user = userEvent.setup()
    const saveModels = vi.fn(async () => {
      useModelStore.setState({ hasChanges: false })
    })
    useModelStore.setState({
      hasChanges: true,
      saveModels,
    })

    render(<DroidFeatureList />)

    const launchButton = await screen.findByTitle(
      'Open Droid CLI in a new terminal window'
    )
    await user.click(launchButton)

    await waitFor(() => {
      expect(saveModels).toHaveBeenCalledTimes(1)
      expect(commands.launchDroid).toHaveBeenCalledTimes(1)
    })
  })

  it('switches back to Models and shows an error when saving model changes hits a config parse error', async () => {
    const user = userEvent.setup()
    useUIStore.setState({ droidSubView: 'settings' })
    const saveModels = vi.fn(async () => {
      useModelStore.setState({
        configParseError: 'CONFIG_PARSE_ERROR: invalid json',
      })
    })
    useModelStore.setState({
      hasChanges: true,
      saveModels,
    })

    render(<DroidFeatureList />)

    const launchButton = await screen.findByTitle(
      'Open Droid CLI in a new terminal window'
    )
    await user.click(launchButton)

    await waitFor(() => {
      expect(saveModels).toHaveBeenCalledTimes(1)
    })
    expect(commands.launchDroid).not.toHaveBeenCalled()
    expect(commands.getDroidLaunchCommand).not.toHaveBeenCalled()
    expect(useUIStore.getState().droidSubView).toBe('models')
    expect(toastMock.error).toHaveBeenCalledWith(
      'Could not launch Droid because model changes could not be saved'
    )
    expect(writeTextMock).not.toHaveBeenCalled()
  })

  it('shows a fallback error before aborting launch when model saving still fails without details', async () => {
    const user = userEvent.setup()
    useUIStore.setState({ droidSubView: 'mcp' })
    const saveModels = vi.fn().mockResolvedValue(undefined)
    useModelStore.setState({
      hasChanges: true,
      saveModels,
    })

    render(<DroidFeatureList />)

    const launchButton = await screen.findByTitle(
      'Open Droid CLI in a new terminal window'
    )
    await user.click(launchButton)

    await waitFor(() => {
      expect(saveModels).toHaveBeenCalledTimes(1)
    })
    expect(commands.launchDroid).not.toHaveBeenCalled()
    expect(commands.getDroidLaunchCommand).not.toHaveBeenCalled()
    expect(useUIStore.getState().droidSubView).toBe('models')
    expect(toastMock.error).toHaveBeenCalledWith(
      'Could not launch Droid because model changes could not be saved'
    )
  })

  it('copies the legacy launch command when direct launch fails', async () => {
    const user = userEvent.setup()
    vi.mocked(commands.launchDroid).mockResolvedValue({
      status: 'error',
      error: 'launch failed',
    })
    vi.mocked(commands.getDroidLaunchCommand).mockResolvedValue({
      status: 'ok',
      data: ['droid --settings "/tmp/runtime/droid/temporary-run.json"', ''],
    })

    render(<DroidFeatureList />)

    const launchButton = await screen.findByTitle(
      'Open Droid CLI in a new terminal window'
    )
    await user.click(launchButton)

    await waitFor(() => {
      expect(commands.getDroidLaunchCommand).toHaveBeenCalledTimes(1)
      expect(writeTextMock).toHaveBeenCalledWith(
        'droid --settings "/tmp/runtime/droid/temporary-run.json"'
      )
    })
    expect(toastMock.info).toHaveBeenCalledWith(
      'Command copied to clipboard: droid --settings "/tmp/runtime/droid/temporary-run.json"'
    )
    expect(toastMock.error).not.toHaveBeenCalled()
  })

  it('shows a generic error when both launch and fallback command retrieval fail', async () => {
    const user = userEvent.setup()
    vi.mocked(commands.launchDroid).mockResolvedValue({
      status: 'error',
      error: 'launch failed',
    })
    vi.mocked(commands.getDroidLaunchCommand).mockResolvedValue({
      status: 'error',
      error: 'command unavailable',
    })

    render(<DroidFeatureList />)

    const launchButton = await screen.findByTitle(
      'Open Droid CLI in a new terminal window'
    )
    await user.click(launchButton)

    await waitFor(() => {
      expect(toastMock.error).toHaveBeenCalledWith('Something went wrong')
    })
    expect(writeTextMock).not.toHaveBeenCalled()
  })
})
