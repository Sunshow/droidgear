import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useChannelStore } from '@/store/channel-store'
import {
  usePlaygroundStore,
  type VideoModel,
  type VideoModelDraft,
} from '@/store/playground-store'
import { ModelManagerDialog } from '../shared/model-manager-dialog'
import { VideoModelDialog } from './VideoModelDialog'
import { videoProtocolLabel } from './video-protocol'

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
    const source = model.source
    if (source.kind !== 'channel') return source.baseUrl
    const channel = channels.find(
      candidate => candidate.id === source.channelId
    )
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

  return (
    <>
      <ModelManagerDialog<VideoModel>
        open={open}
        onOpenChange={onOpenChange}
        title={t('playground.video.models.title')}
        description={t('playground.video.models.description')}
        emptyLabel={t('playground.video.models.empty')}
        addLabel={t('playground.video.addModel')}
        models={models}
        getId={model => model.id}
        getName={model => model.name}
        getModelId={model => model.modelId}
        protocolLabel={model => videoProtocolLabel(t, model.protocol)}
        sourceLabel={sourceLabel}
        detailLabel={model => {
          const durations = model.durations ?? []
          return [
            durations.length > 0
              ? durations.map(duration => `${duration}s`).join('/')
              : null,
            model.supportsImageInput
              ? t('playground.video.models.imageInput')
              : null,
          ]
            .filter(Boolean)
            .join(' · ')
        }}
        onAdd={() => {
          setEditing(undefined)
          setFormOpen(true)
        }}
        onEdit={model => {
          setEditing(model)
          setFormOpen(true)
        }}
        onDelete={id => void usePlaygroundStore.getState().deleteVideoModel(id)}
      />

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
