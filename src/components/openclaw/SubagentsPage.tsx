import { useState, useEffect, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { Plus, RefreshCw, AlertCircle, Bot } from 'lucide-react'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
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
import { useSubagentsStore } from '@/store/subagents-store'
import { SubagentCard } from './SubagentCard'
import { SubagentDialog } from './SubagentDialog'
import type { OpenClawSubAgent } from '@/lib/bindings'

export function SubagentsPage() {
  const { t } = useTranslation()
  const subagents = useSubagentsStore(state => state.subagents)
  const isLoading = useSubagentsStore(state => state.isLoading)
  const error = useSubagentsStore(state => state.error)
  const loadSubagents = useSubagentsStore(state => state.loadSubagents)
  const addSubagent = useSubagentsStore(state => state.addSubagent)
  const updateSubagent = useSubagentsStore(state => state.updateSubagent)
  const deleteSubagent = useSubagentsStore(state => state.deleteSubagent)
  const toggleSubagentAllow = useSubagentsStore(
    state => state.toggleSubagentAllow
  )
  const setError = useSubagentsStore(state => state.setError)

  // Filter out main agent and derive allowed IDs
  const nonMainSubagents = useMemo(
    () => subagents.filter(a => a.id !== 'main'),
    [subagents]
  )
  const allowedIds = useMemo(() => {
    const main = subagents.find(a => a.id === 'main')
    return new Set(main?.subagents?.allowAgents ?? [])
  }, [subagents])

  const [dialogOpen, setDialogOpen] = useState(false)
  const [editingAgent, setEditingAgent] = useState<OpenClawSubAgent | null>(
    null
  )
  const [deleteAgentId, setDeleteAgentId] = useState<string | null>(null)

  useEffect(() => {
    loadSubagents()
  }, [loadSubagents])

  const handleAdd = () => {
    setEditingAgent(null)
    setDialogOpen(true)
  }

  const handleEdit = (agent: OpenClawSubAgent) => {
    setEditingAgent(agent)
    setDialogOpen(true)
  }

  const handleSave = async (agent: OpenClawSubAgent) => {
    if (editingAgent) {
      await updateSubagent(editingAgent.id, agent)
    } else {
      await addSubagent(agent)
    }
    toast.success(t('common.saved'))
  }

  const handleConfirmDelete = async () => {
    if (deleteAgentId) {
      await deleteSubagent(deleteAgentId)
      setDeleteAgentId(null)
      toast.success(t('common.deleted'))
    }
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between gap-2 p-4 border-b">
        <div className="min-w-0 flex-1">
          <h1 className="text-xl font-semibold">
            {t('openclaw.subagents.title')}
          </h1>
          <p className="text-sm text-muted-foreground mt-1">
            {t('openclaw.subagents.description')}
          </p>
        </div>
        <div className="flex items-center gap-2 shrink-0">
          <Button
            variant="outline"
            size="icon"
            onClick={() => loadSubagents()}
            disabled={isLoading}
            title={t('common.refresh')}
          >
            <RefreshCw className="h-4 w-4" />
          </Button>
          <Button variant="outline" size="sm" onClick={handleAdd}>
            <Plus className="h-4 w-4 mr-2" />
            {t('openclaw.subagents.add')}
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

      {/* Content */}
      <div className="flex-1 overflow-auto p-4">
        {nonMainSubagents.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full gap-3 text-muted-foreground">
            <Bot className="h-12 w-12" />
            <p>{t('openclaw.subagents.empty')}</p>
            <Button variant="outline" size="sm" onClick={handleAdd}>
              <Plus className="h-4 w-4 mr-2" />
              {t('openclaw.subagents.add')}
            </Button>
          </div>
        ) : (
          <div className="space-y-2">
            {nonMainSubagents.map(agent => (
              <SubagentCard
                key={agent.id}
                agent={agent}
                isAllowed={allowedIds.has(agent.id)}
                onToggleAllow={() => toggleSubagentAllow(agent.id)}
                onEdit={() => handleEdit(agent)}
                onDelete={() => setDeleteAgentId(agent.id)}
              />
            ))}
          </div>
        )}
      </div>

      {/* Dialog */}
      <SubagentDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        editingAgent={editingAgent}
        existingIds={nonMainSubagents.map(a => a.id)}
        onSave={handleSave}
      />

      {/* Delete Confirmation */}
      <AlertDialog
        open={deleteAgentId !== null}
        onOpenChange={() => setDeleteAgentId(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {t('openclaw.subagents.delete')}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {t('openclaw.subagents.deleteConfirm')}
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
    </div>
  )
}
