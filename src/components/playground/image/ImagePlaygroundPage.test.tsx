import userEvent from '@testing-library/user-event'
import { toast } from 'sonner'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { render, screen, waitFor } from '@/test/test-utils'
import { commands, type ImageModel, type ImageTaskRecord } from '@/lib/bindings'
import { useImagePlaygroundStore } from '@/store/image-playground-store'
import { ImagePlaygroundPage } from './ImagePlaygroundPage'

vi.mock('sonner', () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}))

vi.mock('@/lib/bindings', () => ({
  commands: {
    loadImageModels: vi.fn().mockResolvedValue({ status: 'ok', data: [] }),
    saveImageModels: vi.fn().mockResolvedValue({ status: 'ok', data: null }),
    loadImageTasks: vi.fn().mockResolvedValue({ status: 'ok', data: [] }),
    saveImageTasks: vi.fn().mockResolvedValue({ status: 'ok', data: null }),
    createImageTask: vi.fn(),
    savePlaygroundMediaAs: vi.fn(),
  },
}))

const initialState = useImagePlaygroundStore.getState()

const model: ImageModel = {
  id: 'im_1',
  name: 'Nano Banana',
  modelId: 'gemini-3-pro-image',
  source: { kind: 'channel', channelId: 'ch_1', tokenId: 1 },
  protocol: 'google',
  sizes: ['16:9', '1:1'],
  qualities: ['1K', '2K'],
  supportsImageInput: true,
}

const params = {
  prompt: 'a cat',
  size: '16:9',
  quality: '2K',
  count: 1,
  imageUrl: '',
  extraJson: '',
}

function task(overrides: Partial<ImageTaskRecord> = {}): ImageTaskRecord {
  return {
    id: 'img_1',
    modelId: model.id,
    modelName: model.name,
    model,
    params,
    status: 'completed',
    images: [
      {
        localPath: '/tmp/droidgear/image-0.png',
        url: null,
        mimeType: 'image/png',
      },
    ],
    revisedPrompt: null,
    responseText: null,
    createdAt: Date.now() - 60_000,
    completedAt: Date.now() - 30_000,
    error: null,
    ...overrides,
  }
}

function seed(imageTasks: ImageTaskRecord[]) {
  useImagePlaygroundStore.setState(
    {
      ...initialState,
      imageModels: [model],
      imageModelsLoaded: true,
      imageTasksLoaded: true,
      imageTasks,
      selectedImageTaskId: imageTasks[0]?.id ?? null,
      imageDraft: { ...initialState.imageDraft, modelId: model.id },
    },
    true
  )
}

