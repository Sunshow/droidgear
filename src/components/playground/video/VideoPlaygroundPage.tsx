import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from '@/components/ui/resizable'
import {
  usePlaygroundStore,
  type VideoModelDraft,
} from '@/store/playground-store'
import { VideoCreatePanel } from './VideoCreatePanel'
import { VideoModelDialog } from './VideoModelDialog'
import { VideoModelToolbar } from './VideoModelToolbar'
import { VideoModelManagerDialog } from './VideoModelManagerDialog'
import { VideoTaskDetail } from './VideoTaskDetail'
import { VideoTaskList } from './VideoTaskList'

export function VideoPlaygroundPage() {
  const { t } = useTranslation()
  const [addOpen, setAddOpen] = useState(false)
  const [manageOpen, setManageOpen] = useState(false)
  const loadVideoModels = usePlaygroundStore(state => state.loadVideoModels)
  const loadVideoTasks = usePlaygroundStore(state => state.loadVideoTasks)
  const videoModelsLoaded = usePlaygroundStore(state => state.videoModelsLoaded)
  const videoTasksLoaded = usePlaygroundStore(state => state.videoTasksLoaded)

  useEffect(() => {
    if (!videoModelsLoaded) void loadVideoModels()
    if (!videoTasksLoaded) void loadVideoTasks()
  }, [videoModelsLoaded, videoTasksLoaded, loadVideoModels, loadVideoTasks])

  const handleSave = (draft: VideoModelDraft) =>
    usePlaygroundStore.getState().addVideoModel(draft)

  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-wrap items-center justify-between gap-3 border-b p-4">
        <div className="min-w-0 flex-1">
          <h1 className="text-xl font-semibold">
            {t('playground.video.title')}
          </h1>
        </div>
        <VideoModelToolbar
          onAddModel={() => setAddOpen(true)}
          onManageModels={() => setManageOpen(true)}
        />
      </div>

      <ResizablePanelGroup direction="horizontal" className="min-h-0 flex-1">
        <ResizablePanel defaultSize={32} minSize={24}>
          <VideoCreatePanel />
        </ResizablePanel>
        <ResizableHandle />
        <ResizablePanel defaultSize={68} minSize={40}>
          <ResizablePanelGroup direction="horizontal">
            <ResizablePanel defaultSize={38} minSize={28}>
              <VideoTaskList />
            </ResizablePanel>
            <ResizableHandle />
            <ResizablePanel defaultSize={62} minSize={35}>
              <VideoTaskDetail />
            </ResizablePanel>
          </ResizablePanelGroup>
        </ResizablePanel>
      </ResizablePanelGroup>

      <VideoModelDialog
        open={addOpen}
        onOpenChange={setAddOpen}
        onSave={handleSave}
        onSaveMany={drafts =>
          usePlaygroundStore.getState().addVideoModels(drafts)
        }
      />
      <VideoModelManagerDialog open={manageOpen} onOpenChange={setManageOpen} />
    </div>
  )
}
