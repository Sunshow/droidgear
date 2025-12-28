import { useTranslation } from 'react-i18next'
import { Cpu, LifeBuoy, FileText } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { useUIStore } from '@/store/ui-store'
import type { DroidSubView } from '@/store/ui-store'

interface FeatureItem {
  id: DroidSubView
  labelKey: string
  icon: React.ElementType
}

const features: FeatureItem[] = [
  { id: 'models', labelKey: 'droid.features.models', icon: Cpu },
  { id: 'helpers', labelKey: 'droid.features.helpers', icon: LifeBuoy },
  { id: 'specs', labelKey: 'droid.features.specs', icon: FileText },
]

export function DroidFeatureList() {
  const { t } = useTranslation()
  const droidSubView = useUIStore(state => state.droidSubView)
  const setDroidSubView = useUIStore(state => state.setDroidSubView)

  return (
    <div className="flex flex-col gap-1 p-2">
      {features.map(feature => (
        <Button
          key={feature.id}
          variant={droidSubView === feature.id ? 'secondary' : 'ghost'}
          size="sm"
          className={cn('justify-start w-full')}
          onClick={() => setDroidSubView(feature.id)}
        >
          <feature.icon className="h-4 w-4 mr-2" />
          {t(feature.labelKey)}
        </Button>
      ))}
    </div>
  )
}
