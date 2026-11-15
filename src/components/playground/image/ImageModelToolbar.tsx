import { useTranslation } from 'react-i18next'
import { useImagePlaygroundStore } from '@/store/image-playground-store'
import { ModelToolbar } from '../shared/model-toolbar'
import { supportsMultipleImages } from './image-protocol'

export function ImageModelToolbar({
  onAddModel,
  onManageModels,
}: {
  onAddModel: () => void
  onManageModels: () => void
}) {
  const { t } = useTranslation()
  const models = useImagePlaygroundStore(state => state.imageModels)
  const modelsLoading = useImagePlaygroundStore(
    state => state.imageModelsLoading
  )
  const draft = useImagePlaygroundStore(state => state.imageDraft)
  const updateDraft = useImagePlaygroundStore(state => state.updateImageDraft)

  return (
    <ModelToolbar
      models={models}
      selectedId={draft.modelId}
      loading={modelsLoading}
      onSelect={modelId => {
        const next = models.find(model => model.id === modelId)
        updateDraft({
          modelId,
          size: '',
          quality: '',
          count: next && supportsMultipleImages(next) ? draft.count : 1,
          imageUrl: next && !next.supportsImageInput ? '' : draft.imageUrl,
        })
      }}
      onAdd={onAddModel}
      onManage={onManageModels}
      ariaLabel={t('playground.image.create.model')}
      placeholder={t('playground.image.create.modelPlaceholder')}
      addLabel={t('playground.image.addModel')}
      manageLabel={t('playground.image.manageModels')}
    />
  )
}
