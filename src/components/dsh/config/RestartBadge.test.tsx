import { describe, expect, it } from 'vitest'
import { render, screen } from '@/test/test-utils'
import { RestartBadge } from './RestartBadge'

describe('RestartBadge', () => {
  it('renders the restart hint copy', () => {
    render(<RestartBadge />)

    expect(screen.getByText('Restart to apply')).toBeInTheDocument()
  })
})
