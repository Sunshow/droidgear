import { useState, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { X } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { useOpenClawStore } from '@/store/openclaw-store'
import type { OpenClawSubAgent } from '@/lib/bindings'

interface SubagentDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  editingAgent: OpenClawSubAgent | null
  existingIds: string[]
  onSave: (agent: OpenClawSubAgent) => void
}

interface FormState {
  agentId: string
  agentName: string
  emoji: string
  primaryModel: string
  fallbacks: string[]
  toolsProfile: string
  workspace: string
  allowAgents: string[]
  maxConcurrent: string
}

function buildFormState(agent: OpenClawSubAgent | null): FormState {
  if (agent) {
    return {
      agentId: agent.id,
      agentName: agent.name ?? agent.identity?.name ?? '',
      emoji: agent.identity?.emoji ?? '',
      primaryModel: agent.model?.primary ?? '',
      fallbacks: agent.model?.fallbacks ?? [],
      toolsProfile: agent.tools?.profile ?? '',
      workspace: agent.workspace ?? '',
      allowAgents: agent.subagents?.allowAgents ?? [],
      maxConcurrent: agent.subagents?.maxConcurrent?.toString() ?? '',
    }
  }
  return {
    agentId: '',
    agentName: '',
    emoji: '💻',
    primaryModel: '',
    fallbacks: [],
    toolsProfile: 'full',
    workspace: '',
    allowAgents: [],
    maxConcurrent: '',
  }
}

const TOOLS_PROFILES = ['full', 'read', 'none']

