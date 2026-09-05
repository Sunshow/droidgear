import { useTranslation } from 'react-i18next'
import { CircuitBoard, TerminalSquare } from 'lucide-react'
import { cn } from '@/lib/utils'
import { ActionButton } from '@/components/ui/action-button'
import { useUIStore, type PiSubView } from '@/store/ui-store'

interface FeatureItem {
  id: PiSubView
  labelKey: string
  icon: React.ElementType
}

const features: FeatureItem[] = [
  { id: 'providers', labelKey: 'pi.features.providers', icon: CircuitBoard },
  {
    id: 'terminal',
    labelKey: 'pi.features.terminal',
    icon: TerminalSquare,
  },
]

export function PiFeatureList() {
  const { t } = useTranslation()
  const piSubView = useUIStore(state => state.piSubView)
  const setPiSubView = useUIStore(state => state.setPiSubView)

  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-col gap-1 p-2">
        {features.map(feature => (
          <ActionButton
            key={feature.id}
            variant={piSubView === feature.id ? 'secondary' : 'ghost'}
            size="sm"
            className={cn('justify-start w-full')}
            onClick={() => setPiSubView(feature.id)}
          >
            <feature.icon className="h-4 w-4 mr-2" />
            {t(feature.labelKey)}
          </ActionButton>
        ))}
      </div>

      <div className="mt-auto p-3 border-t text-xs text-muted-foreground">
        {t('pi.features.hint')}
      </div>
    </div>
  )
}
