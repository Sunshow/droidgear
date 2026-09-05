import { useTranslation } from 'react-i18next'
import { Cpu, TerminalSquare } from 'lucide-react'
import { cn } from '@/lib/utils'
import { ActionButton } from '@/components/ui/action-button'
import { useUIStore, type HermesSubView } from '@/store/ui-store'

interface FeatureItem {
  id: HermesSubView
  labelKey: string
  icon: React.ElementType
}

const features: FeatureItem[] = [
  { id: 'model', labelKey: 'hermes.features.model', icon: Cpu },
  {
    id: 'terminal',
    labelKey: 'hermes.features.terminal',
    icon: TerminalSquare,
  },
]

export function HermesFeatureList() {
  const { t } = useTranslation()
  const hermesSubView = useUIStore(state => state.hermesSubView)
  const setHermesSubView = useUIStore(state => state.setHermesSubView)

  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-col gap-1 p-2">
        {features.map(feature => (
          <ActionButton
            key={feature.id}
            variant={hermesSubView === feature.id ? 'secondary' : 'ghost'}
            size="sm"
            className={cn('justify-start w-full')}
            onClick={() => setHermesSubView(feature.id)}
          >
            <feature.icon className="h-4 w-4 mr-2" />
            {t(feature.labelKey)}
          </ActionButton>
        ))}
      </div>

      <div className="mt-auto p-3 border-t text-xs text-muted-foreground">
        {t('hermes.features.hint')}
      </div>
    </div>
  )
}
