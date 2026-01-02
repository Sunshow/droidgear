import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Cpu, LifeBuoy, FileText } from 'lucide-react'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { toast } from 'sonner'
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
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { useUIStore } from '@/store/ui-store'
import { useModelStore } from '@/store/model-store'
import { useIsWindows } from '@/hooks/use-platform'
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
  const isWindows = useIsWindows()

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

  const handleCopyCommand = async (command: string) => {
    await writeText(command)
    toast.success(t('common.copied'))
  }

  return (
    <div className="flex h-full flex-col">
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

      {/* Install Section */}
      <div className="mt-auto p-3 border-t text-xs text-muted-foreground">
        <div className="font-medium mb-2">{t('droid.install.title')}</div>
        <Tabs defaultValue={isWindows ? 'windows' : 'unix'} className="w-full">
          <TabsList className="w-full">
            <TabsTrigger value="unix" className="flex-1">
              macOS / Linux
            </TabsTrigger>
            <TabsTrigger value="windows" className="flex-1">
              Windows
            </TabsTrigger>
          </TabsList>
          <TabsContent value="unix">
            <code
              className="block bg-muted p-2 rounded text-xs break-all cursor-pointer hover:bg-muted/80 transition-colors"
              onClick={() =>
                handleCopyCommand('curl -fsSL https://app.factory.ai/cli | sh')
              }
            >
              curl -fsSL https://app.factory.ai/cli | sh
            </code>
          </TabsContent>
          <TabsContent value="windows">
            <code
              className="block bg-muted p-2 rounded text-xs break-all cursor-pointer hover:bg-muted/80 transition-colors"
              onClick={() =>
                handleCopyCommand(
                  'irm https://app.factory.ai/cli/windows | iex'
                )
              }
            >
              irm https://app.factory.ai/cli/windows | iex
            </code>
          </TabsContent>
        </Tabs>
        <a
          href="https://factory.ai"
          target="_blank"
          rel="noopener noreferrer"
          className="text-primary hover:underline mt-2 inline-block"
        >
          {t('droid.install.learnMore')}
        </a>
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
    </div>
  )
}
