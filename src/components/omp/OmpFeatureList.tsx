import { useTranslation } from 'react-i18next'
import { Settings, TerminalSquare } from 'lucide-react'
import { cn } from '@/lib/utils'
import { ActionButton } from '@/components/ui/action-button'
import { useUIStore, type OmpSubView } from '@/store/ui-store'

interface FeatureItem {
  id: OmpSubView
  labelKey: string
  icon: React.ElementType
}

const features: FeatureItem[] = [
  { id: 'config', labelKey: 'omp.features.config', icon: Settings },
  {
    id: 'terminal',
    labelKey: 'omp.features.terminal',
    icon: TerminalSquare,
  },
]

export function OmpFeatureList() {
  const { t } = useTranslation()
  const ompSubView = useUIStore(state => state.ompSubView)
  const setOmpSubView = useUIStore(state => state.setOmpSubView)

  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-col gap-1 p-2">
        {features.map(feature => (
          <ActionButton
            key={feature.id}
            variant={ompSubView === feature.id ? 'secondary' : 'ghost'}
            size="sm"
            className={cn('justify-start w-full')}
            onClick={() => setOmpSubView(feature.id)}
          >
            <feature.icon className="h-4 w-4 mr-2" />
            {t(feature.labelKey)}
          </ActionButton>
        ))}
      </div>

      <div className="mt-auto p-3 border-t text-xs text-muted-foreground">
        {t('omp.features.hint')}
      </div>
    </div>
  )
}
