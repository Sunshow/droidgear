import { useTranslation } from 'react-i18next'
import { Server } from 'lucide-react'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { toast } from 'sonner'
import { cn } from '@/lib/utils'
import { ActionButton } from '@/components/ui/action-button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { useUIStore, type OpenClawSubView } from '@/store/ui-store'
import { useIsWindows } from '@/hooks/use-platform'

interface FeatureItem {
  id: OpenClawSubView
  labelKey: string
  icon: React.ElementType
}

const features: FeatureItem[] = [
  { id: 'providers', labelKey: 'openclaw.features.providers', icon: Server },
]

export function OpenClawFeatureList() {
  const { t } = useTranslation()
  const openclawSubView = useUIStore(state => state.openclawSubView)
  const setOpenClawSubView = useUIStore(state => state.setOpenClawSubView)
  const isWindows = useIsWindows()

  const handleCopyCommand = async (command: string) => {
    await writeText(command)
    toast.success(t('common.copied'))
  }

  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-col gap-1 p-2">
        {features.map(feature => (
          <ActionButton
            key={feature.id}
            variant={openclawSubView === feature.id ? 'secondary' : 'ghost'}
            size="sm"
            className={cn('justify-start w-full')}
            onClick={() => setOpenClawSubView(feature.id)}
          >
            <feature.icon className="h-4 w-4 mr-2" />
            {t(feature.labelKey)}
          </ActionButton>
        ))}
      </div>

      {/* Install Section */}
      <div className="mt-auto p-3 border-t text-xs text-muted-foreground">
        <div className="font-medium mb-2">{t('openclaw.install.title')}</div>
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
                handleCopyCommand('curl -fsSL https://openclaw.ai/install | sh')
              }
            >
              curl -fsSL https://openclaw.ai/install | sh
            </code>
          </TabsContent>
          <TabsContent value="windows">
            <code
              className="block bg-muted p-2 rounded text-xs break-all cursor-pointer hover:bg-muted/80 transition-colors"
              onClick={() =>
                handleCopyCommand('irm https://openclaw.ai/install.ps1 | iex')
              }
            >
              irm https://openclaw.ai/install.ps1 | iex
            </code>
          </TabsContent>
        </Tabs>
        <a
          href="https://docs.openclaw.ai"
          target="_blank"
          rel="noopener noreferrer"
          className="text-primary hover:underline mt-2 inline-block"
        >
          {t('openclaw.install.learnMore')}
        </a>
      </div>
    </div>
  )
}
