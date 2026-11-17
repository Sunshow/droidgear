import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { convertFileSrc } from '@tauri-apps/api/core'
import { revealItemInDir } from '@tauri-apps/plugin-opener'
import { toast } from 'sonner'
import {
  Check,
  Copy,
  Download,
  FolderOpen,
  ImageOff,
  ListRestart,
  Loader2,
  RotateCcw,
  Trash2,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { commands } from '@/lib/bindings'
import { logger } from '@/lib/logger'
import { mediaFileName, parseImageUrls } from '@/lib/playground-utils'
import {
  isRunningImageTask,
  useImagePlaygroundStore,
  type ImageAsset,
  type ImageTask,
} from '@/store/image-playground-store'
import { formatDateTime } from '../shared/format'
import { imageProtocolLabel } from './image-protocol'
import { ImageTaskStatusBadge } from './ImageTaskStatus'

function MetaRow({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <div className="contents">
      <span className="text-muted-foreground">{label}</span>
      <span className="min-w-0 break-all">{value}</span>
    </div>
  )
}

function imageAssetSrc(asset: ImageAsset): string | null {
  if (asset.localPath) return convertFileSrc(asset.localPath)
  return asset.url
}

function ImageStage({
  task,
  index,
  onSelect,
}: {
  task: ImageTask
  index: number
  onSelect: (index: number) => void
}) {
  const { t } = useTranslation()
  const [failedSource, setFailedSource] = useState<string | null>(null)
  const asset = task.images[index]
  const sourceUrl = asset ? imageAssetSrc(asset) : null
  const loadFailed = sourceUrl !== null && failedSource === sourceUrl

  if (task.status === 'completed' && sourceUrl && !loadFailed) {
    return (
      <div className="flex h-full w-full flex-col">
        <div className="flex min-h-0 flex-1 items-center justify-center p-2">
          <img
            src={sourceUrl}
            alt={
              task.params.prompt ||
              t('playground.image.detail.imageAlt', { index: index + 1 })
            }
            className="max-h-full max-w-full object-contain"
            onError={() => {
              if (sourceUrl) setFailedSource(sourceUrl)
            }}
            draggable={false}
          />
        </div>
        {task.images.length > 1 && (
          <div className="flex shrink-0 gap-2 overflow-x-auto border-t border-white/10 p-2">
            {task.images.map((item, itemIndex) => {
              const thumb = imageAssetSrc(item)
              return (
                <button
                  key={`${itemIndex}-${thumb ?? 'image'}`}
                  type="button"
                  aria-label={t('playground.image.detail.imageAlt', {
                    index: itemIndex + 1,
                  })}
                  onClick={() => onSelect(itemIndex)}
                  className={`size-14 shrink-0 overflow-hidden rounded border-2 bg-black ${
                    itemIndex === index
                      ? 'border-primary'
                      : 'border-transparent opacity-70 hover:opacity-100'
                  }`}
                >
                  {thumb && (
                    <img
                      src={thumb}
                      alt=""
                      className="size-full object-cover"
                      draggable={false}
                    />
                  )}
                </button>
              )
            })}
          </div>
        )}
      </div>
    )
  }

  let content: React.ReactNode
  if (isRunningImageTask(task)) {
    content = (
      <>
        <Loader2 className="size-8 animate-spin opacity-60" />
        <span>{t('playground.image.detail.generating')}</span>
      </>
    )
  } else if (task.status === 'failed') {
    content = (
      <>
        <ImageOff className="size-8 text-destructive opacity-80" />
        <span className="max-w-md text-destructive">{task.error}</span>
      </>
    )
  } else {
    content = (
      <>
        <ImageOff className="size-8 opacity-60" />
        <span>{t('playground.image.detail.loadFailed')}</span>
      </>
    )
  }

  return (
    <div className="flex h-full flex-col items-center justify-center gap-3 p-6 text-center text-sm text-white/70">
      {content}
    </div>
  )
}

export function ImageTaskDetail() {
  const { t } = useTranslation()
  const task = useImagePlaygroundStore(state =>
    state.imageTasks.find(item => item.id === state.selectedImageTaskId)
  )
  const [copied, setCopied] = useState(false)
  const [selection, setSelection] = useState<{ taskId: string; index: number }>(
    { taskId: task?.id ?? '', index: 0 }
  )

  if (!task) {
    return (
      <div className="flex h-full items-center justify-center p-6 text-sm text-muted-foreground">
        {t('playground.image.detail.selectHint')}
      </div>
    )
  }

  const running = isRunningImageTask(task)
  const referenceImages = parseImageUrls(task.params.imageUrl)
  const selectedIndex = selection.taskId === task.id ? selection.index : 0
  const imageIndex =
    task.images.length > 0 ? Math.min(selectedIndex, task.images.length - 1) : 0
  const selectedAsset = task.images[imageIndex]
  const store = useImagePlaygroundStore.getState()

  const handleCopyId = async () => {
    await navigator.clipboard.writeText(task.id)
    setCopied(true)
    setTimeout(() => setCopied(false), 1500)
  }

  const reportSaveFailure = (error: unknown) => {
    logger.error('Failed to save the image', { error })
    toast.error(t('playground.image.detail.saveFailed'))
  }
  const handleSaveAs = async () => {
    if (!selectedAsset) return
    try {
      const result = await commands.savePlaygroundMediaAs(
        selectedAsset.localPath,
        selectedAsset.url,
        mediaFileName(
          `${task.id}-${imageIndex + 1}`,
          selectedAsset.localPath ?? selectedAsset.url,
          'png'
        )
      )
      if (result.status === 'error') {
        reportSaveFailure(result.error)
        return
      }
      if (result.data) {
        toast.success(t('playground.image.detail.saved', { path: result.data }))
      }
    } catch (error) {
      reportSaveFailure(error)
    }
  }

  const handleReveal = () => {
    if (!selectedAsset?.localPath) return
    void revealItemInDir(selectedAsset.localPath)
  }

  const handleLoadParams = () => {
    store.updateImageDraft({ modelId: task.modelId, ...task.params })
  }

  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-wrap items-center gap-x-3 gap-y-1 border-b px-4 py-2">
        <ImageTaskStatusBadge status={task.status} />
        <button
          type="button"
          onClick={() => void handleCopyId()}
          className="flex min-w-0 max-w-40 items-center gap-1 font-mono text-xs text-muted-foreground hover:text-foreground"
          title={task.id}
        >
          <span className="truncate">{task.id}</span>
          {copied ? (
            <Check className="size-3 shrink-0" />
          ) : (
            <Copy className="size-3 shrink-0" />
          )}
        </button>
        <div className="ms-auto flex shrink-0 items-center gap-1">
          {!running && (
            <>
              {selectedAsset && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => void handleSaveAs()}
                >
                  <Download />
                  {t('playground.image.detail.saveAs')}
                </Button>
              )}
              {selectedAsset?.localPath && (
                <Button
                  variant="ghost"
                  size="icon-sm"
                  title={t('playground.image.detail.reveal')}
                  aria-label={t('playground.image.detail.reveal')}
                  onClick={handleReveal}
                >
                  <FolderOpen />
                </Button>
              )}
              <Button
                variant="ghost"
                size="sm"
                onClick={() => void store.rerunImageTask(task.id)}
              >
                <RotateCcw />
                {t('playground.image.detail.rerun')}
              </Button>
            </>
          )}
          <Button
            variant="ghost"
            size="icon-sm"
            title={t('common.delete')}
            aria-label={t('common.delete')}
            onClick={() => void store.deleteImageTask(task.id)}
          >
            <Trash2 />
          </Button>
        </div>
      </div>

      <div className="aspect-square max-h-[55%] w-full shrink-0 bg-black">
        <ImageStage
          key={task.id}
          task={task}
          index={imageIndex}
          onSelect={index => setSelection({ taskId: task.id, index })}
        />
      </div>

      <div className="min-h-0 flex-1 space-y-3 overflow-auto p-4">
        <div className="flex items-start justify-between gap-2">
          <p className="text-sm whitespace-pre-wrap">{task.params.prompt}</p>
          <Button
            variant="outline"
            size="sm"
            className="shrink-0"
            onClick={handleLoadParams}
          >
            <ListRestart />
            {t('playground.image.detail.loadParams')}
          </Button>
        </div>
        <Separator />
        <div className="grid grid-cols-[auto_minmax(0,1fr)] items-start gap-x-3 gap-y-1.5 text-sm">
          <MetaRow
            label={t('playground.image.create.model')}
            value={task.modelName}
          />
          <MetaRow
            label={t('playground.image.detail.protocol')}
            value={imageProtocolLabel(t, task.model.protocol)}
          />
          <MetaRow
            label={t('playground.image.create.size')}
            value={task.params.size || t('playground.image.create.default')}
          />
          {(task.params.quality || task.model.qualities.length > 0) && (
            <MetaRow
              label={t('playground.image.create.quality')}
              value={
                task.params.quality || t('playground.image.create.default')
              }
            />
          )}
          <MetaRow
            label={t('playground.image.create.count')}
            value={t('playground.image.detail.images', {
              count: Math.max(task.images.length, task.params.count),
            })}
          />
          {referenceImages.length > 0 && (
            <MetaRow
              label={t('playground.image.create.imageUrls')}
              value={
                <div className="flex flex-wrap gap-1.5">
                  {referenceImages.map((url, index) => (
                    <img
                      key={`${index}-${url.slice(0, 40)}`}
                      src={url}
                      alt={t('playground.image.create.imageAlt', {
                        index: index + 1,
                      })}
                      className="size-12 rounded border object-cover"
                    />
                  ))}
                </div>
              }
            />
          )}
          <MetaRow
            label={t('playground.image.detail.createdAt')}
            value={formatDateTime(task.createdAt)}
          />
          {task.completedAt && (
            <MetaRow
              label={t('playground.image.detail.elapsed')}
              value={t('playground.image.detail.seconds', {
                count: Math.max(
                  0,
                  Math.round((task.completedAt - task.createdAt) / 1000)
                ),
              })}
            />
          )}
        </div>
        {task.revisedPrompt && (
          <div className="space-y-1">
            <p className="text-xs font-medium text-muted-foreground">
              {t('playground.image.detail.revisedPrompt')}
            </p>
            <p className="text-sm whitespace-pre-wrap">{task.revisedPrompt}</p>
          </div>
        )}
        {task.responseText && (
          <div className="space-y-1">
            <p className="text-xs font-medium text-muted-foreground">
              {t('playground.image.detail.responseText')}
            </p>
            <p className="text-sm whitespace-pre-wrap">{task.responseText}</p>
          </div>
        )}
        {task.error && task.status !== 'failed' && (
          <p className="text-sm text-destructive">{task.error}</p>
        )}
        {task.params.extraJson.trim() && (
          <pre className="overflow-auto rounded-md bg-muted p-3 font-mono text-xs">
            {task.params.extraJson}
          </pre>
        )}
      </div>
    </div>
  )
}
