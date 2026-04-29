import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import {
  Cpu,
  Settings,
  FileText,
  Plug,
  MessageSquare,
  TerminalSquare,
  History,
  Rocket,
  FileJson,
  Plus,
  Trash2,
  Play,
  ChevronDown,
} from 'lucide-react'
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
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { useUIStore } from '@/store/ui-store'
import { useModelStore } from '@/store/model-store'
import { useIsWindows } from '@/hooks/use-platform'
import { commands } from '@/lib/tauri-bindings'
import type { DroidSubView } from '@/store/ui-store'
import type { SettingsFileInfo } from '@/lib/tauri-bindings'

interface FeatureItem {
  id: DroidSubView
  labelKey: string
  icon: React.ElementType
}

const features: FeatureItem[] = [
  { id: 'models', labelKey: 'droid.features.models', icon: Cpu },
  { id: 'settings', labelKey: 'droid.features.settings', icon: Settings },
  { id: 'specs', labelKey: 'droid.features.specs', icon: FileText },
  { id: 'missions', labelKey: 'droid.features.missions', icon: Rocket },
  { id: 'mcp', labelKey: 'droid.features.mcp', icon: Plug },
  { id: 'sessions', labelKey: 'droid.features.sessions', icon: MessageSquare },
  { id: 'terminal', labelKey: 'droid.features.terminal', icon: TerminalSquare },
  {
    id: 'legacy-versions',
    labelKey: 'droid.features.legacyVersions',
    icon: History,
  },
]

