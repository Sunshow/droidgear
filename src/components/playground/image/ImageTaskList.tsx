import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { ImageIcon } from 'lucide-react'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import { cn } from '@/lib/utils'
import {
  isRunningImageTask,
  useImagePlaygroundStore,
  type ImageTask,
} from '@/store/image-playground-store'
import { formatClock } from '../shared/format'
import { ImageTaskStatusIcon } from './ImageTaskStatus'

type Filter = 'all' | 'running' | 'completed' | 'failed'

function matchesFilter(task: ImageTask, filter: Filter): boolean {
  switch (filter) {
    case 'running':
      return isRunningImageTask(task)
    case 'completed':
      return task.status === 'completed'
    case 'failed':
      return task.status === 'failed'
    default:
      return true
  }
}

export function ImageTaskList() {
  const { t } = useTranslation()
  const tasks = useImagePlaygroundStore(state => state.imageTasks)
  const selectedId = useImagePlaygroundStore(state => state.selectedImageTaskId)
  const selectTask = useImagePlaygroundStore(state => state.selectImageTask)
  const [filter, setFilter] = useState<Filter>('all')

  const runningCount = tasks.filter(isRunningImageTask).length
  const visible = tasks.filter(task => matchesFilter(task, filter))

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
            {(['all', 'running', 'completed', 'failed'] as const).map(item => (
              <ToggleGroupItem key={item} value={item} className="px-2 text-xs">
                {t(`playground.image.list.filter.${item}`)}
                {item === 'running' && runningCount > 0 && ` ${runningCount}`}
              </ToggleGroupItem>
            ))}
          </ToggleGroup>
        </div>
        <p className="text-xs text-muted-foreground">
          {t('playground.image.list.hint')}
        </p>
      </div>

      <div className="min-h-0 flex-1 overflow-auto">
        {visible.length === 0 ? (
          <div className="flex h-full flex-col items-center justify-center gap-2 p-6 text-center text-sm text-muted-foreground">
            <ImageIcon className="size-8 opacity-40" />
            {tasks.length === 0
              ? t('playground.image.list.empty')
              : t('playground.image.list.emptyFilter')}
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
                    <ImageTaskStatusIcon status={task.status} />
                    <span className="min-w-0 flex-1 truncate text-sm">
                      {task.params.prompt}
                    </span>
                  </div>
                  <div className="flex items-center gap-2 ps-6 text-xs text-muted-foreground">
                    <span className="truncate">{task.modelName}</span>
                    <span>·</span>
                    <span>{formatClock(task.createdAt)}</span>
                    <span className="ms-auto shrink-0">
                      {task.status === 'completed'
                        ? t('playground.image.list.imageCount', {
                            count: task.images.length,
                          })
                        : t(`playground.image.status.${task.status}`)}
                    </span>
                  </div>
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  )
}
