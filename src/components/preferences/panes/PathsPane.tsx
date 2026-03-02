import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { FolderOpen, RotateCcw, Check, AlertCircle, Loader2 } from 'lucide-react'
import { open } from '@tauri-apps/plugin-dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
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
import { SettingsField, SettingsSection } from '../shared/SettingsComponents'
import { commands } from '@/lib/tauri-bindings'
import { logger } from '@/lib/logger'
import { toast } from 'sonner'
import { useOpenClawStore } from '@/store/openclaw-store'
import { useCodexStore } from '@/store/codex-store'
import { useOpenCodeStore } from '@/store/opencode-store'
import { usePlatform } from '@/hooks/use-platform'

type PathKey = 'factory' | 'opencode' | 'opencodeAuth' | 'codex' | 'openclaw'

interface PathItem {
  key: PathKey
  labelKey: string
  descriptionKey: string
}

const pathItems: PathItem[] = [
  {
    key: 'factory',
    labelKey: 'preferences.paths.factory',
    descriptionKey: 'preferences.paths.factoryDescription',
  },
  {
    key: 'opencode',
    labelKey: 'preferences.paths.opencode',
    descriptionKey: 'preferences.paths.opencodeDescription',
  },
  {
    key: 'opencodeAuth',
    labelKey: 'preferences.paths.opencodeAuth',
    descriptionKey: 'preferences.paths.opencodeAuthDescription',
  },
  {
    key: 'codex',
    labelKey: 'preferences.paths.codex',
    descriptionKey: 'preferences.paths.codexDescription',
  },
  {
    key: 'openclaw',
    labelKey: 'preferences.paths.openclaw',
    descriptionKey: 'preferences.paths.openclawDescription',
  },
]

function PathEditor({
  item,
  currentPath,
  isDefault,
  defaultPath,
  onSave,
  onReset,
  isSaving,
  wslButton,
}: {
  item: PathItem
  currentPath: string
  isDefault: boolean
  defaultPath: string
  onSave: (path: string) => void
  onReset: () => void
  isSaving: boolean
  wslButton?: React.ReactNode
}) {
  const { t } = useTranslation()
  const [editValue, setEditValue] = useState(currentPath)
  const [isEditing, setIsEditing] = useState(false)

  // Sync editValue when currentPath changes (e.g. after save/reset)
  const [lastCurrentPath, setLastCurrentPath] = useState(currentPath)
  if (currentPath !== lastCurrentPath) {
    setLastCurrentPath(currentPath)
    setEditValue(currentPath)
    setIsEditing(false)
  }

  const hasChanges = editValue !== currentPath

  const handleBrowse = async () => {
    try {
      const selected = await open({
        directory: true,
        defaultPath: currentPath || defaultPath,
      })
      if (selected) {
        setEditValue(selected)
        setIsEditing(true)
      }
    } catch (error) {
      logger.error('Failed to open directory picker', { error })
    }
  }

  const handleSave = () => {
    onSave(editValue)
    setIsEditing(false)
  }

  const handleCancel = () => {
    setEditValue(currentPath)
    setIsEditing(false)
  }

  return (
    <SettingsField
      label={t(item.labelKey)}
      description={t(item.descriptionKey)}
    >
      <div className="space-y-2">
        <div className="flex items-center gap-2">
          <Input
            value={editValue}
            onChange={e => {
              setEditValue(e.target.value)
              setIsEditing(true)
            }}
            placeholder={defaultPath}
            className="flex-1 font-mono text-xs"
          />
          <Button
            variant="outline"
            size="icon"
            onClick={handleBrowse}
            title={t('preferences.paths.browse')}
          >
            <FolderOpen className="h-4 w-4" />
          </Button>
          {wslButton}
        </div>

        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            {isDefault ? (
              <Badge variant="secondary" className="text-xs">
                {t('preferences.paths.default')}
              </Badge>
            ) : (
              <Badge variant="outline" className="text-xs">
                {t('preferences.paths.custom')}
              </Badge>
            )}
            {isDefault && (
              <span className="text-xs text-muted-foreground">
                {defaultPath}
              </span>
            )}
          </div>

          <div className="flex items-center gap-2">
            {isEditing && hasChanges && (
              <>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={handleCancel}
                  disabled={isSaving}
                >
                  {t('common.cancel')}
                </Button>
                <Button
                  variant="default"
                  size="sm"
                  onClick={handleSave}
                  disabled={isSaving}
                >
                  <Check className="h-4 w-4 mr-1" />
                  {t('common.save')}
                </Button>
              </>
            )}
            {!isDefault && !isEditing && (
              <Button
                variant="ghost"
                size="sm"
                onClick={onReset}
                disabled={isSaving}
              >
                <RotateCcw className="h-4 w-4 mr-1" />
                {t('common.reset')}
              </Button>
            )}
          </div>
        </div>
      </div>
    </SettingsField>
  )
}

