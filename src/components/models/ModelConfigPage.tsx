import { useState, useEffect, useMemo } from 'react'
import {
  Plus,
  Save,
  AlertCircle,
  FileText,
  RefreshCw,
  Search,
  Trash2,
  X,
  CheckSquare,
  Download,
  Upload,
} from 'lucide-react'
import { save, open } from '@tauri-apps/plugin-dialog'
import { writeTextFile, readTextFile } from '@tauri-apps/plugin-fs'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
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
import { ModelImportDialog, type MergeStrategy } from './ModelImportDialog'
import { useModelStore } from '@/store/model-store'
import type { CustomModel } from '@/lib/bindings'
import { useTranslation } from 'react-i18next'

interface ExportData {
  version: number
  exportedAt: string
  models: CustomModel[]
}

export function ModelConfigPage() {
  const { t } = useTranslation()
  const {
    models,
    configPath,
    hasChanges,
    isLoading,
    error,
    configParseError,
    defaultModelId,
    loadModels,
    saveModels,
    resetConfigAndSave,
    addModel,
    updateModel,
    deleteModel,
    deleteModels,
    setError,
    clearConfigParseError,
    loadDefaultModel,
    setDefaultModel,
  } = useModelStore()

  const [dialogOpen, setDialogOpen] = useState(false)
  const [editingIndex, setEditingIndex] = useState<number | null>(null)
  const [copyingModel, setCopyingModel] = useState<CustomModel | null>(null)
  const [deleteIndex, setDeleteIndex] = useState<number | null>(null)
  const [showRefreshConfirm, setShowRefreshConfirm] = useState(false)

  // Filter state
  const [filterText, setFilterText] = useState('')
  const [filterProvider, setFilterProvider] = useState<string>('all')

  // Selection mode state
  const [selectionMode, setSelectionMode] = useState(false)
  const [selectedIndices, setSelectedIndices] = useState<Set<number>>(new Set())
  const [showBatchDeleteConfirm, setShowBatchDeleteConfirm] = useState(false)

  // Import/Export state
  const [importDialogOpen, setImportDialogOpen] = useState(false)
  const [importModels, setImportModels] = useState<CustomModel[]>([])

  useEffect(() => {
    loadModels()
    loadDefaultModel()
  }, [loadModels, loadDefaultModel])

  // Filter models based on search text and provider
  const filteredModels = useMemo(() => {
    const searchLower = filterText.toLowerCase()
    const hasFilter = filterText || filterProvider !== 'all'

    if (!hasFilter) {
      return undefined // Return undefined to indicate no filtering
    }

    return models
      .map((model, index) => ({ model, originalIndex: index }))
      .filter(({ model }) => {
        // Provider filter
        if (filterProvider !== 'all' && model.provider !== filterProvider) {
          return false
        }
        // Text filter
        if (filterText) {
          const matchesModel = model.model.toLowerCase().includes(searchLower)
          const matchesDisplayName = model.displayName
            ?.toLowerCase()
            .includes(searchLower)
          const matchesBaseUrl = model.baseUrl
            .toLowerCase()
            .includes(searchLower)
          if (!matchesModel && !matchesDisplayName && !matchesBaseUrl) {
            return false
          }
        }
        return true
      })
  }, [models, filterText, filterProvider])

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
      loadDefaultModel()
    }
  }

  const handleConfirmRefresh = () => {
    setShowRefreshConfirm(false)
    loadModels()
    loadDefaultModel()
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

  const handleSaveModel = async (model: CustomModel) => {
    if (editingIndex !== null) {
      updateModel(editingIndex, model)
    } else {
      addModel(model)
    }
    await saveModels()
  }

  const handleSaveBatchModels = async (batchModels: CustomModel[]) => {
    for (const model of batchModels) {
      addModel(model)
    }
    await saveModels()
  }

  const handleConfirmDelete = () => {
    if (deleteIndex !== null) {
      deleteModel(deleteIndex)
      setDeleteIndex(null)
    }
  }

  // Selection mode handlers
  const handleEnterSelectionMode = () => {
    setSelectionMode(true)
    setSelectedIndices(new Set())
  }

  const handleExitSelectionMode = () => {
    setSelectionMode(false)
    setSelectedIndices(new Set())
  }

  const handleSelect = (index: number, selected: boolean) => {
    setSelectedIndices(prev => {
      const next = new Set(prev)
      if (selected) {
        next.add(index)
      } else {
        next.delete(index)
      }
      return next
    })
  }

  const handleSelectAll = () => {
    const indicesToSelect = filteredModels
      ? filteredModels.map(({ originalIndex }) => originalIndex)
      : models.map((_, index) => index)
    setSelectedIndices(new Set(indicesToSelect))
  }

  const handleConfirmBatchDelete = () => {
    deleteModels(Array.from(selectedIndices))
    setShowBatchDeleteConfirm(false)
    handleExitSelectionMode()
  }

  // Export models to JSON file
  const handleExport = async () => {
    if (models.length === 0) return

    try {
      const filePath = await save({
        filters: [{ name: 'JSON', extensions: ['json'] }],
        defaultPath: 'models-export.json',
      })

      if (!filePath) return

      const exportData: ExportData = {
        version: 1,
        exportedAt: new Date().toISOString(),
        models: models,
      }

      await writeTextFile(filePath, JSON.stringify(exportData, null, 2))
      toast.success(t('models.export.success'))
    } catch (e) {
      setError(String(e))
    }
  }

  // Import models from JSON file
  const handleImport = async () => {
    try {
      const filePath = await open({
        filters: [{ name: 'JSON', extensions: ['json'] }],
        multiple: false,
      })

      if (!filePath) return

      const content = await readTextFile(filePath)
      const data = JSON.parse(content) as ExportData

      // Validate structure
      if (!data.models || !Array.isArray(data.models)) {
        setError(t('models.import.invalidFormat'))
        return
      }

      // Validate each model has required fields
      const validModels = data.models.filter(
        m =>
          typeof m.model === 'string' &&
          typeof m.baseUrl === 'string' &&
          typeof m.apiKey === 'string' &&
          typeof m.provider === 'string'
      )

      if (validModels.length === 0) {
        setError(t('models.import.noValidModels'))
        return
      }

      setImportModels(validModels)
      setImportDialogOpen(true)
    } catch (e) {
      setError(String(e))
    }
  }

  // Handle import confirmation with merge strategy
  const handleImportConfirm = (
    selectedModels: CustomModel[],
    strategy: MergeStrategy
  ) => {
    const findDuplicateIndex = (model: CustomModel) =>
      models.findIndex(
        existing =>
          existing.model === model.model &&
          existing.baseUrl === model.baseUrl &&
          existing.apiKey === model.apiKey
      )

    selectedModels.forEach(model => {
      const dupIndex = findDuplicateIndex(model)
      const isDup = dupIndex !== -1

      if (!isDup) {
        // Not a duplicate, always add
        addModel(model)
      } else {
        // Handle based on strategy
        switch (strategy) {
          case 'skip':
            // Do nothing
            break
          case 'replace':
            updateModel(dupIndex, model)
            break
          case 'keep-both':
            addModel(model)
            break
        }
      }
    })

    setImportDialogOpen(false)
    setImportModels([])
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
            onClick={handleImport}
            title={t('models.import.button')}
          >
            <Upload className="h-4 w-4" />
          </Button>
          <Button
            variant="outline"
            size="icon"
            onClick={handleExport}
            disabled={models.length === 0}
            title={t('models.export.button')}
          >
            <Download className="h-4 w-4" />
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

      {/* Filter Bar */}
      {models.length > 0 && (
        <div className="flex items-center gap-2 px-4 pt-4">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
            <Input
              placeholder={t('models.search')}
              value={filterText}
              onChange={e => setFilterText(e.target.value)}
              className="pl-9"
            />
          </div>
          <Select value={filterProvider} onValueChange={setFilterProvider}>
            <SelectTrigger className="w-[160px]">
              <SelectValue placeholder={t('models.filterProvider')} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">{t('models.filterProvider')}</SelectItem>
              <SelectItem value="anthropic">
                {t('models.providerAnthropic')}
              </SelectItem>
              <SelectItem value="openai">
                {t('models.providerOpenAI')}
              </SelectItem>
              <SelectItem value="generic-chat-completion-api">
                {t('models.providerGeneric')}
              </SelectItem>
            </SelectContent>
          </Select>
          {!selectionMode ? (
            <Button
              variant="outline"
              size="sm"
              onClick={handleEnterSelectionMode}
              disabled={models.length === 0}
            >
              <CheckSquare className="h-4 w-4 mr-2" />
              {t('models.batchDelete')}
            </Button>
          ) : (
            <div className="flex items-center gap-2">
              <Button variant="outline" size="sm" onClick={handleSelectAll}>
                {t('models.selectAll')}
              </Button>
              <Button
                variant="destructive"
                size="sm"
                onClick={() => setShowBatchDeleteConfirm(true)}
                disabled={selectedIndices.size === 0}
              >
                <Trash2 className="h-4 w-4 mr-2" />
                {t('models.deleteSelected', { count: selectedIndices.size })}
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onClick={handleExitSelectionMode}
              >
                <X className="h-4 w-4" />
              </Button>
            </div>
          )}
        </div>
      )}

      {/* Model List */}
      <div className="flex-1 overflow-auto p-4">
        <ModelList
          onEdit={handleEdit}
          onDelete={setDeleteIndex}
          onCopy={handleCopy}
          onSetDefault={setDefaultModel}
          filteredModels={filteredModels}
          selectionMode={selectionMode}
          selectedIndices={selectedIndices}
          onSelect={handleSelect}
          defaultModelId={defaultModelId}
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
        onSaveBatch={handleSaveBatchModels}
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

      {/* Batch Delete Confirmation */}
      <AlertDialog
        open={showBatchDeleteConfirm}
        onOpenChange={setShowBatchDeleteConfirm}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('models.batchDelete')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('models.batchDeleteConfirm', { count: selectedIndices.size })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction onClick={handleConfirmBatchDelete}>
              {t('common.delete')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Import Dialog */}
      <ModelImportDialog
        open={importDialogOpen}
        onOpenChange={setImportDialogOpen}
        importModels={importModels}
        existingModels={models}
        onImport={handleImportConfirm}
      />
    </div>
  )
}
