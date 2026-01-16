import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Plus, Pencil, Trash2, GripVertical } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Textarea } from '@/components/ui/textarea'
import { useTerminalStore, type TerminalSnippet } from '@/store/terminal-store'

interface TerminalSnippetDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

type EditMode = 'list' | 'add' | 'edit'

export function TerminalSnippetDialog({
  open,
  onOpenChange,
}: TerminalSnippetDialogProps) {
  const { t } = useTranslation()
  const snippets = useTerminalStore(state => state.snippets)
  const addSnippet = useTerminalStore(state => state.addSnippet)
  const updateSnippet = useTerminalStore(state => state.updateSnippet)
  const removeSnippet = useTerminalStore(state => state.removeSnippet)
  const reorderSnippets = useTerminalStore(state => state.reorderSnippets)

  const [mode, setMode] = useState<EditMode>('list')
  const [editingSnippet, setEditingSnippet] = useState<TerminalSnippet | null>(
    null
  )
  const [name, setName] = useState('')
  const [content, setContent] = useState('')
  const [autoExecute, setAutoExecute] = useState(false)
  const [draggedId, setDraggedId] = useState<string | null>(null)

  const resetForm = () => {
    setName('')
    setContent('')
    setAutoExecute(false)
    setEditingSnippet(null)
  }

  const handleAdd = () => {
    resetForm()
    setMode('add')
  }

  const handleEdit = (snippet: TerminalSnippet) => {
    setEditingSnippet(snippet)
    setName(snippet.name)
    setContent(snippet.content)
    setAutoExecute(snippet.autoExecute)
    setMode('edit')
  }

  const handleSave = () => {
    if (!name.trim() || !content.trim()) return

    if (mode === 'add') {
      addSnippet({
        name: name.trim(),
        content: content.trim(),
        autoExecute,
      })
    } else if (mode === 'edit' && editingSnippet) {
      updateSnippet(editingSnippet.id, {
        name: name.trim(),
        content: content.trim(),
        autoExecute,
      })
    }

    resetForm()
    setMode('list')
  }

  const handleDelete = (id: string) => {
    removeSnippet(id)
  }

  const handleCancel = () => {
    resetForm()
    setMode('list')
  }

  const handleDragStart = (id: string) => {
    setDraggedId(id)
  }

  const handleDragOver = (e: React.DragEvent, targetId: string) => {
    e.preventDefault()
    if (!draggedId || draggedId === targetId) return

    const currentIds = snippets.map(s => s.id)
    const draggedIndex = currentIds.indexOf(draggedId)
    const targetIndex = currentIds.indexOf(targetId)

    if (draggedIndex === -1 || targetIndex === -1) return

    const newIds = [...currentIds]
    newIds.splice(draggedIndex, 1)
    newIds.splice(targetIndex, 0, draggedId)
    reorderSnippets(newIds)
  }

  const handleDragEnd = () => {
    setDraggedId(null)
  }

  const handleOpenChange = (newOpen: boolean) => {
    if (!newOpen) {
      resetForm()
      setMode('list')
    }
    onOpenChange(newOpen)
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>
            {mode === 'list' && t('droid.terminal.snippets.manage')}
            {mode === 'add' && t('droid.terminal.snippets.add')}
            {mode === 'edit' && t('droid.terminal.snippets.edit')}
          </DialogTitle>
        </DialogHeader>

        {mode === 'list' ? (
          <div className="flex flex-col gap-2">
            {snippets.length === 0 ? (
              <div className="text-center text-muted-foreground py-8">
                {t('droid.terminal.snippets.empty')}
              </div>
            ) : (
              <div className="max-h-64 overflow-y-auto">
                {snippets.map(snippet => (
                  <div
                    key={snippet.id}
                    draggable
                    onDragStart={() => handleDragStart(snippet.id)}
                    onDragOver={e => handleDragOver(e, snippet.id)}
                    onDragEnd={handleDragEnd}
                    className="flex items-center gap-2 p-2 rounded-md hover:bg-accent group"
                  >
                    <GripVertical className="h-4 w-4 text-muted-foreground cursor-grab" />
                    <div className="flex-1 min-w-0">
                      <div className="font-medium text-sm truncate">
                        {snippet.name}
                      </div>
                      <div className="text-xs text-muted-foreground truncate">
                        {snippet.content}
                      </div>
                    </div>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-8 w-8 opacity-0 group-hover:opacity-100"
                      onMouseDown={e => {
                        e.preventDefault()
                        handleEdit(snippet)
                      }}
                    >
                      <Pencil className="h-4 w-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-8 w-8 opacity-0 group-hover:opacity-100 text-destructive"
                      onMouseDown={e => {
                        e.preventDefault()
                        handleDelete(snippet.id)
                      }}
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </div>
                ))}
              </div>
            )}
            <DialogFooter className="mt-4">
              <Button
                onMouseDown={e => {
                  e.preventDefault()
                  handleAdd()
                }}
              >
                <Plus className="h-4 w-4 mr-2" />
                {t('droid.terminal.snippets.add')}
              </Button>
            </DialogFooter>
          </div>
        ) : (
          <div className="flex flex-col gap-4">
            <div className="flex flex-col gap-2">
              <Label htmlFor="snippet-name">
                {t('droid.terminal.snippets.name')}
              </Label>
              <Input
                id="snippet-name"
                value={name}
                onChange={e => setName(e.target.value)}
                placeholder={t('droid.terminal.snippets.namePlaceholder')}
              />
            </div>
            <div className="flex flex-col gap-2">
              <Label htmlFor="snippet-content">
                {t('droid.terminal.snippets.content')}
              </Label>
              <Textarea
                id="snippet-content"
                value={content}
                onChange={e => setContent(e.target.value)}
                placeholder={t('droid.terminal.snippets.contentPlaceholder')}
                rows={3}
              />
            </div>
            <div className="flex items-center justify-between">
              <Label htmlFor="auto-execute">
                {t('droid.terminal.snippets.autoExecute')}
              </Label>
              <Switch
                id="auto-execute"
                checked={autoExecute}
                onCheckedChange={setAutoExecute}
              />
            </div>
            <DialogFooter>
              <Button
                variant="outline"
                onMouseDown={e => {
                  e.preventDefault()
                  handleCancel()
                }}
              >
                {t('common.cancel')}
              </Button>
              <Button
                onMouseDown={e => {
                  e.preventDefault()
                  handleSave()
                }}
                disabled={!name.trim() || !content.trim()}
              >
                {t('common.save')}
              </Button>
            </DialogFooter>
          </div>
        )}
      </DialogContent>
    </Dialog>
  )
}
