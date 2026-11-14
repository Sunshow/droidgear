import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Plus } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from '@/components/ui/resizable'
import { usePlaygroundStore } from '@/store/playground-store'
import { VideoCreatePanel } from './VideoCreatePanel'
import { VideoModelDialog } from './VideoModelDialog'
import { VideoModelManagerDialog } from './VideoModelManagerDialog'
import { VideoTaskDetail } from './VideoTaskDetail'
import { VideoTaskList } from './VideoTaskList'

export function VideoPlaygroundPage() {
  const { t } = useTranslation()
  const [addOpen, setAddOpen] = useState(false)
  const [manageOpen, setManageOpen] = useState(false)

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center justify-between gap-2 border-b p-4">
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <h1 className="text-xl font-semibold">
              {t('playground.video.title')}
            </h1>
            <Badge variant="outline" className="text-amber-600">
              {t('playground.demoBadge')}
            </Badge>
          </div>
          <p className="text-sm text-muted-foreground">
            {t('playground.video.description')}
          </p>
        </div>
        <Button variant="outline" size="sm" onClick={() => setAddOpen(true)}>
          <Plus />
          {t('playground.video.addModel')}
        </Button>
      </div>

      <ResizablePanelGroup direction="horizontal" className="min-h-0 flex-1">
        <ResizablePanel defaultSize={32} minSize={24}>
          <VideoCreatePanel
            onAddModel={() => setAddOpen(true)}
            onManageModels={() => setManageOpen(true)}
          />
        </ResizablePanel>
        <ResizableHandle />
        <ResizablePanel defaultSize={68} minSize={40}>
          <ResizablePanelGroup direction="horizontal">
            <ResizablePanel defaultSize={42} minSize={28}>
              <VideoTaskList />
            </ResizablePanel>
            <ResizableHandle />
            <ResizablePanel defaultSize={58} minSize={35}>
              <VideoTaskDetail />
            </ResizablePanel>
          </ResizablePanelGroup>
        </ResizablePanel>
      </ResizablePanelGroup>

      <VideoModelDialog
        open={addOpen}
        onOpenChange={setAddOpen}
        onSave={draft => usePlaygroundStore.getState().addVideoModel(draft)}
      />
      <VideoModelManagerDialog open={manageOpen} onOpenChange={setManageOpen} />
    </div>
  )
}
