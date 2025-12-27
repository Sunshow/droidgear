import { useState, useEffect } from 'react'
import { Plus, Save, AlertCircle, FileText, RefreshCw } from 'lucide-react'
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
import { useTranslation } from 'react-i18next'

export function ModelConfigPage() {
  const { t } = useTranslation()
  const {
    models,
    configPath,
    hasChanges,
    isLoading,
    error,
    configParseError,
    loadModels,
    saveModels,
    resetConfigAndSave,
    addModel,
    updateModel,
    deleteModel,
    setError,
    clearConfigParseError,
  } = useModelStore()

  const [dialogOpen, setDialogOpen] = useState(false)
  const [editingIndex, setEditingIndex] = useState<number | null>(null)
  const [copyingModel, setCopyingModel] = useState<CustomModel | null>(null)
  const [deleteIndex, setDeleteIndex] = useState<number | null>(null)
  const [showRefreshConfirm, setShowRefreshConfirm] = useState(false)

  useEffect(() => {
    loadModels()
  }, [loadModels])

  const handleAdd = () => {
    setEditingIndex(null)
    setCopyingModel(null)
    setDialogOpen(true)
  }

  const handleRefresh = () => {
    if (hasChanges) {
      setShowRefreshConfirm(true)
    } else {
      loadModels()
    }
  }

  const handleConfirmRefresh = () => {
    setShowRefreshConfirm(false)
    loadModels()
  }

  const handleEdit = (index: number) => {
    setEditingIndex(index)
    setCopyingModel(null)
    setDialogOpen(true)
  }

  const handleCopy = (index: number) => {
    const modelToCopy = models[index]
    if (modelToCopy) {
      setCopyingModel(modelToCopy)
      setEditingIndex(null)
      setDialogOpen(true)
    }
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
      <div className="flex items-center justify-between gap-2 p-4 border-b">
        <div className="min-w-0 flex-1">
          <h1 className="text-xl font-semibold">{t('models.title')}</h1>
          <div className="flex items-center gap-2 mt-1 text-sm text-muted-foreground">
            <FileText className="h-4 w-4 flex-shrink-0" />
            <span className="truncate">{configPath}</span>
            {hasChanges && (
              <Badge
                variant="secondary"
                className="bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200 flex-shrink-0"
              >
                {t('models.unsavedChanges')}
              </Badge>
            )}
          </div>
        </div>
        <div className="flex items-center gap-2 flex-shrink-0">
          <Button
            variant="outline"
            size="icon"
            onClick={handleRefresh}
            disabled={isLoading}
            title={t('models.refresh')}
          >
            <RefreshCw className="h-4 w-4" />
          </Button>
          <Button
            variant="outline"
            size="icon"
            onClick={handleAdd}
            title={t('models.addModel')}
          >
            <Plus className="h-4 w-4" />
          </Button>
          <Button
            onClick={saveModels}
            disabled={!hasChanges || isLoading}
            title={t('common.save')}
          >
            <Save className="h-4 w-4 mr-2" />
            {t('common.save')}
          </Button>
        </div>
      </div>

      {/* Error Alert */}
      {error && (
        <div className="mx-4 mt-4 p-3 bg-destructive/10 border border-destructive/20 rounded-md flex items-center gap-2">
          <AlertCircle className="h-4 w-4 text-destructive" />
          <span className="text-sm text-destructive">{error}</span>
          <Button
            variant="ghost"
            size="sm"
            className="ml-auto"
            onClick={() => setError(null)}
          >
            {t('common.dismiss')}
          </Button>
        </div>
      )}

      {/* Model List */}
      <div className="flex-1 overflow-auto p-4">
        <ModelList
          onEdit={handleEdit}
          onDelete={setDeleteIndex}
          onCopy={handleCopy}
        />
      </div>

      {/* Model Dialog */}
      <ModelDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        model={
          editingIndex !== null
            ? models[editingIndex]
            : (copyingModel ?? undefined)
        }
        mode={
          editingIndex !== null
            ? 'edit'
            : copyingModel !== null
              ? 'duplicate'
              : 'add'
        }
        onSave={handleSaveModel}
      />

      {/* Delete Confirmation */}
      <AlertDialog
        open={deleteIndex !== null}
        onOpenChange={() => setDeleteIndex(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('models.deleteModel')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('models.deleteConfirm')}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction onClick={handleConfirmDelete}>
              {t('common.delete')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Config Parse Error Confirmation */}
      <AlertDialog
        open={configParseError !== null}
        onOpenChange={() => clearConfigParseError()}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {t('models.configParseError.title', 'Config File Error')}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {t(
                'models.configParseError.description',
                'The settings.json file is corrupted or invalid. Would you like to reset it and save your current models? This will remove any other settings in the file.'
              )}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>
              {t('models.configParseError.cancel', 'Cancel')}
            </AlertDialogCancel>
            <AlertDialogAction onClick={resetConfigAndSave}>
              {t('models.configParseError.confirm', 'Reset and Save')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Refresh Confirmation */}
      <AlertDialog
        open={showRefreshConfirm}
        onOpenChange={setShowRefreshConfirm}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {t('models.refreshConfirm.title')}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {t('models.refreshConfirm.description')}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction onClick={handleConfirmRefresh}>
              {t('models.refreshConfirm.confirm')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
