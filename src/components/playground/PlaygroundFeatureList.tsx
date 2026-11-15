import { useTranslation } from 'react-i18next'
import { Clapperboard, ImageIcon } from 'lucide-react'
import { ActionButton } from '@/components/ui/action-button'
import { isActiveVideoTask, usePlaygroundStore } from '@/store/playground-store'
import {
  isRunningImageTask,
  useImagePlaygroundStore,
} from '@/store/image-playground-store'
import { useUIStore } from '@/store/ui-store'
import { cn } from '@/lib/utils'

function FeatureRow({
  active,
  icon,
  label,
  count,
  onClick,
}: {
  active: boolean
  icon: React.ReactNode
  label: string
  count: number
  onClick: () => void
}) {
  return (
    <ActionButton
      variant={active ? 'secondary' : 'ghost'}
      size="sm"
      className={cn('w-full justify-start', !active && 'text-muted-foreground')}
      onClick={onClick}
    >
      {icon}
      {label}
      {count > 0 && (
        <span className="ms-auto rounded-full bg-primary px-1.5 text-[10px] leading-4 text-primary-foreground">
          {count}
        </span>
      )}
    </ActionButton>
  )
}

export function PlaygroundFeatureList() {
  const { t } = useTranslation()
  const playgroundSubView = useUIStore(state => state.playgroundSubView)
  const setPlaygroundSubView = useUIStore(state => state.setPlaygroundSubView)
  const activeVideoCount = usePlaygroundStore(
    state => state.videoTasks.filter(isActiveVideoTask).length
  )
  const runningImageCount = useImagePlaygroundStore(
    state => state.imageTasks.filter(isRunningImageTask).length
  )

  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-col gap-1 p-2">
        <FeatureRow
          active={playgroundSubView === 'video'}
          icon={<Clapperboard className="mr-2 h-4 w-4" />}
          label={t('playground.features.video')}
          count={activeVideoCount}
          onClick={() => setPlaygroundSubView('video')}
        />
        <FeatureRow
          active={playgroundSubView === 'image'}
          icon={<ImageIcon className="mr-2 h-4 w-4" />}
          label={t('playground.features.image')}
          count={runningImageCount}
          onClick={() => setPlaygroundSubView('image')}
        />
      </div>

      <div className="mt-auto border-t p-3 text-xs text-muted-foreground">
        {t('playground.features.hint')}
      </div>
    </div>
  )
}