export function SubagentDialog({
  open,
  onOpenChange,
  editingAgent,
  existingIds,
  onSave,
}: SubagentDialogProps) {
  const { t } = useTranslation()
  const isEditing = editingAgent !== null

  const [form, setForm] = useState<FormState>(() =>
    buildFormState(editingAgent)
  )

  // Draft values for the "add chip" inputs (fallbacks / allowAgents)
  const [fallbackDraft, setFallbackDraft] = useState('')
  const [allowAgentsDraft, setAllowAgentsDraft] = useState('')

  // Update a single form field
  const setField = <K extends keyof FormState>(key: K, value: FormState[K]) => {
    setForm(prev => ({ ...prev, [key]: value }))
  }

  // Get available model refs from the openclaw store
  const currentProfile = useOpenClawStore(state => state.currentProfile)
  const availableModelRefs = useMemo(() => {
    if (!currentProfile) return []
    return Object.entries(currentProfile.providers ?? {}).flatMap(
      ([providerId, config]) =>
        (config?.models ?? []).map(m => `${providerId}/${m.id}`)
    )
  }, [currentProfile])

  // Reset form when dialog opens — use onOpenChange wrapper to avoid
  // cascading setState inside useEffect
  const handleOpenChange = (nextOpen: boolean) => {
    if (nextOpen) {
      setForm(buildFormState(editingAgent))
      setFallbackDraft('')
      setAllowAgentsDraft('')
    }
    onOpenChange(nextOpen)
  }

  const handleSave = () => {
    if (!form.agentId.trim()) return

    const agent: OpenClawSubAgent = {
      id: form.agentId.trim(),
      name: form.agentName.trim() || null,
      identity:
        form.emoji.trim() || form.agentName.trim()
          ? {
              emoji: form.emoji.trim() || null,
              name: form.agentName.trim() || null,
            }
          : null,
      model: form.primaryModel.trim()
        ? {
            primary: form.primaryModel.trim(),
            fallbacks: form.fallbacks.length > 0 ? form.fallbacks : null,
          }
        : null,
      tools: form.toolsProfile ? { profile: form.toolsProfile } : null,
      workspace: form.workspace.trim() || null,
      subagents:
        form.allowAgents.length > 0 || form.maxConcurrent
          ? {
              allowAgents:
                form.allowAgents.length > 0 ? form.allowAgents : null,
              maxConcurrent: form.maxConcurrent
                ? parseInt(form.maxConcurrent)
                : null,
            }
          : null,
    }

    onSave(agent)
    onOpenChange(false)
  }

  const idError =
    !isEditing &&
    form.agentId.trim() &&
    existingIds.includes(form.agentId.trim())

  const isMainAgent = form.agentId === 'main'

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="max-w-lg max-h-[85vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>
            {isEditing
              ? t('openclaw.subagents.edit')
              : t('openclaw.subagents.add')}
          </DialogTitle>
          <DialogDescription>
            {t('openclaw.subagents.dialogDescription')}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-2">
          {/* ID */}
          <div className="space-y-2">
            <Label>{t('openclaw.subagents.id')}</Label>
            <Input
              value={form.agentId}
              onChange={e => setField('agentId', e.target.value)}
              placeholder="my-agent"
              disabled={isEditing}
            />
            {idError && (
              <p className="text-xs text-destructive">
                {t('openclaw.subagents.idExists')}
              </p>
            )}
          </div>

          {/* Name */}
          <div className="space-y-2">
            <Label>{t('openclaw.subagents.name')}</Label>
            <Input
              value={form.agentName}
              onChange={e => setField('agentName', e.target.value)}
              placeholder={t('openclaw.subagents.namePlaceholder')}
            />
          </div>

          {/* Emoji */}
          <div className="space-y-2">
            <Label>{t('openclaw.subagents.emoji')}</Label>
            <Input
              value={form.emoji}
              onChange={e => setField('emoji', e.target.value)}
              placeholder="💻"
              className="w-24"
            />
          </div>

          {/* Primary Model */}
          <div className="space-y-2">
            <Label>{t('openclaw.subagents.primaryModel')}</Label>
            <div className="flex gap-2">
              <Input
                value={form.primaryModel}
                onChange={e => setField('primaryModel', e.target.value)}
                placeholder="provider/model-id"
                className="flex-1"
              />
              {availableModelRefs.length > 0 && (
                <Select
                  value=""
                  onValueChange={val => setField('primaryModel', val)}
                >
                  <SelectTrigger className="w-12 shrink-0">
                    <SelectValue placeholder="..." />
                  </SelectTrigger>
                  <SelectContent>
                    {availableModelRefs.map(modelRef => (
                      <SelectItem key={modelRef} value={modelRef}>
                        {modelRef}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              )}
            </div>
          </div>

          {/* Fallback Models */}
          <div className="space-y-2">
            <Label>{t('openclaw.subagents.fallbacks')}</Label>
            <div className="flex gap-2">
              <Input
                value={fallbackDraft}
                onChange={e => setFallbackDraft(e.target.value)}
                placeholder={t('openclaw.subagents.fallbackPlaceholder')}
                onKeyDown={e => {
                  if (e.key === 'Enter') {
                    const val = fallbackDraft.trim()
                    if (val && !form.fallbacks.includes(val)) {
                      setField('fallbacks', [...form.fallbacks, val])
                      setFallbackDraft('')
                    }
                  }
                }}
                className="flex-1"
              />
              {availableModelRefs.length > 0 && (
                <Select
                  value=""
                  onValueChange={val => {
                    if (!form.fallbacks.includes(val)) {
                      setField('fallbacks', [...form.fallbacks, val])
                    }
                  }}
                >
                  <SelectTrigger className="w-12 shrink-0">
                    <SelectValue placeholder="..." />
                  </SelectTrigger>
                  <SelectContent>
                    {availableModelRefs
                      .filter(r => !form.fallbacks.includes(r))
                      .map(modelRef => (
                        <SelectItem key={modelRef} value={modelRef}>
                          {modelRef}
                        </SelectItem>
                      ))}
                  </SelectContent>
                </Select>
              )}
            </div>
            {form.fallbacks.length > 0 && (
              <div className="space-y-1">
                {form.fallbacks.map((modelRef, idx) => (
                  <div
                    key={modelRef}
                    className="flex items-center gap-2 px-3 py-1.5 border rounded-md bg-muted/30"
                  >
                    <span className="text-xs text-muted-foreground w-4 shrink-0 text-center">
                      {idx + 1}
                    </span>
                    <span className="flex-1 text-sm font-mono truncate">
                      {modelRef}
                    </span>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-5 w-5 shrink-0"
                      onClick={() =>
                        setField(
                          'fallbacks',
                          form.fallbacks.filter(r => r !== modelRef)
                        )
                      }
                    >
                      <X className="h-3 w-3" />
                    </Button>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Tools Profile */}
          <div className="space-y-2">
            <Label>{t('openclaw.subagents.toolsProfile')}</Label>
            <Select
              value={form.toolsProfile}
              onValueChange={val => setField('toolsProfile', val)}
            >
              <SelectTrigger>
                <SelectValue
                  placeholder={t('openclaw.subagents.toolsProfilePlaceholder')}
                />
              </SelectTrigger>
              <SelectContent>
                {TOOLS_PROFILES.map(p => (
                  <SelectItem key={p} value={p}>
                    {p}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Workspace */}
          <div className="space-y-2">
            <Label>{t('openclaw.subagents.workspace')}</Label>
            <Input
              value={form.workspace}
              onChange={e => setField('workspace', e.target.value)}
              placeholder={t('openclaw.subagents.workspacePlaceholder')}
            />
          </div>

          {/* Allow Agents (only for main agent) */}
          {isMainAgent && (
            <div className="space-y-2">
              <Label>{t('openclaw.subagents.allowAgents')}</Label>
              <Input
                value={allowAgentsDraft}
                onChange={e => setAllowAgentsDraft(e.target.value)}
                placeholder={t('openclaw.subagents.allowAgentsPlaceholder')}
                onKeyDown={e => {
                  if (e.key === 'Enter') {
                    const val = allowAgentsDraft.trim()
                    if (val && !form.allowAgents.includes(val)) {
                      setField('allowAgents', [...form.allowAgents, val])
                      setAllowAgentsDraft('')
                    }
                  }
                }}
              />
              {form.allowAgents.length > 0 && (
                <div className="flex flex-wrap gap-1">
                  {form.allowAgents.map(id => (
                    <span
                      key={id}
                      className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-muted text-xs"
                    >
                      {id}
                      <button
                        className="hover:text-destructive"
                        onClick={() =>
                          setField(
                            'allowAgents',
                            form.allowAgents.filter(a => a !== id)
                          )
                        }
                      >
                        <X className="h-3 w-3" />
                      </button>
                    </span>
                  ))}
                </div>
              )}
            </div>
          )}

          {/* Max Concurrent (only for main agent) */}
          {isMainAgent && (
            <div className="space-y-2">
              <Label>{t('openclaw.subagents.maxConcurrent')}</Label>
              <Input
                type="number"
                value={form.maxConcurrent}
                onChange={e => setField('maxConcurrent', e.target.value)}
                placeholder="8"
                className="w-32"
              />
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t('common.cancel')}
          </Button>
          <Button
            onClick={handleSave}
            disabled={!form.agentId.trim() || !!idError}
          >
            {t('common.save')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
