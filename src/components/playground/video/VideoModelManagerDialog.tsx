import { useEffect, useState } from 'react'
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
  const channelsLoading = useChannelStore(state => state.isLoading)
  const loadChannels = useChannelStore(state => state.loadChannels)
  const [editing, setEditing] = useState<VideoModel | undefined>()
  const [formOpen, setFormOpen] = useState(false)

  useEffect(() => {
    // The channel list may not have been loaded in this view yet; without it
    // every channel-backed model would briefly look like a deleted channel.
    if (open && channels.length === 0) void loadChannels()
  }, [open, channels.length, loadChannels])

  const sourceLabel = (model: VideoModel) => {
    if (model.source.kind !== 'channel') return model.source.baseUrl
    const channelId = model.source.channelId
    const channel = channels.find(candidate => candidate.id === channelId)
    if (channel) return channel.name
    return channelsLoading
      ? t('playground.video.models.loadingChannel')
      : t('playground.video.models.missingChannel')
  }

  const handleSave = (draft: VideoModelDraft) => {
    const store = usePlaygroundStore.getState()
    if (editing) void store.updateVideoModel(editing.id, draft)
    else void store.addVideoModel(draft)
  }

  const protocolLabel = (model: VideoModel) => {
    switch (model.protocol) {
      case 'new-api':
        return t('playground.video.modelDialog.protocolNewApi')
      case 'doubao':
        return t('playground.video.modelDialog.protocolDoubao')
      case 'sora':
      default:
        return t('playground.video.modelDialog.protocolSora')
    }
  }

  return (
    <>
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent className="sm:max-w-3xl">
          <DialogHeader>
            <DialogTitle>{t('playground.video.models.title')}</DialogTitle>
            <DialogDescription>
              {t('playground.video.models.description')}
            </DialogDescription>
          </DialogHeader>

          <div className="max-h-[60vh] divide-y overflow-auto rounded-md border">
            {models.length === 0 ? (
              <p className="p-6 text-center text-sm text-muted-foreground">
                {t('playground.video.models.empty')}
              </p>
            ) : (
              models.map(model => (
                <div key={model.id} className="flex items-center gap-3 p-3">
                  <div className="min-w-0 flex-1 space-y-1">
                    <div className="flex min-w-0 items-center gap-2">
                      <span
                        className="min-w-0 flex-1 truncate font-medium"
                        title={model.name}
                      >
                        {model.name}
                      </span>
                      <Badge variant="outline" className="shrink-0">
                        {protocolLabel(model)}
                      </Badge>
                    </div>
                    <div
                      className="truncate font-mono text-xs text-muted-foreground"
                      title={model.modelId}
                    >
                      {model.modelId}
                    </div>
                    <p className="truncate text-xs text-muted-foreground">
                      {sourceLabel(model)}
                      {model.durations.length > 0 &&
                        ` · ${model.durations.map(d => `${d}s`).join('/')}`}
                      {model.supportsImageInput &&
                        ` · ${t('playground.video.models.imageInput')}`}
                    </p>
                  </div>
                  <div className="flex shrink-0 items-center gap-1">
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
                        void usePlaygroundStore
                          .getState()
                          .deleteVideoModel(model.id)
                      }
                    >
                      <Trash2 />
                    </Button>
                  </div>
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
        onSaveMany={drafts =>
          usePlaygroundStore.getState().addVideoModels(drafts)
        }
      />
    </>
  )
}
