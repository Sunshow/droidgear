import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Cpu, LifeBuoy, FileText } from 'lucide-react'
import { cn } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { useUIStore } from '@/store/ui-store'
import { useModelStore } from '@/store/model-store'
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
  const modelHasChanges = useModelStore(state => state.hasChanges)

  const [pendingSubView, setPendingSubView] = useState<DroidSubView | null>(
    null
  )

  const handleSubViewChange = (view: DroidSubView) => {
    if (view === droidSubView) return

    // Only check for unsaved changes when leaving models view
    if (droidSubView === 'models' && modelHasChanges) {
      setPendingSubView(view)
    } else {
      setDroidSubView(view)
    }
  }

  const handleSaveAndSwitch = async () => {
    await useModelStore.getState().saveModels()
    if (pendingSubView) {
      setDroidSubView(pendingSubView)
      setPendingSubView(null)
    }
  }

  const handleDiscardAndSwitch = () => {
    useModelStore.getState().resetChanges()
    if (pendingSubView) {
      setDroidSubView(pendingSubView)
      setPendingSubView(null)
    }
  }

  return (
    <>
      <div className="flex flex-col gap-1 p-2">
        {features.map(feature => (
          <Button
            key={feature.id}
            variant={droidSubView === feature.id ? 'secondary' : 'ghost'}
            size="sm"
            className={cn('justify-start w-full')}
            onClick={() => handleSubViewChange(feature.id)}
          >
            <feature.icon className="h-4 w-4 mr-2" />
            {t(feature.labelKey)}
          </Button>
        ))}
      </div>

      {/* Unsaved Changes Confirmation Dialog */}
      <AlertDialog
        open={pendingSubView !== null}
        onOpenChange={open => !open && setPendingSubView(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {t('sidebar.unsavedChanges.title')}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {t('sidebar.unsavedChanges.description')}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <Button variant="destructive" onClick={handleDiscardAndSwitch}>
              {t('sidebar.unsavedChanges.discard')}
            </Button>
            <Button onClick={handleSaveAndSwitch}>
              {t('sidebar.unsavedChanges.save')}
            </Button>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  )
}
