import { Loader2, Plus, Settings2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'

interface ModelToolbarProps {
  models: { id: string; name: string; modelId: string }[]
  selectedId: string
  loading: boolean
  onSelect: (id: string) => void
  onAdd: () => void
  onManage: () => void
  ariaLabel: string
  placeholder: string
  addLabel: string
  manageLabel: string
}

/**
 * The model picker and its management actions live in the page header so the
 * create form can start with the prompt. The picker's last item adds a model
 * and Manage opens the list, so there is no separate add button.
 */
export function ModelToolbar({
  models,
  selectedId,
  loading,
  onSelect,
  onAdd,
  onManage,
  ariaLabel,
  placeholder,
  addLabel,
  manageLabel,
}: ModelToolbarProps) {
  const { t } = useTranslation()
  const selectedModel = models.find(model => model.id === selectedId)

  return (
    <div className="flex flex-wrap items-center justify-end gap-2">
      {loading && models.length === 0 ? (
        <div className="flex h-8 items-center gap-2 px-2 text-sm text-muted-foreground">
          <Loader2 className="size-4 animate-spin" />
          {t('common.loading')}
        </div>
      ) : (
        <Select
          value={selectedModel ? selectedId : ''}
          onValueChange={value => {
            if (value === '__add__') {
              onAdd()
              return
            }
            onSelect(value)
          }}
        >
          <SelectTrigger className="h-8 w-56 max-w-full" aria-label={ariaLabel}>
            <SelectValue placeholder={placeholder}>
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
              {addLabel}
            </SelectItem>
          </SelectContent>
        </Select>
      )}
      <Button variant="ghost" size="sm" title={manageLabel} onClick={onManage}>
        <Settings2 />
        {manageLabel}
      </Button>
    </div>
  )
}
