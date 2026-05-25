import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  ChevronDown,
  FileJson,
  Play,
  Plus,
  Settings as SettingsIcon,
  Trash2,
  Zap,
} from 'lucide-react'
import { open } from '@tauri-apps/plugin-dialog'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { toast } from 'sonner'
import { cn } from '@/lib/utils'
import { ActionButton } from '@/components/ui/action-button'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Checkbox } from '@/components/ui/checkbox'
import { Label } from '@/components/ui/label'
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
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { ActionDropdownMenuItem } from '@/components/ui/action-dropdown-menu-item'
import { commands } from '@/lib/bindings'
import { useClaudeSettingsStore } from '@/store/claude-settings-store'

const features = [
  { id: 'settings', labelKey: 'claude.features.settings', icon: SettingsIcon },
] as const

export function ClaudeFeatureList() {
  const { t } = useTranslation()
  const files = useClaudeSettingsStore(state => state.files)
  const activeFile = useClaudeSettingsStore(state => state.activeFile)
  const currentJson = useClaudeSettingsStore(state => state.currentJson)
  const isLaunching = useClaudeSettingsStore(state => state.isLaunching)
  const loadFiles = useClaudeSettingsStore(state => state.loadFiles)
  const selectFile = useClaudeSettingsStore(state => state.selectFile)
  const createFile = useClaudeSettingsStore(state => state.createFile)
  const deleteFile = useClaudeSettingsStore(state => state.deleteFile)
  const launch = useClaudeSettingsStore(state => state.launch)

  const [newFileDialogOpen, setNewFileDialogOpen] = useState(false)
  const [deleteFileDialogOpen, setDeleteFileDialogOpen] = useState(false)
  const [newFileName, setNewFileName] = useState('')
  const [copyFromActive, setCopyFromActive] = useState(true)

  useEffect(() => {
    loadFiles()
  }, [loadFiles])

  const bypassDisabled = currentJson?.disableBypassPermissionsMode === 'disable'

  const handleCreateFile = async () => {
    if (!newFileName.trim()) return
    try {
      await createFile(newFileName.trim(), copyFromActive)
      setNewFileDialogOpen(false)
      setNewFileName('')
      setCopyFromActive(true)
    } catch (err) {
      toast.error(String(err))
    }
  }

  const handleDeleteFile = async () => {
    if (!activeFile || activeFile.isGlobal) return
    try {
      await deleteFile(activeFile.name)
      setDeleteFileDialogOpen(false)
    } catch (err) {
      toast.error(String(err))
    }
  }

  const handleLaunch = async (skipDangerous: boolean) => {
    if (isLaunching) return
    const selected = await open({
      directory: true,
      multiple: false,
      title: t('claude.settingsFile.selectDirectory'),
    })
    if (!selected) return
    const cwd = selected as string

    const ok = await launch(cwd, skipDangerous)
    if (!ok) {
      const cmdResult =
        await commands.getClaudeSettingsLaunchCommand(skipDangerous)
      if (cmdResult.status === 'ok') {
        await writeText(cmdResult.data[0])
        toast.info(
          `${t('claude.settingsFile.launchFailedCopy')}: ${cmdResult.data[0]}`
        )
      } else {
        toast.error(t('toast.error.generic'))
      }
      return
    }
    toast.success(
      skipDangerous
        ? t('claude.settingsFile.launchSuccessSkip')
        : t('claude.settingsFile.launchSuccess')
    )
  }

  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-col gap-1 p-2">
        {features.map(feature => (
          <ActionButton
            key={feature.id}
            variant="secondary"
            size="sm"
            className={cn('justify-start w-full')}
          >
            <feature.icon className="h-4 w-4 mr-2" />
            {t(feature.labelKey)}
          </ActionButton>
        ))}
      </div>

      {activeFile && (
        <div className="border-t p-2">
          <div className="flex items-center gap-1 mb-1">
            <FileJson className="h-3 w-3 text-muted-foreground shrink-0" />
            <span className="text-[10px] text-muted-foreground font-medium uppercase tracking-wider">
              {t('claude.settingsFile.sectionLabel')}
            </span>
          </div>
          <div className="flex items-center gap-1">
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <ActionButton
                  variant="ghost"
                  size="sm"
                  className="flex-1 justify-start text-xs h-7"
                >
                  <span className="truncate">
                    {activeFile.isGlobal
                      ? t('claude.settingsFile.global')
                      : activeFile.name}
                  </span>
                  <ChevronDown className="h-3 w-3 ml-1 shrink-0" />
                </ActionButton>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="start" className="min-w-40">
                {files.map(file => (
                  <ActionDropdownMenuItem
                    key={file.name}
                    onClick={() => selectFile(file)}
                  >
                    <span className="truncate flex-1">
                      {file.isGlobal
                        ? t('claude.settingsFile.global')
                        : file.name}
                    </span>
                    {file.isActive && (
                      <span className="text-[10px] text-muted-foreground ml-2">
                        ✓
                      </span>
                    )}
                  </ActionDropdownMenuItem>
                ))}
              </DropdownMenuContent>
            </DropdownMenu>

            <ActionButton
              variant="ghost"
              size="icon"
              className="h-7 w-7 shrink-0"
              onClick={() => {
                setNewFileName('')
                setCopyFromActive(true)
                setNewFileDialogOpen(true)
              }}
              title={t('claude.settingsFile.new')}
            >
              <Plus className="h-3.5 w-3.5" />
            </ActionButton>

            {!activeFile.isGlobal && (
              <ActionButton
                variant="ghost"
                size="icon"
                className="h-7 w-7 shrink-0 text-destructive hover:text-destructive"
                onClick={() => setDeleteFileDialogOpen(true)}
                title={t('claude.settingsFile.delete')}
              >
                <Trash2 className="h-3.5 w-3.5" />
              </ActionButton>
            )}
          </div>

          <div className="mt-2 flex flex-col gap-1">
            <ActionButton
              variant="default"
              size="sm"
              className="justify-start h-8"
              onClick={() => handleLaunch(false)}
              disabled={isLaunching}
              title={t('claude.settingsFile.runTooltip')}
            >
              <Play className="h-3.5 w-3.5 mr-2" />
              {t('claude.settingsFile.run')}
            </ActionButton>
            <ActionButton
              variant="outline"
              size="sm"
              className="justify-start h-8 border-amber-500/40 text-amber-700 dark:text-amber-400 hover:bg-amber-500/10"
              onClick={() => handleLaunch(true)}
              disabled={isLaunching || bypassDisabled}
              title={
                bypassDisabled
                  ? t('claude.settingsFile.runSkipDisabledTooltip')
                  : t('claude.settingsFile.runSkipTooltip')
              }
            >
              <Zap className="h-3.5 w-3.5 mr-2" />
              {t('claude.settingsFile.runSkip')}
            </ActionButton>
          </div>
        </div>
      )}

      <div className="mt-auto p-3 border-t text-xs text-muted-foreground">
        {t('claude.features.hint')}
      </div>

      <Dialog open={newFileDialogOpen} onOpenChange={setNewFileDialogOpen}>
        <DialogContent className="max-w-sm">
          <DialogHeader>
            <DialogTitle>{t('claude.settingsFile.newTitle')}</DialogTitle>
            <DialogDescription>
              {t('claude.settingsFile.nameLabel')}
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <Input
              placeholder={t('claude.settingsFile.nameLabel')}
              value={newFileName}
              onChange={e => setNewFileName(e.target.value)}
              onKeyDown={e => {
                if (e.key === 'Enter') handleCreateFile()
              }}
            />
            <div className="flex items-center gap-2">
              <Checkbox
                id="claude-copy-from-active"
                checked={copyFromActive}
                onCheckedChange={v => setCopyFromActive(!!v)}
              />
              <Label
                htmlFor="claude-copy-from-active"
                className="text-sm cursor-pointer"
              >
                {t('claude.settingsFile.copyFromActive')}
              </Label>
            </div>
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setNewFileDialogOpen(false)}
            >
              {t('common.cancel')}
            </Button>
            <Button onClick={handleCreateFile} disabled={!newFileName.trim()}>
              {t('common.create')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <AlertDialog
        open={deleteFileDialogOpen}
        onOpenChange={setDeleteFileDialogOpen}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {t('claude.settingsFile.deleteTitle')}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {t('claude.settingsFile.deleteConfirm', {
                name: activeFile?.name ?? '',
              })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <Button variant="destructive" onClick={handleDeleteFile}>
              {t('common.delete')}
            </Button>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
