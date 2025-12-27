import { render, screen } from '@/test/test-utils'
import { describe, it, expect } from 'vitest'
import App from './App'

// Tauri bindings are mocked globally in src/test/setup.ts

describe('App', () => {
  it('renders main window layout', () => {
    render(<App />)
    // Default view is now channels, check for channel navigation buttons
    expect(
      screen.getByRole('button', { name: /Channels/i })
    ).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /Droid/i })).toBeInTheDocument()
  })

  it('renders title bar with app name', () => {
    render(<App />)
    expect(screen.getByText('DroidGear')).toBeInTheDocument()
  })
})
