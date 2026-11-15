import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from '@/components/ui/resizable'
import {
  useImagePlaygroundStore,
  type ImageModelDraft,
} from '@/store/image-playground-store'
import { ImageCreatePanel } from './ImageCreatePanel'
import { ImageModelDialog } from './ImageModelDialog'
import { ImageModelManagerDialog } from './ImageModelManagerDialog'
import { ImageModelToolbar } from './ImageModelToolbar'
import { ImageTaskDetail } from './ImageTaskDetail'
import { ImageTaskList } from './ImageTaskList'

export function ImagePlaygroundPage() {
  const { t } = useTranslation()
  const [addOpen, setAddOpen] = useState(false)
  const [manageOpen, setManageOpen] = useState(false)
  const loadImageModels = useImagePlaygroundStore(
    state => state.loadImageModels
  )
  const loadImageTasks = useImagePlaygroundStore(state => state.loadImageTasks)
  const imageModelsLoaded = useImagePlaygroundStore(
    state => state.imageModelsLoaded
  )
  const imageTasksLoaded = useImagePlaygroundStore(
    state => state.imageTasksLoaded
  )

  useEffect(() => {
    if (!imageModelsLoaded) void loadImageModels()
    if (!imageTasksLoaded) void loadImageTasks()
  }, [imageModelsLoaded, imageTasksLoaded, loadImageModels, loadImageTasks])

  const handleSave = (draft: ImageModelDraft) =>
    useImagePlaygroundStore.getState().addImageModel(draft)

  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-wrap items-center justify-between gap-3 border-b p-4">
        <div className="min-w-0 flex-1">
          <h1 className="text-xl font-semibold">
            {t('playground.image.title')}
          </h1>
        </div>
        <ImageModelToolbar
          onAddModel={() => setAddOpen(true)}
          onManageModels={() => setManageOpen(true)}
        />
      </div>

      <ResizablePanelGroup direction="horizontal" className="min-h-0 flex-1">
        <ResizablePanel defaultSize={32} minSize={24}>
          <ImageCreatePanel />
        </ResizablePanel>
        <ResizableHandle />
        <ResizablePanel defaultSize={68} minSize={40}>
          <ResizablePanelGroup direction="horizontal">
            <ResizablePanel defaultSize={42} minSize={28}>
              <ImageTaskList />
            </ResizablePanel>
            <ResizableHandle />
            <ResizablePanel defaultSize={58} minSize={35}>
              <ImageTaskDetail />
            </ResizablePanel>
          </ResizablePanelGroup>
        </ResizablePanel>
      </ResizablePanelGroup>

      <ImageModelDialog
        open={addOpen}
        onOpenChange={setAddOpen}
        onSave={handleSave}
        onSaveMany={drafts =>
          useImagePlaygroundStore.getState().addImageModels(drafts)
        }
      />
      <ImageModelManagerDialog open={manageOpen} onOpenChange={setManageOpen} />
    </div>
  )
}
