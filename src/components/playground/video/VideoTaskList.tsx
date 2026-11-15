import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Film, Loader2, RefreshCw } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Switch } from '@/components/ui/switch'
import { Label } from '@/components/ui/label'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { cn } from '@/lib/utils'
import {
  isActiveVideoTask,
  usePlaygroundStore,
  type VideoTask,
} from '@/store/playground-store'
import { formatClock } from './video-format'
import { VideoTaskProgress, VideoTaskStatusIcon } from './VideoTaskStatus'

type Filter = 'all' | 'active' | 'completed' | 'failed'

const INTERVALS = [3, 5, 10, 30]

function matchesFilter(task: VideoTask, filter: Filter): boolean {
  switch (filter) {
    case 'active':
      return isActiveVideoTask(task)
    case 'completed':
      return task.status === 'completed'
    case 'failed':
      return task.status === 'failed' || task.status === 'cancelled'
    default:
      return true
  }
}

export function VideoTaskList() {
  const { t } = useTranslation()
  const tasks = usePlaygroundStore(state => state.videoTasks)
  const selectedId = usePlaygroundStore(state => state.selectedVideoTaskId)
  const selectTask = usePlaygroundStore(state => state.selectVideoTask)
  const autoRefresh = usePlaygroundStore(state => state.autoRefresh)
  const intervalSec = usePlaygroundStore(state => state.refreshIntervalSec)
  const isRefreshing = usePlaygroundStore(state => state.isRefreshing)
  const lastRefreshedAt = usePlaygroundStore(state => state.lastRefreshedAt)
  const [filter, setFilter] = useState<Filter>('all')

  const activeCount = tasks.filter(isActiveVideoTask).length
  const visible = tasks.filter(task => matchesFilter(task, filter))

  useEffect(() => {
    if (!autoRefresh || activeCount === 0) return
    const timer = setInterval(() => {
      void usePlaygroundStore.getState().refreshVideoTasks()
    }, intervalSec * 1000)
    return () => clearInterval(timer)
  }, [autoRefresh, intervalSec, activeCount])

  return (
    <div className="flex h-full flex-col">
      <div className="space-y-2 border-b p-3">
        <div className="flex items-center justify-between gap-2">
          <ToggleGroup
            type="single"
            size="sm"
            variant="outline"
            value={filter}
            onValueChange={value => {
              if (value) setFilter(value as Filter)
            }}
          >
            {(['all', 'active', 'completed', 'failed'] as const).map(f => (
              <ToggleGroupItem key={f} value={f} className="px-2 text-xs">
                {t(`playground.video.list.filter.${f}`)}
                {f === 'active' && activeCount > 0 && ` ${activeCount}`}
              </ToggleGroupItem>
            ))}
          </ToggleGroup>
          <Button
            variant="outline"
            size="icon-sm"
            title={t('playground.video.list.refreshNow')}
            disabled={isRefreshing}
            onClick={() =>
              void usePlaygroundStore.getState().refreshVideoTasks()
            }
          >
            <RefreshCw className={cn(isRefreshing && 'animate-spin')} />
          </Button>
        </div>
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <Switch
            id="video-auto-refresh"
            checked={autoRefresh}
            onCheckedChange={usePlaygroundStore.getState().setAutoRefresh}
          />
          <Label htmlFor="video-auto-refresh" className="text-xs font-normal">
            {t('playground.video.list.autoRefresh')}
          </Label>
          <Select
            value={String(intervalSec)}
            onValueChange={value =>
              usePlaygroundStore.getState().setRefreshIntervalSec(Number(value))
            }
            disabled={!autoRefresh}
          >
            <SelectTrigger size="sm" className="h-6 px-2 text-xs">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {INTERVALS.map(s => (
                <SelectItem key={s} value={String(s)}>
                  {t('playground.video.list.everySeconds', { count: s })}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <span className="ms-auto truncate">
            {isRefreshing ? (
              <Loader2 className="size-3 animate-spin" />
            ) : lastRefreshedAt ? (
              t('playground.video.list.lastRefreshed', {
                time: formatClock(lastRefreshedAt),
              })
            ) : null}
          </span>
        </div>
      </div>

      <div className="min-h-0 flex-1 overflow-auto">
        {visible.length === 0 ? (
          <div className="flex h-full flex-col items-center justify-center gap-2 p-6 text-center text-sm text-muted-foreground">
            <Film className="size-8 opacity-40" />
            {tasks.length === 0
              ? t('playground.video.list.empty')
              : t('playground.video.list.emptyFilter')}
          </div>
        ) : (
          <ul className="divide-y">
            {visible.map(task => (
              <li key={task.id}>
                <button
                  type="button"
                  onClick={() => selectTask(task.id)}
                  className={cn(
                    'flex w-full flex-col gap-1.5 px-3 py-2.5 text-start hover:bg-accent/50',
                    selectedId === task.id && 'bg-accent'
                  )}
                >
                  <div className="flex items-center gap-2">
                    <VideoTaskStatusIcon status={task.status} />
                    <span className="min-w-0 flex-1 truncate text-sm">
                      {task.params.prompt}
                    </span>
                  </div>
                  <div className="flex items-center gap-2 ps-6 text-xs text-muted-foreground">
                    <span className="truncate">{task.modelName}</span>
                    <span>·</span>
                    <span>{formatClock(task.createdAt)}</span>
                    <span className="ms-auto">
                      {task.status === 'in_progress'
                        ? `${task.progress}%`
                        : t(`playground.video.status.${task.status}`)}
                    </span>
                  </div>
                  {task.status === 'in_progress' && (
                    <VideoTaskProgress
                      value={task.progress}
                      className="ms-6 w-auto"
                    />
                  )}
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  )
}
