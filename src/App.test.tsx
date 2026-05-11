import { render, screen, waitFor } from '@/test/test-utils'
import { describe, it, expect } from 'vitest'
import App from './App'

// Tauri bindings are mocked globally in src/test/setup.ts

describe('App', () => {
  it('renders main window layout', async () => {
    render(<App />)
    // Let async effects (ResizableDialog centering, etc.) settle
    await waitFor(() => {
      expect(
        screen.getByRole('button', { name: /Channels/i })
      ).toBeInTheDocument()
    })
    // Use getAllByRole to find the Droid dropdown trigger (the first match)
    const droidButtons = screen.getAllByRole('button', { name: /Droid/i })
    expect(droidButtons.length).toBeGreaterThanOrEqual(1)
  })

  it('renders title bar with app name', async () => {
    render(<App />)
    await waitFor(() => {
      expect(screen.getByText('DroidGear')).toBeInTheDocument()
    })
  })
})
