import { useState, useEffect, useCallback, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import type { BundledTheme } from 'shiki'
import {
  RefreshCw,
  FileText,
  Clock,
  Pencil,
  Download,
  Trash2,
  Copy,
  Save,
  X,
  FolderOpen,
} from 'lucide-react'
import { Streamdown } from 'streamdown'
import { listen } from '@tauri-apps/api/event'
import { save } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from '@/components/ui/context-menu'
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
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { cn } from '@/lib/utils'
import { commands, type SpecFile } from '@/lib/bindings'
import { useUIStore } from '@/store/ui-store'
import { useTheme } from '@/hooks/use-theme'

export function SpecsPage() {
  const { t } = useTranslation()
  const { theme } = useTheme()
  const [specs, setSpecs] = useState<SpecFile[]>([])
  const [selectedSpec, setSelectedSpec] = useState<SpecFile | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const [systemPrefersDark, setSystemPrefersDark] = useState(
    () => window.matchMedia('(prefers-color-scheme: dark)').matches
  )

  useEffect(() => {
    if (theme !== 'system') return

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    const handleChange = (e: MediaQueryListEvent) => {
      setSystemPrefersDark(e.matches)
    }

    mediaQuery.addEventListener('change', handleChange)
    return () => mediaQuery.removeEventListener('change', handleChange)
  }, [theme])

  const isDark = theme === 'dark' || (theme === 'system' && systemPrefersDark)

  const shikiTheme: [BundledTheme, BundledTheme] = isDark
    ? ['github-light', 'github-dark']
    : ['github-light', 'github-light']

  // Rename dialog state
  const [renameDialogOpen, setRenameDialogOpen] = useState(false)
  const [renameSpec, setRenameSpec] = useState<SpecFile | null>(null)
  const [newName, setNewName] = useState('')

  // Delete dialog state
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false)
  const [deleteSpec, setDeleteSpec] = useState<SpecFile | null>(null)

  // Edit mode state
  const [isEditing, setIsEditing] = useState(false)
  const [editContent, setEditContent] = useState('')
  const [discardDialogOpen, setDiscardDialogOpen] = useState(false)
  const [pendingSpecChange, setPendingSpecChange] = useState<SpecFile | null>(
    null
  )

  // Inline rename state (for content header)
  const [isRenamingInline, setIsRenamingInline] = useState(false)
  const [inlineNewName, setInlineNewName] = useState('')

  // Ref for content scroll area
  const contentScrollRef = useRef<HTMLDivElement>(null)

  // Reset scroll position when selected spec changes
  useEffect(() => {
    if (selectedSpec && contentScrollRef.current) {
      const viewport = contentScrollRef.current.querySelector(
        '[data-slot="scroll-area-viewport"]'
      )
      if (viewport) {
        viewport.scrollTo(0, 0)
      }
    }
  }, [selectedSpec])

  const loadSpecs = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const result = await commands.listSpecs()
      if (result.status === 'ok') {
        setSpecs(result.data)
        // Auto-select first spec if none selected or selected was deleted
        setSelectedSpec(prev => {
          if (result.data.length === 0) return null
          if (!prev) return result.data[0] ?? null
          // Check if previously selected spec still exists
          const stillExists = result.data.find(s => s.path === prev.path)
          if (stillExists) {
            // Update content if it changed
            return stillExists
          }
          return result.data[0] ?? null
        })
      } else {
        setError(result.error)
      }
    } catch (err) {
      setError(String(err))
    } finally {
      setLoading(false)
    }
  }, [])

  // Start watcher and load specs on mount
  useEffect(() => {
    loadSpecs()

    // Start file watcher
    commands.startSpecsWatcher().catch(err => {
      console.error('Failed to start specs watcher:', err)
    })

    // Listen for specs-changed events
    const unlisten = listen('specs-changed', () => {
      loadSpecs()
    })

    return () => {
      // Stop watcher on unmount
      commands.stopSpecsWatcher().catch(err => {
        console.error('Failed to stop specs watcher:', err)
      })
      unlisten.then(fn => fn())
    }
  }, [loadSpecs])

  const handleRefresh = () => {
    loadSpecs()
  }

  const formatDate = (timestamp: number) => {
    const date = new Date(timestamp)
    return date.toLocaleDateString(undefined, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    })
  }

  // Rename handlers
  const handleRenameClick = (spec: SpecFile) => {
    setRenameSpec(spec)
    // Remove .md extension for editing
    setNewName(spec.name.replace(/\.md$/, ''))
    setRenameDialogOpen(true)
  }

  // Inline rename handlers (for content header)
  const handleInlineRenameStart = () => {
    if (selectedSpec) {
      setInlineNewName(selectedSpec.name.replace(/\.md$/, ''))
      setIsRenamingInline(true)
    }
  }

  const handleInlineRenameCancel = () => {
    setIsRenamingInline(false)
    setInlineNewName('')
  }

  const handleInlineRenameConfirm = async () => {
    if (!selectedSpec || !inlineNewName.trim()) return

    try {
      const result = await commands.renameSpec(
        selectedSpec.path,
        inlineNewName.trim()
      )
      if (result.status === 'ok') {
        toast.success(t('droid.specs.renameSuccess'))
        setSelectedSpec(result.data)
        loadSpecs()
      } else {
        toast.error(result.error)
      }
    } catch (err) {
      toast.error(String(err))
    } finally {
      setIsRenamingInline(false)
      setInlineNewName('')
    }
  }

  const handleRenameConfirm = async () => {
    if (!renameSpec || !newName.trim()) return

    try {
      const result = await commands.renameSpec(renameSpec.path, newName.trim())
      if (result.status === 'ok') {
        toast.success(t('droid.specs.renameSuccess'))
        // Update selected spec if it was renamed
        if (selectedSpec?.path === renameSpec.path) {
          setSelectedSpec(result.data)
        }
        loadSpecs()
      } else {
        toast.error(result.error)
      }
    } catch (err) {
      toast.error(String(err))
    } finally {
      setRenameDialogOpen(false)
      setRenameSpec(null)
      setNewName('')
    }
  }

  // Delete handlers
  const handleDeleteClick = (spec: SpecFile) => {
    setDeleteSpec(spec)
    setDeleteDialogOpen(true)
  }

  const handleDeleteConfirm = async () => {
    if (!deleteSpec) return

    try {
      const result = await commands.deleteSpec(deleteSpec.path)
      if (result.status === 'ok') {
        toast.success(t('droid.specs.deleteSuccess'))
        // Clear selection if deleted spec was selected
        if (selectedSpec?.path === deleteSpec.path) {
          setSelectedSpec(null)
        }
        loadSpecs()
      } else {
        toast.error(result.error)
      }
    } catch (err) {
      toast.error(String(err))
    } finally {
      setDeleteDialogOpen(false)
      setDeleteSpec(null)
    }
  }

  // Export handler
  const handleExport = async (spec: SpecFile) => {
    try {
      const lastPath = useUIStore.getState().lastSpecExportPath
      const defaultPath = lastPath ? `${lastPath}/${spec.name}` : spec.name

      const filePath = await save({
        filters: [{ name: 'Markdown', extensions: ['md'] }],
        defaultPath,
      })

      if (!filePath) return

      await writeTextFile(filePath, spec.content)

      // Save directory path for next time
      const lastSlash = filePath.lastIndexOf('/')
      if (lastSlash > 0) {
        const dir = filePath.substring(0, lastSlash)
        useUIStore.getState().setLastSpecExportPath(dir)
      }

      toast.success(t('droid.specs.exportSuccess'))
    } catch (err) {
      toast.error(String(err))
    }
  }

  // Copy handler
  const handleCopy = async (
    content: string,
    successKey = 'droid.specs.copySuccess'
  ) => {
    try {
      await writeText(content)
      toast.success(t(successKey))
    } catch (err) {
      toast.error(String(err))
    }
  }

  // Edit mode handlers
  const hasUnsavedChanges =
    isEditing && selectedSpec && editContent !== selectedSpec.content

  const handleEditClick = () => {
    if (selectedSpec) {
      setEditContent(selectedSpec.content)
      setIsEditing(true)
    }
  }

  const handleCancelEdit = () => {
    if (hasUnsavedChanges) {
      setDiscardDialogOpen(true)
    } else {
      setIsEditing(false)
      setEditContent('')
    }
  }

  const handleDiscardConfirm = () => {
    setIsEditing(false)
    setEditContent('')
    setDiscardDialogOpen(false)
    // If there's a pending spec change, apply it
    if (pendingSpecChange) {
      setSelectedSpec(pendingSpecChange)
      setPendingSpecChange(null)
    }
  }

  const handleSaveEdit = useCallback(async () => {
    if (!selectedSpec) return

    try {
      const result = await commands.updateSpec(selectedSpec.path, editContent)
      if (result.status === 'ok') {
        toast.success(t('droid.specs.saveSuccess'))
        setSelectedSpec(result.data)
        setIsEditing(false)
        setEditContent('')
        loadSpecs()
      } else {
        toast.error(result.error)
      }
    } catch (err) {
      toast.error(String(err))
    }
  }, [selectedSpec, editContent, loadSpecs, t])

  // Handle spec selection with unsaved changes check
  const handleSpecSelect = (spec: SpecFile) => {
    if (hasUnsavedChanges) {
      setPendingSpecChange(spec)
      setDiscardDialogOpen(true)
    } else {
      setSelectedSpec(spec)
      setIsEditing(false)
      setEditContent('')
    }
  }

  // Keyboard shortcut for save (Ctrl+S / Cmd+S)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 's' && isEditing) {
        e.preventDefault()
        handleSaveEdit()
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [isEditing, handleSaveEdit])

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between p-4 border-b">
        <h1 className="text-xl font-semibold">{t('droid.specs.title')}</h1>
        <Button
          variant="outline"
          size="sm"
          onClick={handleRefresh}
          disabled={loading}
        >
          <RefreshCw
            className={cn('h-4 w-4 mr-2', loading && 'animate-spin')}
          />
          {t('common.refresh')}
        </Button>
      </div>

      <div className="flex flex-1 overflow-hidden">
        {/* Specs List */}
        <div className="w-64 border-r flex flex-col">
          <ScrollArea className="flex-1">
            <div className="p-2 space-y-1">
              {loading && specs.length === 0 ? (
                <div className="flex items-center justify-center p-4 text-muted-foreground">
                  {t('common.loading')}
                </div>
              ) : error ? (
                <div className="p-4 text-destructive text-sm">{error}</div>
              ) : specs.length === 0 ? (
                <div className="flex flex-col items-center justify-center p-4 text-muted-foreground text-sm">
                  <FileText className="h-8 w-8 mb-2 opacity-50" />
                  <p>{t('droid.specs.noSpecs')}</p>
                  <p className="text-xs mt-1">{t('droid.specs.noSpecsHint')}</p>
                </div>
              ) : (
                specs.map(spec => (
                  <ContextMenu key={spec.path}>
                    <ContextMenuTrigger asChild>
                      <button
                        onClick={() => handleSpecSelect(spec)}
                        className={cn(
                          'w-full text-start p-2 rounded-md hover:bg-accent transition-colors',
                          selectedSpec?.path === spec.path && 'bg-accent'
                        )}
                      >
                        <div className="font-medium text-sm truncate">
                          {spec.name}
                        </div>
                        <div className="flex items-center gap-1 text-xs text-muted-foreground mt-1">
                          <Clock className="h-3 w-3" />
                          {formatDate(spec.modifiedAt)}
                        </div>
                      </button>
                    </ContextMenuTrigger>
                    <ContextMenuContent>
                      <ContextMenuItem onClick={() => handleRenameClick(spec)}>
                        <Pencil className="h-4 w-4 mr-2" />
                        {t('common.rename')}
                      </ContextMenuItem>
                      <ContextMenuItem onClick={() => handleExport(spec)}>
                        <Download className="h-4 w-4 mr-2" />
                        {t('common.export')}
                      </ContextMenuItem>
                      <ContextMenuItem
                        onClick={() => handleDeleteClick(spec)}
                        className="text-destructive focus:text-destructive"
                      >
                        <Trash2 className="h-4 w-4 mr-2" />
                        {t('common.delete')}
                      </ContextMenuItem>
                    </ContextMenuContent>
                  </ContextMenu>
                ))
              )}
            </div>
          </ScrollArea>
        </div>

        {/* Spec Content */}
        <div className="flex-1 flex flex-col min-w-0">
          {selectedSpec ? (
            <>
              <div className="p-4 border-b flex items-center justify-between">
                <div>
                  {isRenamingInline ? (
                    <div className="flex items-center gap-2">
                      <Input
                        value={inlineNewName}
                        onChange={e => setInlineNewName(e.target.value)}
                        className="h-7 text-sm font-medium"
                        autoFocus
                        onKeyDown={e => {
                          if (e.key === 'Enter') {
                            handleInlineRenameConfirm()
                          } else if (e.key === 'Escape') {
                            handleInlineRenameCancel()
                          }
                        }}
                        onBlur={handleInlineRenameCancel}
                      />
                      <span className="text-sm text-muted-foreground">.md</span>
                    </div>
                  ) : (
                    <h2
                      className="font-medium cursor-pointer hover:text-primary transition-colors"
                      onClick={handleInlineRenameStart}
                      title={t('common.rename')}
                    >
                      {selectedSpec.name}
                    </h2>
                  )}
                  <p className="text-xs text-muted-foreground mt-1">
                    {formatDate(selectedSpec.modifiedAt)}
                  </p>
                </div>
                <div className="flex items-center gap-2">
                  {isEditing ? (
                    <>
                      <Button
                        variant="outline"
                        size="sm"
                        onMouseDown={e => e.preventDefault()}
                        onClick={handleCancelEdit}
                      >
                        <X className="h-4 w-4 mr-1" />
                        {t('common.cancel')}
                      </Button>
                      <Button
                        size="sm"
                        onMouseDown={e => e.preventDefault()}
                        onClick={handleSaveEdit}
                        disabled={!hasUnsavedChanges}
                      >
                        <Save className="h-4 w-4 mr-1" />
                        {t('common.save')}
                      </Button>
                    </>
                  ) : (
                    <>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Button
                            variant="outline"
                            size="icon"
                            className="h-8 w-8"
                            onClick={() =>
                              handleCopy(
                                selectedSpec.path,
                                'droid.specs.copyPathSuccess'
                              )
                            }
                          >
                            <FolderOpen className="h-4 w-4" />
                          </Button>
                        </TooltipTrigger>
                        <TooltipContent>
                          {t('droid.specs.copyPath')}
                        </TooltipContent>
                      </Tooltip>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Button
                            variant="outline"
                            size="icon"
                            className="h-8 w-8"
                            onClick={() => handleCopy(selectedSpec.content)}
                          >
                            <Copy className="h-4 w-4" />
                          </Button>
                        </TooltipTrigger>
                        <TooltipContent>
                          {t('droid.specs.copyAll')}
                        </TooltipContent>
                      </Tooltip>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <Button
                            variant="outline"
                            size="icon"
                            className="h-8 w-8"
                            onClick={handleEditClick}
                          >
                            <Pencil className="h-4 w-4" />
                          </Button>
                        </TooltipTrigger>
                        <TooltipContent>
                          {t('droid.specs.editMode')}
                        </TooltipContent>
                      </Tooltip>
                    </>
                  )}
                </div>
              </div>
              {isEditing ? (
                <Textarea
                  value={editContent}
                  onChange={e => setEditContent(e.target.value)}
                  className="flex-1 resize-none rounded-none border-0 font-mono text-sm p-4"
                  placeholder="Enter spec content..."
                />
              ) : (
                <ScrollArea ref={contentScrollRef} className="flex-1 min-w-0">
                  <div className="p-4 px-6 select-text">
                    <Streamdown shikiTheme={shikiTheme}>
                      {selectedSpec.content}
                    </Streamdown>
                  </div>
                </ScrollArea>
              )}
            </>
          ) : (
            <div className="flex items-center justify-center h-full text-muted-foreground">
              <p>{t('droid.specs.selectSpecHint')}</p>
            </div>
          )}
        </div>
      </div>

      {/* Rename Dialog */}
      <Dialog open={renameDialogOpen} onOpenChange={setRenameDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('droid.specs.renameTitle')}</DialogTitle>
            <DialogDescription>
              {t('droid.specs.renameDescription')}
            </DialogDescription>
          </DialogHeader>
          <Input
            value={newName}
            onChange={e => setNewName(e.target.value)}
            placeholder={t('droid.specs.fileNamePlaceholder')}
            onKeyDown={e => {
              if (e.key === 'Enter') {
                handleRenameConfirm()
              }
            }}
          />
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setRenameDialogOpen(false)}
            >
              {t('common.cancel')}
            </Button>
            <Button onClick={handleRenameConfirm} disabled={!newName.trim()}>
              {t('common.rename')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Delete Confirmation Dialog */}
      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('droid.specs.deleteTitle')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('droid.specs.deleteDescription', { name: deleteSpec?.name })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction onClick={handleDeleteConfirm}>
              {t('common.delete')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Discard Changes Confirmation Dialog */}
      <AlertDialog open={discardDialogOpen} onOpenChange={setDiscardDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {t('droid.specs.discardChanges')}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {t('droid.specs.unsavedChanges')}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction onClick={handleDiscardConfirm}>
              {t('droid.specs.discardChanges')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
