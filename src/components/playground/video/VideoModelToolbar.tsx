import { useTranslation } from 'react-i18next'
import { usePlaygroundStore } from '@/store/playground-store'
import { ModelToolbar } from '../shared/model-toolbar'

export function VideoModelToolbar({
  onAddModel,
  onManageModels,
}: {
  onAddModel: () => void
  onManageModels: () => void
}) {
  const { t } = useTranslation()
  const models = usePlaygroundStore(state => state.videoModels)
  const modelsLoading = usePlaygroundStore(state => state.videoModelsLoading)
  const draft = usePlaygroundStore(state => state.videoDraft)
  const updateDraft = usePlaygroundStore(state => state.updateVideoDraft)

  return (
    <ModelToolbar
      models={models}
      selectedId={draft.modelId}
      loading={modelsLoading}
      onSelect={modelId =>
        updateDraft({ modelId, duration: '', resolution: '' })
      }
      onAdd={onAddModel}
      onManage={onManageModels}
      ariaLabel={t('playground.video.create.model')}
      placeholder={t('playground.video.create.modelPlaceholder')}
      addLabel={t('playground.video.addModel')}
      manageLabel={t('playground.video.manageModels')}
    />
  )
}
