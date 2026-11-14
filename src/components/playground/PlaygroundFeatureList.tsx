import { useTranslation } from 'react-i18next'
import { Clapperboard } from 'lucide-react'
import { ActionButton } from '@/components/ui/action-button'
import { isActiveVideoTask, usePlaygroundStore } from '@/store/playground-store'

export function PlaygroundFeatureList() {
  const { t } = useTranslation()
  const activeCount = usePlaygroundStore(
    state => state.videoTasks.filter(isActiveVideoTask).length
  )

  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-col gap-1 p-2">
        <ActionButton
          variant="secondary"
          size="sm"
          className="w-full justify-start"
        >
          <Clapperboard className="mr-2 h-4 w-4" />
          {t('playground.features.video')}
          {activeCount > 0 && (
            <span className="ms-auto rounded-full bg-primary px-1.5 text-[10px] leading-4 text-primary-foreground">
              {activeCount}
            </span>
          )}
        </ActionButton>
      </div>

      <div className="mt-auto border-t p-3 text-xs text-muted-foreground">
        {t('playground.features.hint')}
      </div>
    </div>
  )
}