describe('ImagePlaygroundPage', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    seed([task()])
  })

  it('shows the create form, the task list, and the selected local image', () => {
    render(<ImagePlaygroundPage />)

    const modelPicker = screen.getByRole('combobox', { name: 'Model' })
    expect(modelPicker).toHaveTextContent('Nano Banana')
    expect(modelPicker).not.toHaveTextContent('gemini-3-pro-image')
    expect(screen.getByRole('button', { name: 'Manage' })).toBeInTheDocument()
    expect(
      screen.getByRole('button', { name: /Generate image/ })
    ).toBeDisabled()
    expect(screen.getAllByText('a cat').length).toBeGreaterThan(0)

    const image = screen.getByRole('img', { name: 'a cat' })
    expect(image.getAttribute('src')).toContain('asset://')
  })

  it('saves the selected image through the save dialog', async () => {
    vi.mocked(commands.savePlaygroundMediaAs).mockResolvedValueOnce({
      status: 'ok',
      data: '/Users/me/Pictures/img_1-1.png',
    })
    const user = userEvent.setup({ delay: null })

    render(<ImagePlaygroundPage />)
    await user.click(screen.getByRole('button', { name: 'Save as' }))

    expect(commands.savePlaygroundMediaAs).toHaveBeenCalledWith(
      '/tmp/droidgear/image-0.png',
      null,
      'img_1-1.png'
    )
    await waitFor(() =>
      expect(toast.success).toHaveBeenCalledWith(
        'Saved to /Users/me/Pictures/img_1-1.png'
      )
    )
  })

  it('submits the form with the selected model and free-form options', async () => {
    const user = userEvent.setup({ delay: null })
    useImagePlaygroundStore.setState(state => ({
      imageDraft: {
        ...state.imageDraft,
        prompt: 'a red fox',
        size: '1536x1024',
        quality: 'high',
        count: 2,
        imageUrl: 'https://example.com/cat.png',
        extraJson: '{"seed":7}',
      },
    }))
    vi.mocked(commands.createImageTask).mockResolvedValue({
      status: 'ok',
      data: {
        images: [
          { localPath: '/tmp/fox.png', url: null, mimeType: 'image/png' },
        ],
        revisedPrompt: null,
        responseText: 'here you go',
        completedAt: Date.now(),
      },
    })

    render(<ImagePlaygroundPage />)
    await user.click(screen.getByRole('button', { name: /Generate image/ }))

    await waitFor(() => expect(commands.createImageTask).toHaveBeenCalled())
    const call = vi.mocked(commands.createImageTask).mock.calls[0]
    if (!call) throw new Error('createImageTask was not called')
    const [modelArg, taskId, prompt, size, quality, count, urls, extra] = call
    expect(modelArg).toEqual(model)
    expect(taskId).toMatch(/^img_/)
    expect(prompt).toBe('a red fox')
    expect(size).toBe('1536x1024')
    expect(quality).toBe('high')
    expect(count).toBe(2)
    expect(urls).toEqual(['https://example.com/cat.png'])
    expect(extra).toBe('{"seed":7}')

    expect(
      await screen.findByRole('img', { name: 'a red fox' })
    ).toBeInTheDocument()
  })

  it('clears the create form but keeps the selected model', async () => {
    useImagePlaygroundStore.setState(state => ({
      imageDraft: {
        ...state.imageDraft,
        prompt: 'a cat',
        size: '16:9',
        quality: '2K',
        count: 2,
        imageUrl: 'https://example.com/cat.png',
        extraJson: '{"seed":1}',
      },
    }))
    const user = userEvent.setup({ delay: null })

    render(<ImagePlaygroundPage />)

    await user.click(screen.getByRole('button', { name: 'Clear form' }))

    expect(useImagePlaygroundStore.getState().imageDraft).toMatchObject({
      modelId: model.id,
      prompt: '',
      size: '',
      quality: '',
      count: 1,
      imageUrl: '',
      extraJson: '',
    })
  })

  it('switches between multiple generated images', async () => {
    const user = userEvent.setup({ delay: null })
    seed([
      task({
        images: [
          { localPath: '/tmp/first.png', url: null, mimeType: 'image/png' },
          { localPath: '/tmp/second.png', url: null, mimeType: 'image/png' },
        ],
      }),
    ])

    render(<ImagePlaygroundPage />)

    const main = screen.getByRole('img', { name: 'a cat' })
    expect(main.getAttribute('src')).toContain('first.png')

    await user.click(screen.getByRole('button', { name: 'Generated image 2' }))

    expect(
      screen.getByRole('img', { name: 'a cat' }).getAttribute('src')
    ).toContain('second.png')
    expect(
      screen.getByRole('button', { name: 'Generated image 2' })
    ).toHaveClass('border-primary')
  })

  it('renders the upstream error for a failed task', () => {
    seed([task({ status: 'failed', images: [], error: 'HTTP 400: bad size' })])

    render(<ImagePlaygroundPage />)

    expect(screen.getAllByText(/HTTP 400: bad size/).length).toBeGreaterThan(0)
    expect(screen.getAllByText('Failed').length).toBeGreaterThan(0)
  })

  it('hides the reference-image grid for models without image input', () => {
    useImagePlaygroundStore.setState({
      imageModels: [{ ...model, supportsImageInput: false }],
    })

    render(<ImagePlaygroundPage />)

    expect(
      screen.queryByRole('button', { name: 'Add image' })
    ).not.toBeInTheDocument()
  })

  it('shows reference images for image-input models', () => {
    render(<ImagePlaygroundPage />)

    expect(
      screen.getByRole('button', { name: 'Add image' })
    ).toBeInTheDocument()
  })

  it('hides the image count for protocols that return one image per request', () => {
    useImagePlaygroundStore.setState({
      imageModels: [
        {
          ...model,
          modelId: 'gemini-3-pro-image',
          protocol: 'google',
          qualities: ['1K', '2K'],
        },
      ],
    })

    render(<ImagePlaygroundPage />)

    expect(
      screen.queryByRole('spinbutton', { name: 'Images per request' })
    ).not.toBeInTheDocument()
  })

  it('hides reference images for the Images endpoint', () => {
    useImagePlaygroundStore.setState({
      imageModels: [{ ...model, protocol: 'images' }],
    })

    render(<ImagePlaygroundPage />)

    expect(
      screen.queryByRole('button', { name: 'Add image' })
    ).not.toBeInTheDocument()
  })

  it('keeps the count for chat models', () => {
    useImagePlaygroundStore.setState({
      imageModels: [{ ...model, protocol: 'chat' }],
    })

    render(<ImagePlaygroundPage />)

    expect(
      screen.getByRole('spinbutton', { name: 'Images per request' })
    ).toBeInTheDocument()
  })

  it('adds a model from the empty header dropdown', async () => {
    useImagePlaygroundStore.setState(
      {
        ...initialState,
        imageModels: [],
        imageModelsLoaded: true,
        imageTasksLoaded: true,
        imageTasks: [],
        selectedImageTaskId: null,
      },
      true
    )
    const user = userEvent.setup({ delay: null })

    render(<ImagePlaygroundPage />)

    expect(
      screen.getByText('No images yet. Create one on the left.')
    ).toBeInTheDocument()

    await user.click(screen.getByRole('combobox', { name: 'Model' }))
    await user.click(
      await screen.findByRole('option', { name: /Add image model/ })
    )

    expect(await screen.findByRole('dialog')).toBeInTheDocument()
  })
})
