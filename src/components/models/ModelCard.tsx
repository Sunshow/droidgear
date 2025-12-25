import { useSortable } from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { GripVertical, Pencil, Trash2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import type { CustomModel } from '@/lib/bindings'

interface ModelCardProps {
  model: CustomModel
  index: number
  onEdit: () => void
  onDelete: () => void
}

const providerLabels: Record<string, string> = {
  anthropic: 'Anthropic',
  openai: 'OpenAI',
  'generic-chat-completion-api': 'Generic',
}

const providerColors: Record<string, string> = {
  anthropic: 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200',
  openai: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
  'generic-chat-completion-api': 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
}

export function ModelCard({ model, index, onEdit, onDelete }: ModelCardProps) {
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
      <button
        className="cursor-grab touch-none text-muted-foreground hover:text-foreground"
        {...attributes}
        {...listeners}
      >
        <GripVertical className="h-5 w-5" />
      </button>

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
