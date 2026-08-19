import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Folder, Loader2, Plus, RefreshCw, Trash2 } from 'lucide-react'
import { open } from '@tauri-apps/plugin-dialog'
import { toast } from 'sonner'
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { commands, type TrustedFolder } from '@/lib/tauri-bindings'
import { useUIStore } from '@/store/ui-store'

function formatTrustedAt(value: string) {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleString()
}

export function TrustedFoldersPage() {
  const { t } = useTranslation()
  const droidRefreshKey = useUIStore(state => state.droidRefreshKey)
  const [folders, setFolders] = useState<TrustedFolder[]>([])
  const [reloadNonce, setReloadNonce] = useState(0)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [adding, setAdding] = useState(false)
  const [selectedPaths, setSelectedPaths] = useState<Set<string>>(
    () => new Set()
  )
  const [removing, setRemoving] = useState(false)
  const [removingPath, setRemovingPath] = useState<string | null>(null)
  const [foldersToRemove, setFoldersToRemove] = useState<TrustedFolder[]>([])

  useEffect(() => {
    let cancelled = false

    const loadFolders = async () => {
      setLoading(true)
      setError(null)
      try {
        const result = await commands.listDroidTrustedFolders()
        if (cancelled) return
        if (result.status === 'ok') {
          setFolders(result.data)
          const availablePaths = new Set(result.data.map(folder => folder.path))
          setSelectedPaths(
            previous =>
              new Set([...previous].filter(path => availablePaths.has(path)))
          )
        } else {
          setError(result.error)
        }
      } catch (requestError) {
        if (!cancelled) setError(String(requestError))
      } finally {
        if (!cancelled) setLoading(false)
      }
    }

    loadFolders()
    return () => {
      cancelled = true
    }
  }, [droidRefreshKey, reloadNonce])

  const handleRefresh = () => setReloadNonce(value => value + 1)

  const selectedFolders = folders.filter(folder =>
    selectedPaths.has(folder.path)
  )
  const allSelected =
    folders.length > 0 &&
    folders.every(folder => selectedPaths.has(folder.path))
  const someSelected = selectedFolders.length > 0 && !allSelected
  const dialogOpen = foldersToRemove.length > 0
  const controlsDisabled = adding || removing || dialogOpen

  const handleToggleAll = () => {
    setSelectedPaths(previous => {
      if (
        folders.length > 0 &&
        folders.every(folder => previous.has(folder.path))
      ) {
        return new Set()
      }
      return new Set(folders.map(folder => folder.path))
    })
  }

  const handleTogglePath = (path: string) => {
    setSelectedPaths(previous => {
      const next = new Set(previous)
      if (next.has(path)) {
        next.delete(path)
      } else {
        next.add(path)
      }
      return next
    })
  }

  const handleAdd = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t('droid.trustedFolders.selectDirectory'),
    })
    if (typeof selected !== 'string' || !selected) return

    setAdding(true)
    setError(null)
    try {
      const result = await commands.addDroidTrustedFolder(selected)
      if (result.status === 'ok') {
        setFolders(previous =>
          [
            ...previous.filter(folder => folder.path !== result.data.path),
            result.data,
          ].sort((left, right) => left.path.localeCompare(right.path))
        )
        toast.success(t('common.saved'))
      } else {
        toast.error(result.error)
      }
    } catch (requestError) {
      toast.error(String(requestError))
    } finally {
      setAdding(false)
    }
  }

  const handleRemove = async () => {
    if (foldersToRemove.length === 0) return
    const paths = foldersToRemove.map(folder => folder.path)
    const singlePath = paths.length === 1 ? (paths[0] ?? null) : null
    setRemoving(true)
    setRemovingPath(singlePath)
    try {
      const result =
        singlePath !== null
          ? await commands.removeDroidTrustedFolder(singlePath)
          : await commands.removeDroidTrustedFolders(paths)
      if (result.status === 'ok') {
        const removedPaths = new Set(paths)
        setFolders(previous =>
          previous.filter(folder => !removedPaths.has(folder.path))
        )
        setSelectedPaths(previous => {
          const next = new Set(previous)
          paths.forEach(path => next.delete(path))
          return next
        })
        setFoldersToRemove([])
        toast.success(
          paths.length === 1
            ? t('common.deleted')
            : t('droid.trustedFolders.removedSelected', {
                count: paths.length,
              })
        )
      } else {
        toast.error(result.error)
      }
    } catch (requestError) {
      toast.error(String(requestError))
    } finally {
      setRemoving(false)
      setRemovingPath(null)
    }
  }

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center justify-between border-b p-4">
        <h1 className="text-xl font-semibold">
          {t('droid.trustedFolders.title')}
        </h1>
        <div className="flex items-center gap-2">
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                onClick={handleRefresh}
                disabled={loading || controlsDisabled}
                aria-label={t('common.refresh')}
              >
                <RefreshCw className="h-4 w-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>{t('common.refresh')}</TooltipContent>
          </Tooltip>
          <Button onClick={handleAdd} disabled={controlsDisabled}>
            {adding ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <Plus className="h-4 w-4" />
            )}
            {t('droid.trustedFolders.add')}
          </Button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-4">
        <p className="mb-4 text-sm text-muted-foreground">
          {t('droid.trustedFolders.description')}
        </p>

        {loading ? (
          <div className="flex items-center justify-center py-12 text-muted-foreground">
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            {t('common.loading')}
          </div>
        ) : error ? (
          <div className="flex flex-col items-center gap-3 py-12 text-center">
            <p className="text-sm text-destructive">{error}</p>
            <Button variant="outline" onClick={handleRefresh}>
              <RefreshCw className="h-4 w-4" />
              {t('common.refresh')}
            </Button>
          </div>
        ) : folders.length === 0 ? (
          <div className="flex flex-col items-center justify-center rounded-md border border-dashed py-12 text-center text-muted-foreground">
            <Folder className="mb-3 h-8 w-8" />
            <p className="text-sm">{t('droid.trustedFolders.empty')}</p>
          </div>
        ) : (
          <>
            <div className="mb-3 flex items-center justify-between gap-3">
              <label className="flex min-w-0 items-center gap-2 text-sm">
                <Checkbox
                  checked={
                    allSelected ? true : someSelected ? 'indeterminate' : false
                  }
                  onCheckedChange={handleToggleAll}
                  disabled={controlsDisabled}
                  aria-label={t(
                    allSelected ? 'common.deselectAll' : 'common.selectAll'
                  )}
                />
                <span className="truncate">
                  {t(allSelected ? 'common.deselectAll' : 'common.selectAll')}
                </span>
              </label>
              {selectedFolders.length > 0 && (
                <Button
                  variant="destructive"
                  size="sm"
                  onClick={() => setFoldersToRemove(selectedFolders)}
                  disabled={controlsDisabled}
                >
                  <Trash2 className="h-4 w-4" />
                  {t('droid.trustedFolders.removeSelected', {
                    count: selectedFolders.length,
                  })}
                </Button>
              )}
            </div>
            <div className="divide-y rounded-md border">
              {folders.map(folder => (
                <div
                  key={folder.path}
                  className="flex min-w-0 items-center gap-3 p-3"
                >
                  <Checkbox
                    checked={selectedPaths.has(folder.path)}
                    onCheckedChange={() => handleTogglePath(folder.path)}
                    disabled={controlsDisabled}
                    aria-label={t('droid.trustedFolders.selectFolder', {
                      path: folder.path,
                    })}
                  />
                  <Folder className="h-4 w-4 shrink-0 text-muted-foreground" />
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm" title={folder.path}>
                      {folder.path}
                    </p>
                    <p className="text-xs text-muted-foreground">
                      {t('droid.trustedFolders.trustedAt', {
                        date: formatTrustedAt(folder.trustedAt),
                      })}
                    </p>
                  </div>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="shrink-0 text-destructive hover:text-destructive"
                        onClick={() => setFoldersToRemove([folder])}
                        disabled={controlsDisabled}
                        aria-label={t('droid.trustedFolders.remove')}
                      >
                        {removingPath === folder.path ? (
                          <Loader2 className="h-4 w-4 animate-spin" />
                        ) : (
                          <Trash2 className="h-4 w-4" />
                        )}
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>
                      {t('droid.trustedFolders.remove')}
                    </TooltipContent>
                  </Tooltip>
                </div>
              ))}
            </div>
          </>
        )}
      </div>

      <AlertDialog
        open={dialogOpen}
        onOpenChange={open => {
          if (!open && !removing) setFoldersToRemove([])
        }}
      >
        <AlertDialogContent onCloseAutoFocus={event => event.preventDefault()}>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {t(
                foldersToRemove.length > 1
                  ? 'droid.trustedFolders.removeSelectedTitle'
                  : 'droid.trustedFolders.removeTitle'
              )}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {foldersToRemove.length > 1
                ? t('droid.trustedFolders.removeSelectedConfirm', {
                    count: foldersToRemove.length,
                  })
                : t('droid.trustedFolders.removeConfirm', {
                    path: foldersToRemove[0]?.path ?? '',
                  })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={removing}>
              {t('common.cancel')}
            </AlertDialogCancel>
            <Button
              variant="destructive"
              onClick={handleRemove}
              disabled={removing}
            >
              {removing && <Loader2 className="h-4 w-4 animate-spin" />}
              {t('common.delete')}
            </Button>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
