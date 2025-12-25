import { useState, useEffect } from 'react'
import { Plus, Save, AlertCircle, FileText } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { ModelList } from './ModelList'
import { ModelDialog } from './ModelDialog'
import { useModelStore } from '@/store/model-store'
import type { CustomModel } from '@/lib/bindings'

export function ModelConfigPage() {
  const {
    models,
    configPath,
    hasChanges,
    isLoading,
    error,
    loadModels,
    saveModels,
    addModel,
    updateModel,
    deleteModel,
    setError,
  } = useModelStore()

  const [dialogOpen, setDialogOpen] = useState(false)
  const [editingIndex, setEditingIndex] = useState<number | null>(null)
  const [deleteIndex, setDeleteIndex] = useState<number | null>(null)

  useEffect(() => {
    loadModels()
  }, [loadModels])

  const handleAdd = () => {
    setEditingIndex(null)
    setDialogOpen(true)
  }

  const handleEdit = (index: number) => {
    setEditingIndex(index)
    setDialogOpen(true)
  }

  const handleSaveModel = (model: CustomModel) => {
    if (editingIndex !== null) {
      updateModel(editingIndex, model)
    } else {
      addModel(model)
    }
  }

  const handleConfirmDelete = () => {
    if (deleteIndex !== null) {
      deleteModel(deleteIndex)
      setDeleteIndex(null)
    }
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b">
        <div>
          <h1 className="text-xl font-semibold">BYOK Model Configuration</h1>
          <div className="flex items-center gap-2 mt-1 text-sm text-muted-foreground">
            <FileText className="h-4 w-4" />
            <span>{configPath}</span>
            {hasChanges && (
              <Badge variant="secondary" className="bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200">
                Unsaved Changes
              </Badge>
            )}
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" onClick={handleAdd}>
            <Plus className="h-4 w-4 mr-2" />
            Add Model
          </Button>
          <Button onClick={saveModels} disabled={!hasChanges || isLoading}>
            <Save className="h-4 w-4 mr-2" />
            Save
          </Button>
        </div>
      </div>

      {/* Error Alert */}
      {error && (
        <div className="mx-4 mt-4 p-3 bg-destructive/10 border border-destructive/20 rounded-md flex items-center gap-2">
          <AlertCircle className="h-4 w-4 text-destructive" />
          <span className="text-sm text-destructive">{error}</span>
          <Button variant="ghost" size="sm" className="ml-auto" onClick={() => setError(null)}>
            Dismiss
          </Button>
        </div>
      )}

      {/* Model List */}
      <div className="flex-1 overflow-auto p-4">
        <ModelList onEdit={handleEdit} onDelete={setDeleteIndex} />
      </div>

      {/* Model Dialog */}
      <ModelDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        model={editingIndex !== null ? models[editingIndex] : undefined}
        onSave={handleSaveModel}
      />

      {/* Delete Confirmation */}
      <AlertDialog open={deleteIndex !== null} onOpenChange={() => setDeleteIndex(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Model</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete this model? This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleConfirmDelete}>Delete</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
