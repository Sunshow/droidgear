import { describe, expect, it } from 'vitest'
import { render, screen } from '@/test/test-utils'
import { JsExpressionBadge } from './JsExpressionBadge'

describe('JsExpressionBadge', () => {
  it('renders the JS expression read-only copy', () => {
    render(<JsExpressionBadge />)

    expect(screen.getByText('!!js expression (read-only)')).toBeInTheDocument()
  })
})
