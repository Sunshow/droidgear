import { describe, expect, it } from 'vitest'
import { render, screen } from '@/test/test-utils'
import { SecretBadge } from './SecretBadge'

describe('SecretBadge', () => {
  it('renders the secret read-only copy', () => {
    render(<SecretBadge />)

    expect(screen.getByText('Secret (read-only)')).toBeInTheDocument()
  })
})
