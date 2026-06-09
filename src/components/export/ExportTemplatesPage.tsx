import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  Plus,
  Play,
  Pencil,
  Trash2,
  AlertCircle,
  CheckCircle2,
  FileOutput,
  FolderOpen,
  ArrowLeft,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
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
import { openPath, revealItemInDir } from '@tauri-apps/plugin-opener'
import { toast } from 'sonner'
import { useExportStore } from '@/store/export-store'
import { useUIStore } from '@/store/ui-store'
import type { ExportTemplate } from '@/lib/bindings'
import { ExportTemplateDialog } from './ExportTemplateDialog'
import { Loader2 } from 'lucide-react'

export function ExportTemplatesPage() {
  const { t } = useTranslation()
  const templates = useExportStore(state => state.templates)
  const isLoading = useExportStore(state => state.isLoading)
  const error = useExportStore(state => state.error)
  const runningTemplate = useExportStore(state => state.runningTemplate)
  const lastResult = useExportStore(state => state.lastResult)
  const loadTemplates = useExportStore(state => state.loadTemplates)
  const deleteTemplate = useExportStore(state => state.deleteTemplate)
  const runTemplate = useExportStore(state => state.runTemplate)
  const clearResult = useExportStore(state => state.clearResult)
  const clearError = useExportStore(state => state.clearError)

  const [dialogOpen, setDialogOpen] = useState(false)
  const [editingTemplate, setEditingTemplate] = useState<
    ExportTemplate | undefined
  >()
  const [deleteConfirm, setDeleteConfirm] = useState<string | null>(null)

  useEffect(() => {
    loadTemplates()
  }, [loadTemplates])

  const handleAdd = () => {
    setEditingTemplate(undefined)
    setDialogOpen(true)
  }

  const handleEdit = (template: ExportTemplate) => {
    setEditingTemplate(template)
    setDialogOpen(true)
  }

  const handleDelete = async (name: string) => {
    await deleteTemplate(name)
    setDeleteConfirm(null)
    toast.success(t('export.templateDeleted'))
  }

  const handleRun = async (name: string) => {
    await runTemplate(name)
    const result = useExportStore.getState().lastResult
    if (result) {
      toast.success(t('export.runSuccess', { path: result.outputPath }))
    }
  }

  const handleOpenDir = async (filePath: string) => {
    try {
      // First try to reveal the file in the file manager
      await revealItemInDir(filePath)
    } catch {
      // Fallback: open the parent directory
      const normalized = filePath.replace(/\\/g, '/')
      const lastSlash = normalized.lastIndexOf('/')
      const dir =
        lastSlash >= 0 ? normalized.substring(0, lastSlash) : normalized
      await openPath(dir)
    }
  }

  const formatFormat = (format: string) => {
    return format.toUpperCase()
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b">
        <div className="flex items-center gap-3">
          <Button
            variant="ghost"
            size="icon"
            className="shrink-0"
            onClick={() => useUIStore.getState().setChannelsSubView('detail')}
            title={t('common.back')}
          >
            <ArrowLeft className="h-5 w-5" />
          </Button>
          <div>
            <h1 className="text-xl font-semibold flex items-center gap-2">
              <FileOutput className="h-5 w-5" />
              {t('export.title')}
            </h1>
            <p className="text-sm text-muted-foreground mt-1">
              {t('export.subtitle')}
            </p>
          </div>
        </div>
        <Button onClick={handleAdd}>
          <Plus className="h-4 w-4 mr-2" />
          {t('export.addTemplate')}
        </Button>
      </div>

      {/* Error */}
      {error && (
        <div className="mx-4 mt-4 p-3 bg-destructive/10 border border-destructive/20 rounded-md flex items-center gap-2">
          <AlertCircle className="h-4 w-4 text-destructive shrink-0" />
          <span className="text-sm text-destructive">{error}</span>
          <Button
            variant="ghost"
            size="sm"
            className="ml-auto"
            onClick={() => clearError()}
          >
            {t('common.dismiss')}
          </Button>
        </div>
      )}

      {/* Last run result */}
      {lastResult && (
        <div className="mx-4 mt-4 p-3 bg-green-500/10 border border-green-500/20 rounded-md flex items-center gap-2">
          <CheckCircle2 className="h-4 w-4 text-green-600 shrink-0" />
          <div className="text-sm text-green-700 dark:text-green-400">
            <span>
              {t('export.runResult', {
                channels: lastResult.channelsCount,
                tokens: lastResult.tokensCount,
                models: lastResult.modelsCount,
                records: lastResult.recordCount,
              })}
            </span>
            <span className="ml-2 font-mono text-xs">
              {lastResult.outputPath}
            </span>
            {lastResult.warnings.length > 0 && (
              <ul className="mt-1 list-disc list-inside text-xs text-yellow-600">
                {lastResult.warnings.map((w, i) => (
                  <li key={i}>{w}</li>
                ))}
              </ul>
            )}
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={() => handleOpenDir(lastResult.outputPath)}
          >
            <FolderOpen className="h-4 w-4 mr-1" />
            {t('export.openFolder')}
          </Button>
          <Button variant="ghost" size="sm" onClick={() => clearResult()}>
            {t('common.dismiss')}
          </Button>
        </div>
      )}

      {/* Template list */}
      <div className="flex-1 overflow-auto p-4">
        {isLoading ? (
          <div className="flex items-center justify-center py-16">
            <Loader2 className="h-6 w-6 animate-spin mr-2 text-muted-foreground" />
            <span className="text-muted-foreground">{t('common.loading')}</span>
          </div>
        ) : templates.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-16 text-muted-foreground">
            <FileOutput className="h-12 w-12 mb-4 opacity-30" />
            <p className="text-lg font-medium">{t('export.noTemplates')}</p>
            <p className="text-sm mt-1">{t('export.noTemplatesHint')}</p>
            <Button variant="outline" className="mt-4" onClick={handleAdd}>
              <Plus className="h-4 w-4 mr-2" />
              {t('export.createFirst')}
            </Button>
          </div>
        ) : (
          <div className="grid gap-3">
            {templates.map(template => (
              <Card key={template.name}>
                <CardHeader className="pb-3">
                  <div className="flex items-start justify-between">
                    <div>
                      <CardTitle className="text-base flex items-center gap-2">
                        {template.name}
                        {template.description && (
                          <span className="text-sm font-normal text-muted-foreground">
                            — {template.description}
                          </span>
                        )}
                      </CardTitle>
                    </div>
                    <div className="flex items-center gap-1">
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => handleRun(template.name)}
                        disabled={runningTemplate === template.name}
                      >
                        {runningTemplate === template.name ? (
                          <Loader2 className="h-4 w-4 animate-spin mr-1" />
                        ) : (
                          <Play className="h-4 w-4 mr-1" />
                        )}
                        {t('common.run')}
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8"
                        onClick={() => handleEdit(template)}
                      >
                        <Pencil className="h-4 w-4" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8 text-destructive"
                        onClick={() => setDeleteConfirm(template.name)}
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </div>
                  </div>
                </CardHeader>
                <CardContent>
                  <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
                    <Badge variant="outline" className="text-xs">
                      {formatFormat(template.format)}
                    </Badge>
                    <Badge variant="outline" className="text-xs">
                      {template.outputStructure}
                    </Badge>
                    <span className="font-mono text-xs truncate max-w-[300px]">
                      → {template.outputPath}
                    </span>
                    {template.fetchModels && (
                      <Badge variant="secondary" className="text-xs">
                        {t('export.withModels')}
                      </Badge>
                    )}
                    {template.channels.enabledOnly && (
                      <Badge variant="secondary" className="text-xs">
                        {t('export.enabledOnly')}
                      </Badge>
                    )}
                    {template.fields &&
                      Object.keys(template.fields).length > 0 && (
                        <span className="text-xs">
                          {Object.keys(template.fields).length}{' '}
                          {t('export.fields')}
                        </span>
                      )}
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        )}
      </div>

      {/* Delete confirmation */}
      <AlertDialog
        open={deleteConfirm !== null}
        onOpenChange={open => !open && setDeleteConfirm(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('export.deleteTitle')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('export.deleteConfirm', { name: deleteConfirm })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction
              className="bg-destructive text-destructive-foreground"
              onClick={() => deleteConfirm && handleDelete(deleteConfirm)}
            >
              {t('common.delete')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Edit/Create dialog */}
      <ExportTemplateDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        template={editingTemplate}
      />
    </div>
  )
}
