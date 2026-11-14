import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { openUrl } from '@tauri-apps/plugin-opener'
import {
  Ban,
  Check,
  Copy,
  ExternalLink,
  ListRestart,
  Loader2,
  RefreshCw,
  RotateCcw,
  Trash2,
  VideoOff,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import {
  isActiveVideoTask,
  usePlaygroundStore,
  type VideoTask,
} from '@/store/playground-store'
import { formatDateTime } from './video-format'
import { VideoTaskProgress, VideoTaskStatusBadge } from './VideoTaskStatus'

function MetaRow({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <div className="flex gap-3 text-sm">
      <span className="w-20 shrink-0 text-muted-foreground">{label}</span>
      <span className="min-w-0 flex-1 break-all">{value}</span>
    </div>
  )
}

function VideoStage({ task }: { task: VideoTask }) {
  const { t } = useTranslation()
  const [loadFailed, setLoadFailed] = useState(false)

  if (task.status === 'completed' && task.videoUrl && !loadFailed) {
    return (
      <video
        key={task.videoUrl}
        src={task.videoUrl}
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
  } else if (loadFailed) {
    content = (
      <>
        <VideoOff className="size-8 opacity-60" />
        <span>{t('playground.video.detail.loadFailed')}</span>
      </>
    )
  } else {
    content = (
      <>
        <Ban className="size-8 opacity-60" />
        <span>{t('playground.video.status.cancelled')}</span>
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
  const store = usePlaygroundStore.getState()

  const handleCopyId = async () => {
    await navigator.clipboard.writeText(task.id)
    setCopied(true)
    setTimeout(() => setCopied(false), 1500)
  }

  const handleRerun = () => {
    store.createVideoTask(task.modelId, task.params)
  }

  const handleLoadParams = () => {
    store.updateVideoDraft({ modelId: task.modelId, ...task.params })
  }

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center gap-2 border-b px-4 py-2">
        <VideoTaskStatusBadge status={task.status} />
        <button
          type="button"
          onClick={() => void handleCopyId()}
          className="flex items-center gap-1 font-mono text-xs text-muted-foreground hover:text-foreground"
          title={t('common.copy')}
        >
          {task.id}
          {copied ? <Check className="size-3" /> : <Copy className="size-3" />}
        </button>
        <div className="ms-auto flex items-center gap-1">
          {active ? (
            <>
              <Button
                variant="ghost"
                size="sm"
                disabled={isRefreshing}
                onClick={() => void store.refreshVideoTasks(task.id)}
              >
                <RefreshCw className={isRefreshing ? 'animate-spin' : ''} />
                {t('playground.video.detail.queryStatus')}
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => store.cancelVideoTask(task.id)}
              >
                <Ban />
                {t('playground.video.detail.cancel')}
              </Button>
            </>
          ) : (
            <>
              {task.videoUrl && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => task.videoUrl && void openUrl(task.videoUrl)}
                >
                  <ExternalLink />
                  {t('playground.video.detail.openExternal')}
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
            onClick={() => store.deleteVideoTask(task.id)}
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
        <div className="space-y-1.5">
          <MetaRow
            label={t('playground.video.create.model')}
            value={task.modelName}
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
          {task.params.imageUrl && (
            <MetaRow
              label={t('playground.video.create.imageUrl')}
              value={task.params.imageUrl}
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
