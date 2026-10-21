import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  Activity,
  AlertCircle,
  Bot,
  Boxes,
  Globe,
  LayoutDashboard,
  RefreshCw,
  Settings2,
  SlidersHorizontal,
  type LucideIcon,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { useDshStore } from '@/store/dsh-store'
import { DshAdvancedTab } from './DshAdvancedTab'
import { DshAgentTab } from './DshAgentTab'
import { DshDiagnosticsTab } from './DshDiagnosticsTab'
import { DshGeneralTab } from './DshGeneralTab'
import { DshModelsTab } from './DshModelsTab'
import { DshNetworkTab } from './DshNetworkTab'
import { DshOverviewTab } from './DshOverviewTab'

/** Official DSH runtime profile managed by this page. */
export type DshPageProfile = 'desktop' | 'web'

const TABS = [
  'overview',
  'models',
  'general',
  'agent',
  'network',
  'advanced',
  'diagnostics',
] as const

type DshTab = (typeof TABS)[number]

const TAB_ICONS: Record<DshTab, LucideIcon> = {
  overview: LayoutDashboard,
  models: Boxes,
  general: Settings2,
  agent: Bot,
  network: Globe,
  advanced: SlidersHorizontal,
  diagnostics: Activity,
}

interface DshConfigPageProps {
  profile: DshPageProfile
}

export function DshConfigPage({ profile }: DshConfigPageProps) {
  const { t } = useTranslation()
  const error = useDshStore(state => state.error)
  const setError = useDshStore(state => state.setError)
  const setProfile = useDshStore(state => state.setProfile)
  const loadProviders = useDshStore(state => state.loadProviders)
  const loadCredentials = useDshStore(state => state.loadCredentials)
  const loadConfigStatus = useDshStore(state => state.loadConfigStatus)
  const loadSnapshot = useDshStore(state => state.loadSnapshot)

  const [tab, setTab] = useState<DshTab>('overview')

  useEffect(() => {
    setProfile(profile)
    void loadProviders()
    void loadCredentials()
    void loadConfigStatus()
    void loadSnapshot()
  }, [
    profile,
    setProfile,
    loadProviders,
    loadCredentials,
    loadConfigStatus,
    loadSnapshot,
  ])

  const refresh = () => {
    setProfile(profile)
    void loadProviders()
    void loadCredentials()
    void loadConfigStatus()
    void loadSnapshot()
  }

  const title = t(
    profile === 'desktop' ? 'dsh.profile.desktop' : 'dsh.profile.web'
  )

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center justify-between gap-2 border-b p-4">
        <div className="min-w-0 flex-1">
          <h1 className="text-xl font-semibold">{title}</h1>
          <p className="text-sm text-muted-foreground">
            {t(`dsh.tabs.${tab}.description`)}
          </p>
        </div>
        <Button
          variant="outline"
          size="icon"
          onClick={refresh}
          title={t('common.refresh')}
        >
          <RefreshCw aria-hidden="true" />
        </Button>
      </div>

      {error ? (
        <div className="mx-4 mt-4 flex items-center gap-2 rounded-md border border-destructive/20 bg-destructive/10 p-3">
          <AlertCircle className="size-4 shrink-0 text-destructive" />
          <span className="text-sm text-destructive">{error}</span>
          <Button
            variant="ghost"
            size="sm"
            className="ms-auto"
            onClick={() => setError(null)}
          >
            {t('common.dismiss')}
          </Button>
        </div>
      ) : null}

      <Tabs
        value={tab}
        onValueChange={value => setTab(value as DshTab)}
        className="min-h-0 flex-1 gap-0"
      >
        <div className="overflow-x-auto border-b px-2 py-2">
          <TabsList className="w-max">
            {TABS.map(tabId => {
              const Icon = TAB_ICONS[tabId]
              return (
                <TabsTrigger key={tabId} value={tabId} className="gap-1.5 px-3">
                  <Icon aria-hidden="true" />
                  {t(`dsh.tabs.${tabId}`)}
                </TabsTrigger>
              )
            })}
          </TabsList>
        </div>

        <div className="min-h-0 flex-1 overflow-auto p-4">
          <TabsContent value="overview" className="mt-0">
            <DshOverviewTab />
          </TabsContent>
          <TabsContent value="models" className="mt-0">
            <DshModelsTab />
          </TabsContent>
          <TabsContent value="general" className="mt-0">
            <DshGeneralTab />
          </TabsContent>
          <TabsContent value="agent" className="mt-0">
            <DshAgentTab />
          </TabsContent>
          <TabsContent value="network" className="mt-0">
            <DshNetworkTab />
          </TabsContent>
          <TabsContent value="advanced" className="mt-0">
            <DshAdvancedTab />
          </TabsContent>
          <TabsContent value="diagnostics" className="mt-0">
            <DshDiagnosticsTab />
          </TabsContent>
        </div>
      </Tabs>
    </div>
  )
}
