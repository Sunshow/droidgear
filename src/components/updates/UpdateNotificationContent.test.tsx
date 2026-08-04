import { render, screen } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useUIStore } from '@/store/ui-store'
import { UpdateNotificationContent } from './UpdateNotificationContent'

const defaultProps = {
  message: 'Update available: v1.2.2',
  releaseUrl: 'https://example.com/releases/v1.2.2',
  releaseLabel: 'View details',
  installLabel: 'Install now',
  laterLabel: 'Later',
  onOpenRelease: vi.fn(),
  onInstallNow: vi.fn(),
  onLater: vi.fn(),
}

describe('UpdateNotificationContent', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    useUIStore.setState({ isUpdateInstalling: false })
  })

  it('disables the install button while an update is installing', () => {
    useUIStore.setState({ isUpdateInstalling: true })

    render(<UpdateNotificationContent {...defaultProps} />)

    expect(
      screen.getByRole('button', { name: defaultProps.installLabel })
    ).toBeDisabled()
  })
})
