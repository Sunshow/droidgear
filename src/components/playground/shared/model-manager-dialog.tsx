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

interface ModelManagerDialogProps<TModel> {
  open: boolean
  onOpenChange: (open: boolean) => void
  title: string
  description: string
  emptyLabel: string
  addLabel: string
  models: TModel[]
  getId: (model: TModel) => string
  getName: (model: TModel) => string
  getModelId: (model: TModel) => string
  protocolLabel: (model: TModel) => string
  sourceLabel: (model: TModel) => string
  /** Optional one-line summary, e.g. durations or sizes. */
  detailLabel?: (model: TModel) => string
  onAdd: () => void
  onEdit: (model: TModel) => void
  onDelete: (id: string) => void
}

/** Shared model list dialog: edit and delete entries, with an add button at
 * the bottom so the empty state stays reachable. */
export function ModelManagerDialog<TModel>({
  open,
  onOpenChange,
  title,
  description,
  emptyLabel,
  addLabel,
  models,
  getId,
  getName,
  getModelId,
  protocolLabel,
  sourceLabel,
  detailLabel,
  onAdd,
  onEdit,
  onDelete,
}: ModelManagerDialogProps<TModel>) {
  const { t } = useTranslation()

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-3xl">
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>

        <div className="max-h-[60vh] divide-y overflow-auto rounded-md border">
          {models.length === 0 ? (
            <p className="p-6 text-center text-sm text-muted-foreground">
              {emptyLabel}
            </p>
          ) : (
            models.map(model => {
              const detail = detailLabel?.(model)
              return (
                <div key={getId(model)} className="flex items-center gap-3 p-3">
                  <div className="min-w-0 flex-1 space-y-1">
                    <div className="flex min-w-0 items-center gap-2">
                      <span
                        className="min-w-0 flex-1 truncate font-medium"
                        title={getName(model)}
                      >
                        {getName(model)}
                      </span>
                      <Badge variant="outline" className="shrink-0">
                        {protocolLabel(model)}
                      </Badge>
                    </div>
                    <div
                      className="truncate font-mono text-xs text-muted-foreground"
                      title={getModelId(model)}
                    >
                      {getModelId(model)}
                    </div>
                    <p className="truncate text-xs text-muted-foreground">
                      {[sourceLabel(model), detail].filter(Boolean).join(' · ')}
                    </p>
                  </div>
                  <div className="flex shrink-0 items-center gap-1">
                    <Button
                      variant="ghost"
                      size="icon-sm"
                      title={t('common.edit')}
                      onClick={() => onEdit(model)}
                    >
                      <Pencil />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon-sm"
                      title={t('common.delete')}
                      onClick={() => onDelete(getId(model))}
                    >
                      <Trash2 />
                    </Button>
                  </div>
                </div>
              )
            })
          )}
        </div>

        <div className="flex justify-end">
          <Button onClick={onAdd}>
            <Plus />
            {addLabel}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}
