import { useTranslation } from 'react-i18next'
import { CircuitBoard, TerminalSquare } from 'lucide-react'
import { cn } from '@/lib/utils'
import { ActionButton } from '@/components/ui/action-button'
import { useUIStore, type DshSubView } from '@/store/ui-store'

interface FeatureItem {
  id: DshSubView
  labelKey: string
  icon: React.ElementType
}

const features: FeatureItem[] = [
  { id: 'providers', labelKey: 'dsh.features.providers', icon: CircuitBoard },
  {
    id: 'terminal',
    labelKey: 'dsh.features.terminal',
    icon: TerminalSquare,
  },
]

export function DshFeatureList() {
  const { t } = useTranslation()
  const dshSubView = useUIStore(state => state.dshSubView)
  const setDshSubView = useUIStore(state => state.setDshSubView)

  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-col gap-1 p-2">
        {features.map(feature => (
          <ActionButton
            key={feature.id}
            variant={dshSubView === feature.id ? 'secondary' : 'ghost'}
            size="sm"
            className={cn('justify-start w-full')}
            onClick={() => setDshSubView(feature.id)}
          >
            <feature.icon className="h-4 w-4 mr-2" />
            {t(feature.labelKey)}
          </ActionButton>
        ))}
      </div>

      <div className="mt-auto p-3 border-t text-xs text-muted-foreground">
        {t('dsh.features.hint')}
      </div>
    </div>
  )
}
