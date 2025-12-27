import { useSortable } from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { GripVertical, Pencil, Trash2, Copy } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Checkbox } from '@/components/ui/checkbox'
import { providerColors, providerLabels } from '@/lib/platform-colors'
import type { CustomModel } from '@/lib/bindings'

interface ModelCardProps {
  model: CustomModel
  index: number
  selectionMode?: boolean
  isSelected?: boolean
  onSelect?: (index: number, selected: boolean) => void
  onEdit: () => void
  onDelete: () => void
  onCopy: () => void
}

export function ModelCard({
  model,
  index,
  selectionMode = false,
  isSelected = false,
  onSelect,
  onEdit,
  onDelete,
  onCopy,
}: ModelCardProps) {
  const { t } = useTranslation()
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: `model-${index}` })

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  }

  const displayName = model.displayName || model.model

  return (
    <Card
      ref={setNodeRef}
      style={style}
      className="flex items-center gap-3 p-3 mb-2"
    >
      {selectionMode ? (
        <Checkbox
          checked={isSelected}
          onCheckedChange={checked => onSelect?.(index, checked === true)}
          className="h-5 w-5"
        />
      ) : (
        <button
          className="cursor-grab touch-none text-muted-foreground hover:text-foreground"
          {...attributes}
          {...listeners}
        >
          <GripVertical className="h-5 w-5" />
        </button>
      )}

      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="font-medium truncate">{displayName}</span>
          <Badge variant="secondary" className={providerColors[model.provider]}>
            {providerLabels[model.provider]}
          </Badge>
        </div>
        <div className="text-sm text-muted-foreground truncate">
          {model.model} • {model.baseUrl}
        </div>
      </div>

      <div className="flex items-center gap-1">
        <Button
          variant="ghost"
          size="icon"
          onClick={onCopy}
          title={t('models.duplicateModel')}
        >
          <Copy className="h-4 w-4" />
        </Button>
        <Button variant="ghost" size="icon" onClick={onEdit}>
          <Pencil className="h-4 w-4" />
        </Button>
        <Button variant="ghost" size="icon" onClick={onDelete}>
          <Trash2 className="h-4 w-4 text-destructive" />
        </Button>
      </div>
    </Card>
  )
}
