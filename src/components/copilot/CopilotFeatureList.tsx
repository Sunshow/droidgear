import { useTranslation } from 'react-i18next'
import { Settings2, TerminalSquare } from 'lucide-react'
import { cn } from '@/lib/utils'
import { ActionButton } from '@/components/ui/action-button'
import { useUIStore, type CopilotSubView } from '@/store/ui-store'

interface FeatureItem {
  id: CopilotSubView
  labelKey: string
  icon: React.ElementType
}

const features: FeatureItem[] = [
  { id: 'profiles', labelKey: 'copilot.features.profiles', icon: Settings2 },
  {
    id: 'terminal',
    labelKey: 'copilot.features.terminal',
    icon: TerminalSquare,
  },
]

export function CopilotFeatureList() {
  const { t } = useTranslation()
  const copilotSubView = useUIStore(state => state.copilotSubView)
  const setCopilotSubView = useUIStore(state => state.setCopilotSubView)

  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-col gap-1 p-2">
        {features.map(feature => (
          <ActionButton
            key={feature.id}
            variant={copilotSubView === feature.id ? 'secondary' : 'ghost'}
            size="sm"
            className={cn('w-full justify-start')}
            onClick={() => setCopilotSubView(feature.id)}
          >
            <feature.icon className="mr-2 h-4 w-4" />
            {t(feature.labelKey)}
          </ActionButton>
        ))}
      </div>
      <div className="mt-auto border-t p-3 text-xs text-muted-foreground">
        {t('copilot.features.hint')}
      </div>
    </div>
  )
}
