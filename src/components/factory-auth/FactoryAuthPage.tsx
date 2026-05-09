import { useState, useEffect, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { KeyRound, Plus, Trash2, Pencil, Check } from 'lucide-react'
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
import type { AuthProfile, AuthProfileState } from '@/lib/bindings'

export function FactoryAuthPage() {
  const { t } = useTranslation()
  const [state, setState] = useState<AuthProfileState | null>(null)
  const [loading, setLoading] = useState(true)
  const [saveDialogOpen, setSaveDialogOpen] = useState(false)
  const [renameDialogOpen, setRenameDialogOpen] = useState(false)
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false)
  const [selectedProfile, setSelectedProfile] = useState<AuthProfile | null>(
    null
  )
  const [newName, setNewName] = useState('')
  const [newLabel, setNewLabel] = useState('')

  const loadProfiles = useCallback(async () => {
    const result = await commands.listFactoryAuthProfiles()
    if (result.status === 'ok') {
      setState(result.data)
    } else {
      toast.error(t('factoryAuth.loadError'))
    }
    setLoading(false)
  }, [t])

  useEffect(() => {
    let cancelled = false
    const init = async () => {
      const result = await commands.listFactoryAuthProfiles()
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

  const handleSwitch = async (name: string) => {
    const result = await commands.switchFactoryAuthProfile(name)
    if (result.status === 'ok') {
      toast.success(t('factoryAuth.switchSuccess', { name }))
      await loadProfiles()
    } else {
      toast.error(result.error)
    }
  }

  const handleSave = async () => {
    if (!newName.trim()) return
    const result = await commands.saveCurrentFactoryAuthProfile(
      newName.trim(),
      newLabel.trim() || newName.trim()
    )
    if (result.status === 'ok') {
      toast.success(t('factoryAuth.saveSuccess', { name: newName }))
      setSaveDialogOpen(false)
      setNewName('')
      setNewLabel('')
      await loadProfiles()
    } else {
      toast.error(result.error)
    }
  }

  const handleRename = async () => {
    if (!selectedProfile || !newLabel.trim()) return
    const result = await commands.renameFactoryAuthProfile(
      selectedProfile.name,
      newLabel.trim()
    )
    if (result.status === 'ok') {
      toast.success(t('factoryAuth.renameSuccess'))
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
    const result = await commands.deleteFactoryAuthProfile(selectedProfile.name)
    if (result.status === 'ok') {
      toast.success(t('factoryAuth.deleteSuccess'))
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
            <h1 className="text-2xl font-bold">{t('factoryAuth.title')}</h1>
            <p className="text-sm text-muted-foreground mt-1">
              {t('factoryAuth.description')}
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
            >
              <Plus className="h-4 w-4 mr-2" />
              {t('factoryAuth.saveCurrent')}
            </Button>
          </div>
        </div>

        {/* Profile List */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-base">
              {t('factoryAuth.profiles.title')}
            </CardTitle>
          </CardHeader>
          <CardContent>
            {!state?.profiles?.length ? (
              <p className="text-sm text-muted-foreground py-4 text-center">
                {t('factoryAuth.profiles.empty')}
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
                              {t('factoryAuth.profiles.active')}
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
                          title={t('factoryAuth.profiles.switch')}
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
                        title={t('factoryAuth.profiles.rename')}
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
                        title={t('factoryAuth.profiles.delete')}
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
            <DialogTitle>{t('factoryAuth.saveDialog.title')}</DialogTitle>
            <DialogDescription>
              {t('factoryAuth.saveDialog.description')}
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>{t('factoryAuth.saveDialog.nameLabel')}</Label>
              <Input
                value={newName}
                onChange={e => setNewName(e.target.value)}
                placeholder={t('factoryAuth.saveDialog.namePlaceholder')}
              />
            </div>
            <div className="space-y-2">
              <Label>{t('factoryAuth.saveDialog.labelLabel')}</Label>
              <Input
                value={newLabel}
                onChange={e => setNewLabel(e.target.value)}
                placeholder={t('factoryAuth.saveDialog.labelPlaceholder')}
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
            <DialogTitle>{t('factoryAuth.renameDialog.title')}</DialogTitle>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>{t('factoryAuth.renameDialog.labelLabel')}</Label>
              <Input
                value={newLabel}
                onChange={e => setNewLabel(e.target.value)}
                placeholder={t('factoryAuth.renameDialog.labelPlaceholder')}
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
              {t('factoryAuth.deleteDialog.title')}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {t('factoryAuth.deleteDialog.description', {
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
    </div>
  )
}
