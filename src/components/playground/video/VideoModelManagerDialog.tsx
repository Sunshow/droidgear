import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Pencil, Plus, Trash2 } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { useChannelStore } from '@/store/channel-store'
import {
  usePlaygroundStore,
  type VideoModel,
  type VideoModelDraft,
} from '@/store/playground-store'
import { VideoModelDialog } from './VideoModelDialog'

export function VideoModelManagerDialog({
  open,
  onOpenChange,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
}) {
  const { t } = useTranslation()
  const models = usePlaygroundStore(state => state.videoModels)
  const channels = useChannelStore(state => state.channels)
  const [editing, setEditing] = useState<VideoModel | undefined>()
  const [formOpen, setFormOpen] = useState(false)

  const sourceLabel = (model: VideoModel) =>
    model.source.kind === 'channel'
      ? (channels.find(
          c =>
            model.source.kind === 'channel' && c.id === model.source.channelId
        )?.name ?? t('playground.video.models.missingChannel'))
      : model.source.baseUrl

  const handleSave = (draft: VideoModelDraft) => {
    const store = usePlaygroundStore.getState()
    if (editing) store.updateVideoModel(editing.id, draft)
    else store.addVideoModel(draft)
  }

  return (
    <>
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent className="sm:max-w-xl">
          <DialogHeader>
            <DialogTitle>{t('playground.video.models.title')}</DialogTitle>
            <DialogDescription>
              {t('playground.video.models.description')}
            </DialogDescription>
          </DialogHeader>

          <div className="max-h-80 divide-y overflow-auto rounded-md border">
            {models.length === 0 ? (
              <p className="p-6 text-center text-sm text-muted-foreground">
                {t('playground.video.models.empty')}
              </p>
            ) : (
              models.map(model => (
                <div key={model.id} className="flex items-center gap-3 p-3">
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                      <span className="truncate font-medium">{model.name}</span>
                      <Badge variant="secondary" className="font-mono">
                        {model.modelId}
                      </Badge>
                    </div>
                    <p className="truncate text-xs text-muted-foreground">
                      {sourceLabel(model)}
                      {model.durations.length > 0 &&
                        ` · ${model.durations.map(d => `${d}s`).join('/')}`}
                      {model.supportsImageInput &&
                        ` · ${t('playground.video.models.imageInput')}`}
                    </p>
                  </div>
                  <Button
                    variant="ghost"
                    size="icon-sm"
                    title={t('common.edit')}
                    onClick={() => {
                      setEditing(model)
                      setFormOpen(true)
                    }}
                  >
                    <Pencil />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon-sm"
                    title={t('common.delete')}
                    onClick={() =>
                      usePlaygroundStore.getState().deleteVideoModel(model.id)
                    }
                  >
                    <Trash2 />
                  </Button>
                </div>
              ))
            )}
          </div>

          <div className="flex justify-end">
            <Button
              onClick={() => {
                setEditing(undefined)
                setFormOpen(true)
              }}
            >
              <Plus />
              {t('playground.video.addModel')}
            </Button>
          </div>
        </DialogContent>
      </Dialog>

      <VideoModelDialog
        open={formOpen}
        onOpenChange={setFormOpen}
        model={editing}
        onSave={handleSave}
      />
    </>
  )
}
