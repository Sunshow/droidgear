import { useTranslation } from 'react-i18next'
import { Loader2, Plus, Settings2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { usePlaygroundStore } from '@/store/playground-store'

/**
 * The model picker and its management actions live in the page header so the
 * create form can start with the prompt. The picker's last item adds a model
 * and Manage opens the list, so there is no separate add button.
 */
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
  const selectedModel = models.find(model => model.id === draft.modelId)

  return (
    <div className="flex flex-wrap items-center justify-end gap-2">
      {modelsLoading && models.length === 0 ? (
        <div className="flex h-8 items-center gap-2 px-2 text-sm text-muted-foreground">
          <Loader2 className="size-4 animate-spin" />
          {t('common.loading')}
        </div>
      ) : (
        <Select
          value={selectedModel ? draft.modelId : ''}
          onValueChange={value => {
            if (value === '__add__') {
              onAddModel()
              return
            }
            updateDraft({ modelId: value, duration: '', resolution: '' })
          }}
        >
          <SelectTrigger
            className="h-8 w-56 max-w-full"
            aria-label={t('playground.video.create.model')}
          >
            <SelectValue
              placeholder={t('playground.video.create.modelPlaceholder')}
            >
              {selectedModel?.name}
            </SelectValue>
          </SelectTrigger>
          <SelectContent>
            {models.map(model => (
              <SelectItem key={model.id} value={model.id}>
                <span className="truncate">{model.name}</span>
                <span className="truncate font-mono text-xs text-muted-foreground">
                  {model.modelId}
                </span>
              </SelectItem>
            ))}
            <SelectItem value="__add__" className="text-primary">
              <Plus />
              {t('playground.video.addModel')}
            </SelectItem>
          </SelectContent>
        </Select>
      )}
      <Button
        variant="ghost"
        size="sm"
        title={t('playground.video.manageModels')}
        onClick={onManageModels}
      >
        <Settings2 />
        {t('playground.video.manageModels')}
      </Button>
    </div>
  )
}