export function DroidFeatureList() {
  const { t } = useTranslation()
  const droidSubView = useUIStore(state => state.droidSubView)
  const setDroidSubView = useUIStore(state => state.setDroidSubView)
  const incrementDroidRefreshKey = useUIStore(
    state => state.incrementDroidRefreshKey
  )
  const modelHasChanges = useModelStore(state => state.hasChanges)
  const isWindows = useIsWindows()

  const [pendingSubView, setPendingSubView] = useState<DroidSubView | null>(
    null
  )

  // Settings file state
  const [settingsFiles, setSettingsFiles] = useState<SettingsFileInfo[]>([])
  const [activeFile, setActiveFile] = useState<SettingsFileInfo | null>(null)
  const [newFileDialogOpen, setNewFileDialogOpen] = useState(false)
  const [deleteFileDialogOpen, setDeleteFileDialogOpen] = useState(false)
  const [newFileName, setNewFileName] = useState('')
  const [copyFromActive, setCopyFromActive] = useState(true)
  const [fileToDelete, setFileToDelete] = useState<SettingsFileInfo | null>(
    null
  )

  const fetchSettingsFiles = async () => {
    const result = await commands.listDroidSettingsFiles()
    if (result.status === 'ok') {
      setSettingsFiles(result.data)
      const active = result.data.find(f => f.isActive) ?? result.data[0] ?? null
      setActiveFile(active)
    }
  }

  useEffect(() => {
    let cancelled = false
    const fetch = async () => {
      const result = await commands.listDroidSettingsFiles()
      if (cancelled) return
      if (result.status === 'ok') {
        setSettingsFiles(result.data)
        const active =
          result.data.find(f => f.isActive) ?? result.data[0] ?? null
        setActiveFile(active)
      }
    }
    fetch()
    return () => {
      cancelled = true
    }
  }, [])

  const handleSwitchFile = async (file: SettingsFileInfo) => {
    if (file.isActive) return
    const result = await commands.setActiveDroidSettingsFile(
      file.isGlobal ? null : file.name
    )
    if (result.status === 'ok') {
      await fetchSettingsFiles()
      incrementDroidRefreshKey()
      toast.success(t('common.copied'))
    } else {
      toast.error(t('toast.error.generic'))
    }
  }

  const handleCreateFile = async () => {
    if (!newFileName.trim()) return
    const result = await commands.createDroidSettingsFile(
      newFileName.trim(),
      copyFromActive
    )
    if (result.status === 'ok') {
      setNewFileDialogOpen(false)
      setNewFileName('')
      setCopyFromActive(true)
      await fetchSettingsFiles()
      incrementDroidRefreshKey()
    } else {
      toast.error(result.error)
    }
  }

  const handleDeleteFile = async () => {
    if (!fileToDelete) return
    const result = await commands.deleteDroidSettingsFile(fileToDelete.name)
    if (result.status === 'ok') {
      setDeleteFileDialogOpen(false)
      setFileToDelete(null)
      await fetchSettingsFiles()
      incrementDroidRefreshKey()
    } else {
      toast.error(result.error)
    }
  }

  const handleLaunchDroid = async () => {
    const result = await commands.launchDroid()
    if (result.status === 'error') {
      // If launch fails, copy the command to clipboard instead
      const cmdResult = await commands.getDroidLaunchCommand()
      if (cmdResult.status === 'ok') {
        await writeText(cmdResult.data[0])
        toast.info(`Command copied to clipboard: ${cmdResult.data[0]}`)
      } else {
        toast.error(t('toast.error.generic'))
      }
    }
  }

  const handleSubViewChange = (view: DroidSubView) => {
    if (view === droidSubView) return

    // Only check for unsaved changes when leaving models view
    if (droidSubView === 'models' && modelHasChanges) {
      setPendingSubView(view)
    } else {
      setDroidSubView(view)
    }
  }

  const handleSaveAndSwitch = async () => {
    await useModelStore.getState().saveModels()
    if (pendingSubView) {
      setDroidSubView(pendingSubView)
      setPendingSubView(null)
    }
  }

  const handleDiscardAndSwitch = () => {
    useModelStore.getState().resetChanges()
    if (pendingSubView) {
      setDroidSubView(pendingSubView)
      setPendingSubView(null)
    }
  }

  const handleCopyCommand = async (command: string) => {
    await writeText(command)
    toast.success(t('common.copied'))
  }

  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-col gap-1 p-2">
        {features.map(feature => (
          <ActionButton
            key={feature.id}
            variant={droidSubView === feature.id ? 'secondary' : 'ghost'}
            size="sm"
            className={cn('justify-start w-full')}
            onClick={() => handleSubViewChange(feature.id)}
          >
            <feature.icon className="h-4 w-4 mr-2" />
            {t(feature.labelKey)}
          </ActionButton>
        ))}
      </div>

      {/* Settings File Selector */}
      {activeFile && (
        <div className="border-t p-2">
          <div className="flex items-center gap-1 mb-1">
            <FileJson className="h-3 w-3 text-muted-foreground shrink-0" />
            <span className="text-[10px] text-muted-foreground font-medium uppercase tracking-wider">
              {t('droid.settingsFile.sectionLabel')}
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
                      ? t('droid.settingsFile.global')
                      : activeFile.name}
                  </span>
                  <ChevronDown className="h-3 w-3 ml-1 shrink-0" />
                </ActionButton>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="start" className="min-w-[160px]">
                {settingsFiles.map(file => (
                  <ActionDropdownMenuItem
                    key={file.name}
                    onClick={() => handleSwitchFile(file)}
                  >
                    <span className="truncate flex-1">
                      {file.isGlobal
                        ? t('droid.settingsFile.global')
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
              title={t('droid.settingsFile.new')}
            >
              <Plus className="h-3.5 w-3.5" />
            </ActionButton>

            {!activeFile.isGlobal && (
              <ActionButton
                variant="ghost"
                size="icon"
                className="h-7 w-7 shrink-0 text-destructive hover:text-destructive"
                onClick={() => {
                  setFileToDelete(activeFile)
                  setDeleteFileDialogOpen(true)
                }}
                title={t('droid.settingsFile.delete')}
              >
                <Trash2 className="h-3.5 w-3.5" />
              </ActionButton>
            )}

            <ActionButton
              variant="ghost"
              size="icon"
              className="h-7 w-7 shrink-0"
              onClick={handleLaunchDroid}
              title={t('droid.settingsFile.launchTooltip')}
            >
              <Play className="h-3.5 w-3.5" />
            </ActionButton>
          </div>
        </div>
      )}

      {/* Install Section */}
      <div className="mt-auto p-3 border-t text-xs text-muted-foreground">
        <div className="font-medium mb-2">{t('droid.install.title')}</div>
        <Tabs defaultValue={isWindows ? 'windows' : 'unix'} className="w-full">
          <TabsList className="w-full">
            <TabsTrigger value="unix" className="flex-1">
              macOS / Linux
            </TabsTrigger>
            <TabsTrigger value="windows" className="flex-1">
              Windows
            </TabsTrigger>
          </TabsList>
          <TabsContent value="unix">
            <code
              className="block bg-muted p-2 rounded text-xs break-all cursor-pointer hover:bg-muted/80 transition-colors"
              onClick={() =>
                handleCopyCommand('curl -fsSL https://app.factory.ai/cli | sh')
              }
            >
              curl -fsSL https://app.factory.ai/cli | sh
            </code>
          </TabsContent>
          <TabsContent value="windows">
            <code
              className="block bg-muted p-2 rounded text-xs break-all cursor-pointer hover:bg-muted/80 transition-colors"
              onClick={() =>
                handleCopyCommand(
                  'irm https://app.factory.ai/cli/windows | iex'
                )
              }
            >
              irm https://app.factory.ai/cli/windows | iex
            </code>
          </TabsContent>
        </Tabs>
        <a
          href="https://factory.ai"
          target="_blank"
          rel="noopener noreferrer"
          className="text-primary hover:underline mt-2 inline-block"
        >
          {t('droid.install.learnMore')}
        </a>
      </div>

      {/* New Settings File Dialog */}
      <Dialog open={newFileDialogOpen} onOpenChange={setNewFileDialogOpen}>
        <DialogContent className="max-w-sm">
          <DialogHeader>
            <DialogTitle>{t('droid.settingsFile.newTitle')}</DialogTitle>
            <DialogDescription>
              {t('droid.settingsFile.nameLabel')}
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <Input
              placeholder={t('droid.settingsFile.nameLabel')}
              value={newFileName}
              onChange={e => setNewFileName(e.target.value)}
              onKeyDown={e => {
                if (e.key === 'Enter') handleCreateFile()
              }}
            />
            <div className="flex items-center gap-2">
              <Checkbox
                id="copy-from-active"
                checked={copyFromActive}
                onCheckedChange={v => setCopyFromActive(!!v)}
              />
              <Label
                htmlFor="copy-from-active"
                className="text-sm cursor-pointer"
              >
                {t('droid.settingsFile.copyFromActive')}
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

      {/* Delete Settings File Dialog */}
      <AlertDialog
        open={deleteFileDialogOpen}
        onOpenChange={setDeleteFileDialogOpen}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {t('droid.settingsFile.deleteTitle')}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {t('droid.settingsFile.deleteConfirm', {
                name: fileToDelete?.name ?? '',
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

      {/* Unsaved Changes Confirmation Dialog */}
      <AlertDialog
        open={pendingSubView !== null}
        onOpenChange={open => !open && setPendingSubView(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {t('sidebar.unsavedChanges.title')}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {t('sidebar.unsavedChanges.description')}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <ActionButton
              variant="destructive"
              onClick={handleDiscardAndSwitch}
            >
              {t('sidebar.unsavedChanges.discard')}
            </ActionButton>
            <ActionButton onClick={handleSaveAndSwitch}>
              {t('sidebar.unsavedChanges.save')}
            </ActionButton>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
