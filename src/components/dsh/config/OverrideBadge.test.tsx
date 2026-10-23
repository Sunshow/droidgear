import { describe, expect, it } from 'vitest'
import { render, screen } from '@/test/test-utils'
import { OverrideBadge } from './OverrideBadge'

describe('OverrideBadge', () => {
  it('renders the overridden copy', () => {
    render(<OverrideBadge />)

    expect(screen.getByText('Overridden')).toBeInTheDocument()
  })
})