function WslDialog({
  open,
  onOpenChange,
  onSelect,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSelect: (path: string) => void
}) {
  const { t } = useTranslation()
  const [selectedDistro, setSelectedDistro] = useState<string>('')
  const [username, setUsername] = useState<string>('')
  const [isLoadingUsername, setIsLoadingUsername] = useState(false)

  // Query WSL info
  const { data: wslInfo, isLoading: isLoadingWsl } = useQuery({
    queryKey: ['wsl-info'],
    queryFn: async () => {
      const result = await commands.getWslInfo()
      if (result.status === 'ok') {
        return result.data
      }
      throw new Error(result.error)
    },
    enabled: open,
  })

  // Fetch username when distro is selected
  const fetchUsername = async (distro: string) => {
    setIsLoadingUsername(true)
    try {
      const result = await commands.getWslUsername(distro)
      if (result.status === 'ok') {
        setUsername(result.data)
      }
    } catch (error) {
      logger.error('Failed to get WSL username', { error })
    } finally {
      setIsLoadingUsername(false)
    }
  }

  const handleDistroChange = (distro: string) => {
    setSelectedDistro(distro)
    fetchUsername(distro)
  }

  const handleConfirm = async () => {
    if (!selectedDistro || !username) return

    try {
      const result = await commands.buildWslPath(selectedDistro, username, 'openclaw')
      if (result.status === 'ok') {
        onSelect(result.data)
        onOpenChange(false)
      }
    } catch (error) {
      logger.error('Failed to build WSL path', { error })
      toast.error(t('toast.error.wslPathFailed'))
    }
  }

  // Reset state when dialog closes
  const handleOpenChange = (newOpen: boolean) => {
    if (!newOpen) {
      setSelectedDistro('')
      setUsername('')
    }
    onOpenChange(newOpen)
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{t('preferences.paths.wslDialog.title')}</DialogTitle>
          <DialogDescription>
            {t('preferences.paths.wslDialog.description')}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {isLoadingWsl ? (
            <div className="flex items-center justify-center py-4">
              <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
            </div>
          ) : !wslInfo?.available ? (
            <div className="text-sm text-muted-foreground text-center py-4">
              {t('preferences.paths.wslDialog.notAvailable')}
            </div>
          ) : (
            <>
              <div className="space-y-2">
                <label className="text-sm font-medium">
                  {t('preferences.paths.wslDialog.distro')}
                </label>
                <Select value={selectedDistro} onValueChange={handleDistroChange}>
                  <SelectTrigger className="w-full">
                    <SelectValue placeholder={t('preferences.paths.wslDialog.selectDistro')} />
                  </SelectTrigger>
                  <SelectContent>
                    {wslInfo.distros.map(distro => (
                      <SelectItem key={distro.name} value={distro.name}>
                        {distro.name}
                        {distro.isDefault && ` (${t('preferences.paths.wslDialog.default')})`}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {selectedDistro && (
                <div className="space-y-2">
                  <label className="text-sm font-medium">
                    {t('preferences.paths.wslDialog.username')}
                  </label>
                  {isLoadingUsername ? (
                    <div className="flex items-center gap-2 text-sm text-muted-foreground">
                      <Loader2 className="h-4 w-4 animate-spin" />
                      {t('preferences.paths.wslDialog.loadingUsername')}
                    </div>
                  ) : (
                    <Input
                      value={username}
                      onChange={e => setUsername(e.target.value)}
                      placeholder={t('preferences.paths.wslDialog.usernamePlaceholder')}
                    />
                  )}
                </div>
              )}

              {selectedDistro && username && (
                <div className="rounded-md bg-muted p-3">
                  <p className="text-xs text-muted-foreground mb-1">
                    {t('preferences.paths.wslDialog.preview')}
                  </p>
                  <p className="text-xs font-mono break-all">
                    \\wsl${selectedDistro}\home\{username}\.openclaw
                  </p>
                </div>
              )}
            </>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => handleOpenChange(false)}>
            {t('common.cancel')}
          </Button>
          <Button
            onClick={handleConfirm}
            disabled={!selectedDistro || !username || isLoadingUsername}
          >
            {t('preferences.paths.wslDialog.apply')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

export function PathsPane() {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const platform = usePlatform()
  const isWindows = platform === 'windows'
  const [wslDialogOpen, setWslDialogOpen] = useState(false)

  const reloadAllConfigStatus = () => {
    useOpenClawStore.getState().loadConfigStatus()
    useCodexStore.getState().loadConfigStatus()
    useOpenCodeStore.getState().loadConfigStatus()
  }

  const { data: effectivePaths, isLoading: isLoadingEffective } = useQuery({
    queryKey: ['effective-paths'],
    queryFn: async () => {
      const result = await commands.getEffectivePaths()
      if (result.status === 'ok') {
        return result.data
      }
      throw new Error(result.error)
    },
  })

  const { data: defaultPaths, isLoading: isLoadingDefault } = useQuery({
    queryKey: ['default-paths'],
    queryFn: async () => {
      const result = await commands.getDefaultPaths()
      if (result.status === 'ok') {
        return result.data
      }
      throw new Error(result.error)
    },
  })

  const saveMutation = useMutation({
    mutationFn: async ({ key, path }: { key: string; path: string }) => {
      const result = await commands.saveConfigPath(key, path)
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['effective-paths'] })
      reloadAllConfigStatus()
      toast.success(t('toast.success.pathSaved'))
    },
    onError: error => {
      logger.error('Failed to save config path', { error })
      toast.error(t('toast.error.pathSaveFailed'))
    },
  })

  const resetMutation = useMutation({
    mutationFn: async (key: string) => {
      const result = await commands.resetConfigPath(key)
      if (result.status === 'error') {
        throw new Error(result.error)
      }
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['effective-paths'] })
      reloadAllConfigStatus()
      toast.success(t('toast.success.pathReset'))
    },
    onError: error => {
      logger.error('Failed to reset config path', { error })
      toast.error(t('toast.error.pathResetFailed'))
    },
  })

  // Query WSL availability for showing the button
  const { data: wslInfo } = useQuery({
    queryKey: ['wsl-info'],
    queryFn: async () => {
      const result = await commands.getWslInfo()
      if (result.status === 'ok') {
        return result.data
      }
      return { available: false, distros: [] }
    },
    enabled: isWindows,
    staleTime: 60000, // Cache for 1 minute
  })

  const handleWslPathSelect = (path: string) => {
    saveMutation.mutate({ key: 'openclaw', path })
  }

  const isLoading = isLoadingEffective || isLoadingDefault
  const isSaving = saveMutation.isPending || resetMutation.isPending

  if (isLoading) {
    return (
      <div className="space-y-6">
        <SettingsSection title={t('preferences.paths.title')}>
          <div className="text-sm text-muted-foreground">
            {t('common.loading')}
          </div>
        </SettingsSection>
      </div>
    )
  }

  if (!effectivePaths || !defaultPaths) {
    return (
      <div className="space-y-6">
        <SettingsSection title={t('preferences.paths.title')}>
          <div className="flex items-center gap-2 text-sm text-destructive">
            <AlertCircle className="h-4 w-4" />
            {t('preferences.paths.loadError')}
          </div>
        </SettingsSection>
      </div>
    )
  }

  const getEffectivePath = (key: PathKey) => {
    return effectivePaths[key]
  }

  const getDefaultPath = (key: PathKey) => {
    return defaultPaths[key].path
  }

  return (
    <div className="space-y-6">
      <SettingsSection title={t('preferences.paths.title')}>
        <p className="text-sm text-muted-foreground mb-4">
          {t('preferences.paths.description')}
        </p>

        {pathItems.map(item => {
          const effective = getEffectivePath(item.key)

          // WSL button only for OpenClaw on Windows with WSL available
          const wslButton =
            item.key === 'openclaw' && isWindows && wslInfo?.available ? (
              <Button
                variant="outline"
                size="sm"
                onClick={() => setWslDialogOpen(true)}
                title={t('preferences.paths.useWsl')}
              >
                WSL
              </Button>
            ) : undefined

          return (
            <PathEditor
              key={item.key}
              item={item}
              currentPath={effective.path}
              isDefault={effective.isDefault}
              defaultPath={getDefaultPath(item.key)}
              onSave={path => saveMutation.mutate({ key: item.key, path })}
              onReset={() => resetMutation.mutate(item.key)}
              isSaving={isSaving}
              wslButton={wslButton}
            />
          )
        })}
      </SettingsSection>

      <div className="text-xs text-muted-foreground">
        <p>{t('preferences.paths.restartNote')}</p>
      </div>

      <WslDialog
        open={wslDialogOpen}
        onOpenChange={setWslDialogOpen}
        onSelect={handleWslPathSelect}
      />
    </div>
  )
}