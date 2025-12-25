import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
} from '@dnd-kit/core'
import {
  SortableContext,
  sortableKeyboardCoordinates,
  verticalListSortingStrategy,
} from '@dnd-kit/sortable'
import { ModelCard } from './ModelCard'
import { useModelStore } from '@/store/model-store'

interface ModelListProps {
  onEdit: (index: number) => void
  onDelete: (index: number) => void
}

export function ModelList({ onEdit, onDelete }: ModelListProps) {
  const { models, reorderModels } = useModelStore()

  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  )

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event
    if (over && active.id !== over.id) {
      const oldIndex = parseInt(String(active.id).replace('model-', ''))
      const newIndex = parseInt(String(over.id).replace('model-', ''))
      reorderModels(oldIndex, newIndex)
    }
  }

  if (models.length === 0) {
    return (
      <div className="text-center py-12 text-muted-foreground">
        <p>No custom models configured</p>
        <p className="text-sm mt-1">Click &quot;Add Model&quot; to get started</p>
      </div>
    )
  }

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCenter}
      onDragEnd={handleDragEnd}
    >
      <SortableContext
        items={models.map((_, i) => `model-${i}`)}
        strategy={verticalListSortingStrategy}
      >
        <div className="space-y-2">
          {models.map((model, index) => (
            <ModelCard
              key={`model-${index}`}
              model={model}
              index={index}
              onEdit={() => onEdit(index)}
              onDelete={() => onDelete(index)}
            />
          ))}
        </div>
      </SortableContext>
    </DndContext>
  )
}
