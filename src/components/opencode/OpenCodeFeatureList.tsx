import { useTranslation } from 'react-i18next'
import { Server } from 'lucide-react'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { toast } from 'sonner'
import { cn } from '@/lib/utils'
import { ActionButton } from '@/components/ui/action-button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { useUIStore, type OpenCodeSubView } from '@/store/ui-store'
import { useIsWindows } from '@/hooks/use-platform'

interface FeatureItem {
  id: OpenCodeSubView
  labelKey: string
  icon: React.ElementType
}

const features: FeatureItem[] = [
  { id: 'providers', labelKey: 'opencode.features.providers', icon: Server },
]

export function OpenCodeFeatureList() {
  const { t } = useTranslation()
  const opencodeSubView = useUIStore(state => state.opencodeSubView)
  const setOpenCodeSubView = useUIStore(state => state.setOpenCodeSubView)
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
            variant={opencodeSubView === feature.id ? 'secondary' : 'ghost'}
            size="sm"
            className={cn('justify-start w-full')}
            onClick={() => setOpenCodeSubView(feature.id)}
          >
            <feature.icon className="h-4 w-4 mr-2" />
            {t(feature.labelKey)}
          </ActionButton>
        ))}
      </div>

      {/* Install Section */}
      <div className="mt-auto p-3 border-t text-xs text-muted-foreground">
        <div className="font-medium mb-2">{t('opencode.install.title')}</div>
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
                handleCopyCommand(
                  'curl -fsSL https://opencode.ai/install | bash'
                )
              }
            >
              curl -fsSL https://opencode.ai/install | bash
            </code>
          </TabsContent>
          <TabsContent value="windows">
            <code
              className="block bg-muted p-2 rounded text-xs break-all cursor-pointer hover:bg-muted/80 transition-colors"
              onClick={() =>
                handleCopyCommand('irm https://opencode.ai/install.ps1 | iex')
              }
            >
              irm https://opencode.ai/install.ps1 | iex
            </code>
          </TabsContent>
        </Tabs>
        <a
          href="https://opencode.ai"
          target="_blank"
          rel="noopener noreferrer"
          className="text-primary hover:underline mt-2 inline-block"
        >
          {t('opencode.install.learnMore')}
        </a>
      </div>
    </div>
  )
}
