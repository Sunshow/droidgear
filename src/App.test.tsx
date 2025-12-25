import { render, screen } from '@/test/test-utils'
import { describe, it, expect } from 'vitest'
import App from './App'

// Tauri bindings are mocked globally in src/test/setup.ts

describe('App', () => {
  it('renders main window layout', () => {
    render(<App />)
    expect(
      screen.getByRole('heading', { name: /BYOK Model Configuration/i })
    ).toBeInTheDocument()
  })

  it('renders title bar with app name', () => {
    render(<App />)
    expect(screen.getByText('DroidGear')).toBeInTheDocument()
  })
})
