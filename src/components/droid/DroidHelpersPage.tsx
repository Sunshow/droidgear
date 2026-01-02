import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { Info, Copy, Check, Pencil } from 'lucide-react'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { toast } from 'sonner'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { commands } from '@/lib/bindings'
import { usePlatform } from '@/hooks/use-platform'

const ENV_VAR_NAME = 'FACTORY_API_KEY'
const DEFAULT_KEY = 'fk-your-key-here'

export function DroidHelpersPage() {
  const { t } = useTranslation()
  const platform = usePlatform()

  const [envValue, setEnvValue] = useState<string | null>(null)
  const [helpDialogOpen, setHelpDialogOpen] = useState(false)
  const [editDialogOpen, setEditDialogOpen] = useState(false)
  const [editValue, setEditValue] = useState('')
  const [copied, setCopied] = useState(false)
  const [cloudSessionSync, setCloudSessionSync] = useState(true)

  const isEnabled = envValue !== null && envValue.length > 0

  useEffect(() => {
    let cancelled = false
    const fetchEnvVar = async () => {
      const value = await commands.getEnvVar(ENV_VAR_NAME)
      if (!cancelled) {
        setEnvValue(value)
      }
    }
    fetchEnvVar()
    return () => {
      cancelled = true
    }
  }, [])

  useEffect(() => {
    let cancelled = false
    const fetchCloudSessionSync = async () => {
      const result = await commands.getCloudSessionSync()
      if (!cancelled && result.status === 'ok') {
        setCloudSessionSync(result.data)
      }
    }
    fetchCloudSessionSync()
    return () => {
      cancelled = true
    }
  }, [])

  const refreshEnvVar = async () => {
    const value = await commands.getEnvVar(ENV_VAR_NAME)
    setEnvValue(value)
  }

  const handleToggle = async (enabled: boolean) => {
    if (enabled) {
      await commands.setEnvVar(ENV_VAR_NAME, DEFAULT_KEY)
      await refreshEnvVar()
      setHelpDialogOpen(true)
    } else {
      await commands.removeEnvVar(ENV_VAR_NAME)
      await refreshEnvVar()
    }
  }

  const handleEdit = () => {
    setEditValue(envValue ?? DEFAULT_KEY)
    setEditDialogOpen(true)
  }

  const handleSaveEdit = async () => {
    if (editValue.trim()) {
      await commands.setEnvVar(ENV_VAR_NAME, editValue.trim())
      await refreshEnvVar()
      setEditDialogOpen(false)
      toast.success(t('common.save'))
    }
  }

  const handleCopy = async () => {
    const keyValue = envValue ?? DEFAULT_KEY
    const command =
      platform === 'windows'
        ? `set ${ENV_VAR_NAME}=${keyValue}`
        : `export ${ENV_VAR_NAME}="${keyValue}"`

    await writeText(command)
    setCopied(true)
    toast.success(t('common.copied'))
    setTimeout(() => setCopied(false), 2000)
  }

  const handleCloudSessionSyncToggle = async (enabled: boolean) => {
    setCloudSessionSync(enabled)
    const result = await commands.saveCloudSessionSync(enabled)
    if (result.status === 'error') {
      // Revert on error
      setCloudSessionSync(!enabled)
      toast.error(t('toast.error.generic'))
    }
  }

  const maskedValue = envValue
    ? envValue.length > 10
      ? `${envValue.substring(0, 6)}${'*'.repeat(Math.min(envValue.length - 10, 16))}${envValue.substring(envValue.length - 4)}`
      : envValue
    : null

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between p-4 border-b">
        <h1 className="text-xl font-semibold">{t('droid.helpers.title')}</h1>
      </div>

      <div className="flex-1 overflow-y-auto p-4">
        <div className="space-y-6">
          {/* Cloud Session Sync Section */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <div className="flex flex-col gap-1">
                <Label
                  htmlFor="cloud-session-sync"
                  className="text-base font-medium"
                >
                  {t('droid.helpers.cloudSessionSync.title')}
                </Label>
                <p className="text-sm text-muted-foreground">
                  {t('droid.helpers.cloudSessionSync.description')}
                </p>
              </div>
              <Switch
                id="cloud-session-sync"
                checked={cloudSessionSync}
                onCheckedChange={handleCloudSessionSyncToggle}
              />
            </div>
          </div>

          {/* Skip Login Section */}
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Label htmlFor="skip-login" className="text-base font-medium">
                  {t('droid.helpers.skipLogin.title')}
                </Label>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-6 w-6"
                  onClick={() => setHelpDialogOpen(true)}
                >
                  <Info className="h-4 w-4 text-muted-foreground" />
                </Button>
              </div>
              <Switch
                id="skip-login"
                checked={isEnabled}
                onCheckedChange={handleToggle}
              />
            </div>

            {/* Environment Variable Display */}
            <div className="space-y-2">
              <Label className="text-sm text-muted-foreground">
                {ENV_VAR_NAME}
              </Label>
              <div className="flex items-center gap-2">
                <div className="flex-1 p-3 bg-muted rounded-md font-mono text-sm">
                  {maskedValue ?? (
                    <span className="text-muted-foreground">
                      {t('droid.helpers.skipLogin.notSet')}
                    </span>
                  )}
                </div>
                {isEnabled && (
                  <Button
                    variant="outline"
                    size="icon"
                    className="shrink-0"
                    onClick={handleEdit}
                  >
                    <Pencil className="h-4 w-4" />
                  </Button>
                )}
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Edit Dialog */}
      <Dialog open={editDialogOpen} onOpenChange={setEditDialogOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>{t('droid.helpers.skipLogin.editTitle')}</DialogTitle>
            <DialogDescription>
              {t('droid.helpers.skipLogin.editDescription')}
            </DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <Input
              value={editValue}
              onChange={e => setEditValue(e.target.value)}
              placeholder={DEFAULT_KEY}
              className="font-mono"
            />
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setEditDialogOpen(false)}>
              {t('common.cancel')}
            </Button>
            <Button onClick={handleSaveEdit}>{t('common.save')}</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Help Dialog */}
      <Dialog open={helpDialogOpen} onOpenChange={setHelpDialogOpen}>
        <DialogContent className="max-w-lg">
          <DialogHeader>
            <DialogTitle>{t('droid.helpers.skipLogin.helpTitle')}</DialogTitle>
            <DialogDescription>
              {t('droid.helpers.skipLogin.helpDescription')}
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4">
            {/* Copy Command */}
            <div className="space-y-2">
              <Label>{t('droid.helpers.skipLogin.copyCommand')}</Label>
              <div className="flex items-center gap-2">
                <code className="flex-1 p-2 bg-muted rounded-md text-sm font-mono overflow-x-auto">
                  {platform === 'windows'
                    ? `set ${ENV_VAR_NAME}=${envValue ?? DEFAULT_KEY}`
                    : `export ${ENV_VAR_NAME}="${envValue ?? DEFAULT_KEY}"`}
                </code>
                <Button
                  variant="outline"
                  size="icon"
                  className="shrink-0"
                  onClick={handleCopy}
                >
                  {copied ? (
                    <Check className="h-4 w-4" />
                  ) : (
                    <Copy className="h-4 w-4" />
                  )}
                </Button>
              </div>
            </div>

            {/* Platform-specific instructions */}
            <div className="space-y-3">
              {platform === 'windows' ? (
                <>
                  <div>
                    <h4 className="font-medium mb-1">CMD:</h4>
                    <code className="block p-2 bg-muted rounded-md text-sm font-mono">
                      {`set ${ENV_VAR_NAME}=${envValue ?? DEFAULT_KEY}`}
                    </code>
                  </div>
                  <div>
                    <h4 className="font-medium mb-1">PowerShell:</h4>
                    <code className="block p-2 bg-muted rounded-md text-sm font-mono">
                      {`$env:${ENV_VAR_NAME}="${envValue ?? DEFAULT_KEY}"`}
                    </code>
                  </div>
                  <div>
                    <h4 className="font-medium mb-1">
                      {t('droid.helpers.skipLogin.permanent')}:
                    </h4>
                    <p className="text-sm text-muted-foreground">
                      {t('droid.helpers.skipLogin.windowsPermanent')}
                    </p>
                  </div>
                </>
              ) : (
                <>
                  <div>
                    <h4 className="font-medium mb-1">
                      {t('droid.helpers.skipLogin.temporary')}:
                    </h4>
                    <code className="block p-2 bg-muted rounded-md text-sm font-mono">
                      {`export ${ENV_VAR_NAME}="${envValue ?? DEFAULT_KEY}"`}
                    </code>
                  </div>
                  <div>
                    <h4 className="font-medium mb-1">
                      {t('droid.helpers.skipLogin.permanent')}:
                    </h4>
                    <p className="text-sm text-muted-foreground">
                      {platform === 'macos'
                        ? t('droid.helpers.skipLogin.macosPermanent')
                        : t('droid.helpers.skipLogin.linuxPermanent')}
                    </p>
                  </div>
                </>
              )}
            </div>

            {/* Warning */}
            <div className="p-3 bg-yellow-500/10 border border-yellow-500/20 rounded-md">
              <p className="text-sm text-yellow-600 dark:text-yellow-500">
                ⚠️ {t('droid.helpers.skipLogin.restartWarning')}
              </p>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  )
}
