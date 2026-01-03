import '@testing-library/jest-dom'
import { vi } from 'vitest'

// Mock matchMedia for tests
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(), // deprecated
    removeListener: vi.fn(), // deprecated
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
})

// Radix UI components rely on Pointer Events APIs that jsdom doesn't fully implement.
// Provide minimal shims to prevent runtime errors in tests.
if (!('hasPointerCapture' in Element.prototype)) {
  ;(
    Element.prototype as unknown as {
      hasPointerCapture: () => boolean
    }
  ).hasPointerCapture = () => false
}
if (!('setPointerCapture' in Element.prototype)) {
  ;(
    Element.prototype as unknown as {
      setPointerCapture: () => void
    }
  ).setPointerCapture = () => undefined
}
if (!('releasePointerCapture' in Element.prototype)) {
  ;(
    Element.prototype as unknown as {
      releasePointerCapture: () => void
    }
  ).releasePointerCapture = () => undefined
}

if (!('scrollIntoView' in Element.prototype)) {
  ;(
    Element.prototype as unknown as {
      scrollIntoView: () => void
    }
  ).scrollIntoView = () => undefined
}

// Mock Tauri APIs for tests
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {
    // Mock unlisten function
  }),
}))

vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn().mockReturnValue({
    onFocusChanged: vi.fn().mockResolvedValue(() => undefined),
    onCloseRequested: vi.fn().mockResolvedValue(() => undefined),
    show: vi.fn().mockResolvedValue(undefined),
    setFocus: vi.fn().mockResolvedValue(undefined),
  }),
}))

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}))

vi.mock('@tauri-apps/plugin-os', () => ({
  locale: vi.fn().mockResolvedValue('en-US'),
  platform: vi.fn().mockResolvedValue('macos'),
}))

vi.mock('@tauri-apps/api/menu', () => ({
  Menu: {
    new: vi.fn().mockResolvedValue({ setAsAppMenu: vi.fn() }),
  },
  MenuItem: {
    new: vi.fn().mockResolvedValue({}),
  },
  Submenu: {
    new: vi.fn().mockResolvedValue({}),
  },
  PredefinedMenuItem: {
    new: vi.fn().mockResolvedValue({}),
  },
}))

vi.mock('@tauri-apps/plugin-updater', () => ({
  check: vi.fn().mockResolvedValue(null),
}))

// Mock typed Tauri bindings (tauri-specta generated)
vi.mock('@/lib/tauri-bindings', () => ({
  commands: {
    greet: vi.fn().mockResolvedValue('Hello, test!'),
    loadPreferences: vi
      .fn()
      .mockResolvedValue({ status: 'ok', data: { theme: 'system' } }),
    savePreferences: vi.fn().mockResolvedValue({ status: 'ok', data: null }),
    sendNativeNotification: vi
      .fn()
      .mockResolvedValue({ status: 'ok', data: null }),
    saveEmergencyData: vi.fn().mockResolvedValue({ status: 'ok', data: null }),
    loadEmergencyData: vi.fn().mockResolvedValue({ status: 'ok', data: null }),
    cleanupOldRecoveryFiles: vi
      .fn()
      .mockResolvedValue({ status: 'ok', data: 0 }),
    checkLegacyConfig: vi.fn().mockResolvedValue({ status: 'ok', data: false }),
    deleteLegacyConfig: vi.fn().mockResolvedValue({ status: 'ok', data: null }),
  },
  unwrapResult: vi.fn((result: { status: string; data?: unknown }) => {
    if (result.status === 'ok') return result.data
    throw result
  }),
}))
