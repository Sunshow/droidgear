import { useTranslation } from 'react-i18next'
import { Server, KeyRound } from 'lucide-react'
import { cn } from '@/lib/utils'
import { ActionButton } from '@/components/ui/action-button'
import { useUIStore } from '@/store/ui-store'
import type { CodexSubView } from '@/store/ui-store'

interface FeatureItem {
  id: CodexSubView
  labelKey: string
  icon: React.ElementType
}

const features: FeatureItem[] = [
  { id: 'providers', labelKey: 'codex.features.providers', icon: Server },
  {
    id: 'auth-profiles',
    labelKey: 'codex.features.authProfiles',
    icon: KeyRound,
  },
]

export function CodexFeatureList() {
  const { t } = useTranslation()
  const codexSubView = useUIStore(state => state.codexSubView)
  const setCodexSubView = useUIStore(state => state.setCodexSubView)

  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-col gap-1 p-2">
        {features.map(feature => (
          <ActionButton
            key={feature.id}
            variant={codexSubView === feature.id ? 'secondary' : 'ghost'}
            size="sm"
            className={cn('justify-start w-full')}
            onClick={() => setCodexSubView(feature.id)}
          >
            <feature.icon className="h-4 w-4 mr-2" />
            {t(feature.labelKey)}
          </ActionButton>
        ))}
      </div>

      <div className="mt-auto p-3 border-t text-xs text-muted-foreground">
        {t('codex.features.hint')}
      </div>
    </div>
  )
}
