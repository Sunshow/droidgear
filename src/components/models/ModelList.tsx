import { useTranslation } from 'react-i18next'
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
import { useConnectivityStore } from '@/store/connectivity-store'
import type { CustomModel } from '@/lib/bindings'

interface FilteredModel {
  model: CustomModel
  originalIndex: number
}

interface ModelListProps {
  onEdit: (index: number) => void
  onDelete: (index: number) => void
  onCopy: (index: number) => void
  onSetDefault: (index: number) => void
  onToggleFavorite?: (modelId: string) => void
  onTestConnection?: (modelId: string) => void
  filteredModels?: FilteredModel[]
  selectionMode?: boolean
  selectedIndices?: Set<number>
  onSelect?: (index: number, selected: boolean) => void
  defaultModelId?: string | null
  specModeModelId?: string | null
  favoriteModelIds?: Set<string>
}

export function ModelList({
  onEdit,
  onDelete,
  onCopy,
  onSetDefault,
  onToggleFavorite,
  onTestConnection: onTestConnectionProp,
  filteredModels,
  selectionMode = false,
  selectedIndices = new Set(),
  onSelect,
  defaultModelId,
  specModeModelId,
  favoriteModelIds,
}: ModelListProps) {
  const { t } = useTranslation()
  const { models, reorderModels } = useModelStore()
  const { testSingleModel } = useConnectivityStore()

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

  const handleTestConnection = async (modelId: string | null | undefined) => {
    if (!modelId) return
    try {
      if (onTestConnectionProp) {
        onTestConnectionProp(modelId)
      }
      await testSingleModel(modelId)
    } catch (error) {
      console.error('Failed to test model connection:', error)
    }
  }

  // Use filtered models if provided, otherwise use all models
  const isFiltered = filteredModels !== undefined
  const displayModels = isFiltered
    ? filteredModels
    : models.map((model, index) => ({ model, originalIndex: index }))

  if (models.length === 0) {
    return (
      <div className="text-center py-12 text-muted-foreground">
        <p>{t('models.noModels')}</p>
        <p className="text-sm mt-1">{t('models.noModelsHint')}</p>
      </div>
    )
  }

  if (displayModels.length === 0) {
    return (
      <div className="text-center py-12 text-muted-foreground">
        <p>{t('models.noMatchingModels')}</p>
      </div>
    )
  }

  // Disable drag when filtering or in selection mode
  const isDragDisabled = isFiltered || selectionMode

  // Helper function to check if a model is the default
  const isModelDefault = (model: CustomModel) => {
    if (!defaultModelId) return false
    return model.id === defaultModelId
  }

  const isModelSpecMode = (model: CustomModel) => {
    if (!specModeModelId) return false
    return model.id === specModeModelId
  }

  if (isDragDisabled) {
    return (
      <div className="space-y-2">
        {displayModels.map(({ model, originalIndex }) => (
          <ModelCard
            key={`model-${originalIndex}`}
            model={model}
            index={originalIndex}
            selectionMode={selectionMode}
            isSelected={selectedIndices.has(originalIndex)}
            isDefault={isModelDefault(model)}
            isSpecMode={isModelSpecMode(model)}
            isFavorite={favoriteModelIds?.has(model.id ?? '') ?? false}
            onSelect={onSelect}
            onEdit={() => onEdit(originalIndex)}
            onDelete={() => onDelete(originalIndex)}
            onCopy={() => onCopy(originalIndex)}
            onSetDefault={() => onSetDefault(originalIndex)}
            onToggleFavorite={
              onToggleFavorite && model.id
                ? () => onToggleFavorite?.(model.id as string)
                : undefined
            }
            onTestConnection={() => handleTestConnection(model.id)}
          />
        ))}
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
        items={displayModels.map(
          ({ originalIndex }) => `model-${originalIndex}`
        )}
        strategy={verticalListSortingStrategy}
      >
        <div className="space-y-2">
          {displayModels.map(({ model, originalIndex }) => (
            <ModelCard
              key={`model-${originalIndex}`}
              model={model}
              index={originalIndex}
              selectionMode={selectionMode}
              isSelected={selectedIndices.has(originalIndex)}
              isDefault={isModelDefault(model)}
              isSpecMode={isModelSpecMode(model)}
              isFavorite={favoriteModelIds?.has(model.id ?? '') ?? false}
              onSelect={onSelect}
              onEdit={() => onEdit(originalIndex)}
              onDelete={() => onDelete(originalIndex)}
              onCopy={() => onCopy(originalIndex)}
              onSetDefault={() => onSetDefault(originalIndex)}
              onToggleFavorite={
                onToggleFavorite && model.id
                  ? () => onToggleFavorite?.(model.id as string)
                  : undefined
              }
              onTestConnection={() => handleTestConnection(model.id)}
            />
          ))}
        </div>
      </SortableContext>
    </DndContext>
  )
}
