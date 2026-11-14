import { beforeEach, describe, expect, it, vi } from 'vitest'
import { act, render, screen } from '@/test/test-utils'
import { usePlaygroundStore } from '@/store/playground-store'
import { VideoPlaygroundPage } from './VideoPlaygroundPage'

const initialState = usePlaygroundStore.getState()

describe('VideoPlaygroundPage', () => {
  beforeEach(() => {
    usePlaygroundStore.setState(initialState, true)
  })

  it('shows the create form, task list, and the selected video', () => {
    const { container } = render(<VideoPlaygroundPage />)

    expect(screen.getAllByText('Add video model').length).toBeGreaterThan(0)
    expect(screen.getByRole('button', { name: /Create task/ })).toBeDisabled()
    expect(screen.getByText('Auto refresh')).toBeInTheDocument()

    const video = container.querySelector('video')
    expect(video?.getAttribute('src')).toMatch(/\.mp4$/)
    expect(video?.hasAttribute('controls')).toBe(true)
  })

  it('polls running tasks on the auto refresh interval', async () => {
    vi.useFakeTimers()
    try {
      usePlaygroundStore.setState({ refreshIntervalSec: 3 })
      render(<VideoPlaygroundPage />)
      const before = usePlaygroundStore
        .getState()
        .videoTasks.find(task => task.id === 'task_demo0003')?.progress

      await act(async () => {
        await vi.advanceTimersByTimeAsync(3_500)
      })

      const after = usePlaygroundStore
        .getState()
        .videoTasks.find(task => task.id === 'task_demo0003')?.progress
      expect(after).toBeGreaterThan(before ?? 0)
    } finally {
      vi.useRealTimers()
    }
  })
})
