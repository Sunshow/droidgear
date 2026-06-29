import { useState, useEffect, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import {
  KeyRound,
  Plus,
  Trash2,
  Pencil,
  Check,
  ShieldAlert,
} from 'lucide-react'
import { toast } from 'sonner'
import { cn } from '@/lib/utils'
import { ActionButton } from '@/components/ui/action-button'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { commands } from '@/lib/bindings'
import type { CodexAuthProfile, CodexAuthProfileState } from '@/lib/bindings'

export function CodexAuthPage() {
  const { t } = useTranslation()
  const [state, setState] = useState<CodexAuthProfileState | null>(null)
  const [loading, setLoading] = useState(true)
  const [saveDialogOpen, setSaveDialogOpen] = useState(false)
  const [renameDialogOpen, setRenameDialogOpen] = useState(false)
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false)
  const [conflictDialogOpen, setConflictDialogOpen] = useState(false)
  const [conflictProfileName, setConflictProfileName] = useState<string | null>(
    null
  )
  const [selectedProfile, setSelectedProfile] =
    useState<CodexAuthProfile | null>(null)
  const [newName, setNewName] = useState('')
  const [newLabel, setNewLabel] = useState('')

  const loadProfiles = useCallback(async () => {
    const result = await commands.listCodexAuthProfiles()
    if (result.status === 'ok') {
      setState(result.data)
    } else {
      toast.error(t('codexAuth.loadError'))
    }
    setLoading(false)
  }, [t])

  useEffect(() => {
    let cancelled = false
    const init = async () => {
      const result = await commands.listCodexAuthProfiles()
      if (cancelled) return
      if (result.status === 'ok') {
        setState(result.data)
      }
      setLoading(false)
    }
    init()
    return () => {
      cancelled = true
    }
  }, [])

  const handleSave = async () => {
    if (!newName.trim()) return
    const result = await commands.saveCurrentCodexAuthProfile(
      newName.trim(),
      newLabel.trim() || newName.trim()
    )
    if (result.status === 'ok') {
      toast.success(t('codexAuth.saveSuccess', { name: newName }))
      setSaveDialogOpen(false)
      setNewName('')
      setNewLabel('')
      await loadProfiles()
    } else {
      toast.error(result.error)
    }
  }

  const handleSwitch = async (name: string) => {
    // Check for auth mode conflict first
    const conflictResult = await commands.detectCodexAuthConflict(name)
    if (conflictResult.status === 'ok' && conflictResult.data.hasConflict) {
      setConflictProfileName(name)
      setConflictDialogOpen(true)
      return
    }

    await doSwitch(name)
  }

  const handleConflictSaveAndSwitch = async () => {
    if (!conflictProfileName) return

    const now = new Date()
    const timestamp = now.getTime()
    // First save current auth
    const saveResult = await commands.saveCurrentCodexAuthProfile(
      `auto-backup-${timestamp}`,
      `Auto backup ${now.toLocaleString()}`
    )
    if (saveResult.status === 'ok') {
      toast.success(t('codexAuth.autoBackupSuccess'))
    }

    // Then switch
    await doSwitch(conflictProfileName)
    setConflictDialogOpen(false)
    setConflictProfileName(null)
  }

  const handleConflictSwitchWithoutSave = async () => {
    if (!conflictProfileName) return
    await doSwitch(conflictProfileName)
    setConflictDialogOpen(false)
    setConflictProfileName(null)
  }

  const doSwitch = async (name: string) => {
    const result = await commands.switchCodexAuthProfile(name)
    if (result.status === 'ok') {
      toast.success(t('codexAuth.switchSuccess', { name }))
      await loadProfiles()
    } else {
      toast.error(result.error)
    }
  }

  const handleRename = async () => {
    if (!selectedProfile || !newLabel.trim()) return
    const result = await commands.renameCodexAuthProfile(
      selectedProfile.name,
      newLabel.trim()
    )
    if (result.status === 'ok') {
      toast.success(t('codexAuth.renameSuccess'))
      setRenameDialogOpen(false)
      setSelectedProfile(null)
      setNewLabel('')
      await loadProfiles()
    } else {
      toast.error(result.error)
    }
  }

  const handleDelete = async () => {
    if (!selectedProfile) return
    const result = await commands.deleteCodexAuthProfile(selectedProfile.name)
    if (result.status === 'ok') {
      toast.success(t('codexAuth.deleteSuccess'))
      setDeleteDialogOpen(false)
      setSelectedProfile(null)
      await loadProfiles()
    } else {
      toast.error(result.error)
    }
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full text-muted-foreground">
        <p>{t('common.loading')}</p>
      </div>
    )
  }

  return (
    <div className="flex h-full flex-col overflow-y-auto">
      <div className="p-6 space-y-6 max-w-3xl mx-auto w-full">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold">{t('codexAuth.title')}</h1>
            <p className="text-sm text-muted-foreground mt-1">
              {t('codexAuth.description')}
            </p>
          </div>
          <div className="flex gap-2">
            <Button
              size="sm"
              onClick={() => {
                setNewName('')
                setNewLabel('')
                setSaveDialogOpen(true)
              }}
              disabled={!state?.isCurrentOfficial}
              title={
                state?.isCurrentOfficial
                  ? t('codexAuth.saveCurrent')
                  : t('codexAuth.saveCurrentDisabled')
              }
            >
              <Plus className="h-4 w-4 mr-2" />
              {t('codexAuth.saveCurrent')}
            </Button>
          </div>
        </div>

        {/* Current auth status */}
        {state && !state.isCurrentOfficial && (
          <div className="flex items-center gap-2 p-3 rounded-md border bg-amber-50 text-amber-800 dark:bg-amber-950 dark:text-amber-200 border-amber-200 dark:border-amber-800">
            <ShieldAlert className="h-4 w-4 shrink-0" />
            <p className="text-sm">{t('codexAuth.notOfficialWarning')}</p>
          </div>
        )}

        {/* Profile List */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-base">
              {t('codexAuth.profiles.title')}
            </CardTitle>
          </CardHeader>
          <CardContent>
            {!state?.profiles?.length ? (
              <p className="text-sm text-muted-foreground py-4 text-center">
                {t('codexAuth.profiles.empty')}
              </p>
            ) : (
              <div className="space-y-2">
                {state.profiles.map(profile => (
                  <div
                    key={profile.name}
                    className={cn(
                      'flex items-center justify-between p-3 rounded-md border',
                      state.active === profile.name &&
                        'border-primary bg-primary/5'
                    )}
                  >
                    <div className="flex items-center gap-3">
                      <KeyRound className="h-4 w-4 text-muted-foreground" />
                      <div>
                        <div className="flex items-center gap-2">
                          <span className="font-medium">{profile.label}</span>
                          {state.active === profile.name && (
                            <Badge variant="default" className="text-xs">
                              {t('codexAuth.profiles.active')}
                            </Badge>
                          )}
                          {profile.isOfficial && (
                            <Badge variant="secondary" className="text-xs">
                              {t('codexAuth.profiles.official')}
                            </Badge>
                          )}
                        </div>
                        <span className="text-xs text-muted-foreground">
                          {profile.name}
                        </span>
                      </div>
                    </div>
                    <div className="flex items-center gap-1">
                      {state.active !== profile.name && (
                        <ActionButton
                          variant="ghost"
                          size="sm"
                          onClick={() => handleSwitch(profile.name)}
                          title={t('codexAuth.profiles.switch')}
                        >
                          <Check className="h-4 w-4" />
                        </ActionButton>
                      )}
                      <ActionButton
                        variant="ghost"
                        size="sm"
                        onClick={() => {
                          setSelectedProfile(profile)
                          setNewLabel(profile.label)
                          setRenameDialogOpen(true)
                        }}
                        title={t('codexAuth.profiles.rename')}
                      >
                        <Pencil className="h-4 w-4" />
                      </ActionButton>
                      <ActionButton
                        variant="ghost"
                        size="sm"
                        onClick={() => {
                          setSelectedProfile(profile)
                          setDeleteDialogOpen(true)
                        }}
                        disabled={state.active === profile.name}
                        title={t('codexAuth.profiles.delete')}
                      >
                        <Trash2 className="h-4 w-4" />
                      </ActionButton>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Save Dialog */}
      <Dialog open={saveDialogOpen} onOpenChange={setSaveDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('codexAuth.saveDialog.title')}</DialogTitle>
            <DialogDescription>
              {t('codexAuth.saveDialog.description')}
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>{t('codexAuth.saveDialog.nameLabel')}</Label>
              <Input
                value={newName}
                onChange={e => setNewName(e.target.value)}
                placeholder={t('codexAuth.saveDialog.namePlaceholder')}
              />
            </div>
            <div className="space-y-2">
              <Label>{t('codexAuth.saveDialog.labelLabel')}</Label>
              <Input
                value={newLabel}
                onChange={e => setNewLabel(e.target.value)}
                placeholder={t('codexAuth.saveDialog.labelPlaceholder')}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setSaveDialogOpen(false)}>
              {t('common.cancel')}
            </Button>
            <Button onClick={handleSave} disabled={!newName.trim()}>
              {t('common.save')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Rename Dialog */}
      <Dialog open={renameDialogOpen} onOpenChange={setRenameDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('codexAuth.renameDialog.title')}</DialogTitle>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>{t('codexAuth.renameDialog.labelLabel')}</Label>
              <Input
                value={newLabel}
                onChange={e => setNewLabel(e.target.value)}
                placeholder={t('codexAuth.renameDialog.labelPlaceholder')}
              />
            </div>
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setRenameDialogOpen(false)}
            >
              {t('common.cancel')}
            </Button>
            <Button onClick={handleRename} disabled={!newLabel.trim()}>
              {t('common.save')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Delete Confirmation */}
      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {t('codexAuth.deleteDialog.title')}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {t('codexAuth.deleteDialog.description', {
                name: selectedProfile?.label,
              })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <Button variant="destructive" onClick={handleDelete}>
              {t('common.delete')}
            </Button>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Auth Conflict Dialog */}
      <AlertDialog
        open={conflictDialogOpen}
        onOpenChange={setConflictDialogOpen}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {t('codexAuth.conflictDialog.title')}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {t('codexAuth.conflictDialog.description')}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <Button variant="outline" onClick={handleConflictSwitchWithoutSave}>
              {t('codexAuth.conflictDialog.switchWithoutSave')}
            </Button>
            <Button onClick={handleConflictSaveAndSwitch}>
              {t('codexAuth.conflictDialog.saveAndSwitch')}
            </Button>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
