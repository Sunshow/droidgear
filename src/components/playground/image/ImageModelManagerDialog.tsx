import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useChannelStore } from '@/store/channel-store'
import {
  useImagePlaygroundStore,
  type ImageModel,
  type ImageModelDraft,
} from '@/store/image-playground-store'
import { ModelManagerDialog } from '../shared/model-manager-dialog'
import { ImageModelDialog } from './ImageModelDialog'
import { imageProtocolLabel } from './image-protocol'

export function ImageModelManagerDialog({
  open,
  onOpenChange,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
}) {
  const { t } = useTranslation()
  const models = useImagePlaygroundStore(state => state.imageModels)
  const channels = useChannelStore(state => state.channels)
  const channelsLoading = useChannelStore(state => state.isLoading)
  const loadChannels = useChannelStore(state => state.loadChannels)
  const [editing, setEditing] = useState<ImageModel | undefined>()
  const [formOpen, setFormOpen] = useState(false)

  useEffect(() => {
    if (open && channels.length === 0) void loadChannels()
  }, [open, channels.length, loadChannels])

  const sourceLabel = (model: ImageModel) => {
    const source = model.source
    if (source.kind !== 'channel') return source.baseUrl
    const channel = channels.find(
      candidate => candidate.id === source.channelId
    )
    if (channel) return channel.name
    return channelsLoading
      ? t('playground.image.models.loadingChannel')
      : t('playground.image.models.missingChannel')
  }

  const handleSave = (draft: ImageModelDraft) => {
    const store = useImagePlaygroundStore.getState()
    if (editing) void store.updateImageModel(editing.id, draft)
    else void store.addImageModel(draft)
  }

  return (
    <>
      <ModelManagerDialog<ImageModel>
        open={open}
        onOpenChange={onOpenChange}
        title={t('playground.image.models.title')}
        description={t('playground.image.models.description')}
        emptyLabel={t('playground.image.models.empty')}
        addLabel={t('playground.image.addModel')}
        models={models}
        getId={model => model.id}
        getName={model => model.name}
        getModelId={model => model.modelId}
        protocolLabel={model => imageProtocolLabel(t, model.protocol)}
        sourceLabel={sourceLabel}
        detailLabel={model =>
          [
            model.sizes.length > 0 ? model.sizes.slice(0, 4).join('/') : null,
            model.qualities.length > 0
              ? model.qualities.slice(0, 4).join('/')
              : null,
            model.supportsImageInput
              ? t('playground.image.models.imageInput')
              : null,
          ]
            .filter(Boolean)
            .join(' · ')
        }
        onAdd={() => {
          setEditing(undefined)
          setFormOpen(true)
        }}
        onEdit={model => {
          setEditing(model)
          setFormOpen(true)
        }}
        onDelete={id =>
          void useImagePlaygroundStore.getState().deleteImageModel(id)
        }
      />

      <ImageModelDialog
        open={formOpen}
        onOpenChange={setFormOpen}
        model={editing}
        onSave={handleSave}
        onSaveMany={drafts =>
          useImagePlaygroundStore.getState().addImageModels(drafts)
        }
      />
    </>
  )
}
