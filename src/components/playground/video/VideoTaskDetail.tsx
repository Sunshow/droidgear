import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { convertFileSrc } from '@tauri-apps/api/core'
import { toast } from 'sonner'
import {
  Ban,
  Check,
  Copy,
  Download,
  ListRestart,
  Loader2,
  RefreshCw,
  RotateCcw,
  Trash2,
  VideoOff,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { commands } from '@/lib/bindings'
import {
  isActiveVideoTask,
  usePlaygroundStore,
  type VideoTask,
} from '@/store/playground-store'
import { logger } from '@/lib/logger'
import { mediaFileName, parseImageUrls } from '@/lib/playground-utils'
import { formatDateTime } from './video-format'
import { VideoTaskProgress, VideoTaskStatusBadge } from './VideoTaskStatus'

function MetaRow({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <div className="contents">
      <span className="text-muted-foreground">{label}</span>
      <span className="min-w-0 break-all">{value}</span>
    </div>
  )
}

function VideoStage({ task }: { task: VideoTask }) {
  const { t } = useTranslation()
  const [loadFailed, setLoadFailed] = useState(false)

  const sourceUrl = task.localPath
    ? convertFileSrc(task.localPath)
    : task.videoUrl

  if (task.status === 'completed' && sourceUrl && !loadFailed) {
    return (
      <video
        key={sourceUrl}
        src={sourceUrl}
        controls
        playsInline
        className="h-full w-full bg-black object-contain"
        onError={() => setLoadFailed(true)}
      />
    )
  }

  let content: React.ReactNode
  if (isActiveVideoTask(task)) {
    content = (
      <>
        <Loader2 className="size-8 animate-spin opacity-60" />
        <span>
          {task.status === 'queued'
            ? t('playground.video.detail.queued')
            : t('playground.video.detail.generating', {
                progress: task.progress,
              })}
        </span>
        <VideoTaskProgress value={task.progress} className="w-48" />
      </>
    )
  } else if (task.status === 'failed') {
    content = (
      <>
        <VideoOff className="size-8 text-destructive opacity-80" />
        <span className="max-w-md text-destructive">{task.error}</span>
      </>
    )
  } else if (task.status === 'completed' && task.error) {
    content = (
      <>
        <VideoOff className="size-8 text-destructive opacity-80" />
        <span className="max-w-md text-destructive">{task.error}</span>
      </>
    )
  } else if (task.status === 'cancelled') {
    content = (
      <>
        <Ban className="size-8 opacity-60" />
        <span>{t('playground.video.status.cancelled')}</span>
      </>
    )
  } else {
    content = (
      <>
        <VideoOff className="size-8 opacity-60" />
        <span>{t('playground.video.detail.loadFailed')}</span>
      </>
    )
  }

  return (
    <div className="flex h-full flex-col items-center justify-center gap-3 p-6 text-center text-sm text-white/70">
      {content}
    </div>
  )
}

export function VideoTaskDetail() {
  const { t } = useTranslation()
  const task = usePlaygroundStore(state =>
    state.videoTasks.find(item => item.id === state.selectedVideoTaskId)
  )
  const isRefreshing = usePlaygroundStore(state => state.isRefreshing)
  const [copied, setCopied] = useState(false)

  if (!task) {
    return (
      <div className="flex h-full items-center justify-center p-6 text-sm text-muted-foreground">
        {t('playground.video.detail.selectHint')}
      </div>
    )
  }

  const active = isActiveVideoTask(task)
  const referenceImages = parseImageUrls(task.params.imageUrl)
  const store = usePlaygroundStore.getState()

  const handleCopyId = async () => {
    await navigator.clipboard.writeText(task.id)
    setCopied(true)
    setTimeout(() => setCopied(false), 1500)
  }

  const handleRerun = () => {
    void store.rerunVideoTask(task.id)
  }

  const canQuery = active || task.status === 'completed'
  const canSave = Boolean(task.localPath || task.videoUrl)
  const reportSaveFailure = (error: unknown) => {
    logger.error('Failed to save the video', { error })
    toast.error(t('playground.video.detail.saveFailed'))
  }
  const handleSaveAs = async () => {
    try {
      const result = await commands.savePlaygroundMediaAs(
        task.localPath,
        task.videoUrl,
        mediaFileName(task.id, task.localPath ?? task.videoUrl, 'mp4')
      )
      if (result.status === 'error') {
        reportSaveFailure(result.error)
        return
      }
      if (result.data) {
        toast.success(t('playground.video.detail.saved', { path: result.data }))
      }
    } catch (error) {
      reportSaveFailure(error)
    }
  }

  const handleLoadParams = () => {
    store.updateVideoDraft({ modelId: task.modelId, ...task.params })
  }

  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-wrap items-center gap-x-3 gap-y-1 border-b px-4 py-2">
        <VideoTaskStatusBadge status={task.status} />
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
          {canQuery && (
            <Button
              variant="ghost"
              size="icon-sm"
              disabled={isRefreshing}
              title={t('playground.video.detail.queryStatus')}
              aria-label={t('playground.video.detail.queryStatus')}
              onClick={() => void store.refreshVideoTasks(task.id)}
            >
              <RefreshCw className={isRefreshing ? 'animate-spin' : ''} />
            </Button>
          )}
          {active && (
            <Button
              variant="ghost"
              size="sm"
              title={t('playground.video.detail.cancelHint')}
              onClick={() => void store.cancelVideoTask(task.id)}
            >
              <Ban />
              {t('playground.video.detail.cancel')}
            </Button>
          )}
          {!active && (
            <>
              {canSave && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => void handleSaveAs()}
                >
                  <Download />
                  {t('playground.video.detail.saveAs')}
                </Button>
              )}
              <Button variant="ghost" size="sm" onClick={handleRerun}>
                <RotateCcw />
                {t('playground.video.detail.rerun')}
              </Button>
            </>
          )}
          <Button
            variant="ghost"
            size="icon-sm"
            title={t('common.delete')}
            aria-label={t('common.delete')}
            onClick={() => void store.deleteVideoTask(task.id)}
          >
            <Trash2 />
          </Button>
        </div>
      </div>

      <div className="aspect-video max-h-[55%] w-full shrink-0 bg-black">
        <VideoStage key={task.id} task={task} />
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
            {t('playground.video.detail.loadParams')}
          </Button>
        </div>
        <Separator />
        <div className="grid grid-cols-[auto_minmax(0,1fr)] items-start gap-x-3 gap-y-1.5 text-sm">
          <MetaRow
            label={t('playground.video.create.model')}
            value={task.modelName}
          />
          <MetaRow
            label={t('playground.video.detail.protocol')}
            value={
              task.model.protocol === 'new-api'
                ? t('playground.video.modelDialog.protocolNewApi')
                : task.model.protocol === 'doubao'
                  ? t('playground.video.modelDialog.protocolDoubao')
                  : t('playground.video.modelDialog.protocolSora')
            }
          />
          <MetaRow
            label={t('playground.video.create.duration')}
            value={
              task.params.duration
                ? `${task.params.duration}s`
                : t('playground.video.create.default')
            }
          />
          <MetaRow
            label={t('playground.video.create.resolution')}
            value={
              task.params.resolution || t('playground.video.create.default')
            }
          />
          {referenceImages.length > 0 && (
            <MetaRow
              label={t('playground.video.create.imageUrls')}
              value={
                <div className="flex flex-wrap gap-1.5">
                  {referenceImages.map((url, index) => (
                    <img
                      key={`${index}-${url.slice(0, 40)}`}
                      src={url}
                      alt={t('playground.referenceImages.alt', {
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
            label={t('playground.video.detail.createdAt')}
            value={formatDateTime(task.createdAt)}
          />
          {task.completedAt && (
            <MetaRow
              label={t('playground.video.detail.elapsed')}
              value={t('playground.video.detail.seconds', {
                count: Math.round((task.completedAt - task.createdAt) / 1000),
              })}
            />
          )}
        </div>
        {task.params.extraJson.trim() && (
          <pre className="overflow-auto rounded-md bg-muted p-3 font-mono text-xs">
            {task.params.extraJson}
          </pre>
        )}
      </div>
    </div>
  )
}
