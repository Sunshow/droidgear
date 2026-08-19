import { useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { CheckSquare, Heart, Trash2, X } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import type { CustomModel } from '@/lib/bindings'
import { getAllRegistryModels } from '@/lib/model-registry'

interface ModelFavoritesDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  favorites: string[]
  models: CustomModel[]
  onSave: (favorites: string[]) => Promise<void>
}

interface FavoriteOption {
  id: string
  name: string
  isByok: boolean
}

function uniqueOptions(options: FavoriteOption[]): FavoriteOption[] {
  const seen = new Set<string>()
  return options.filter(option => {
    if (seen.has(option.id)) return false
    seen.add(option.id)
    return true
  })
}

export function ModelFavoritesDialog({
  open,
  onOpenChange,
  favorites,
  models,
  onSave,
}: ModelFavoritesDialogProps) {
  const { t } = useTranslation()
  const [selectionMode, setSelectionMode] = useState(false)
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set())
  const [isSaving, setIsSaving] = useState(false)

  const byokModels = useMemo(
    () =>
      models.filter(
        model => typeof model.id === 'string' && model.id.startsWith('custom:')
      ),
    [models]
  )
  const byokIds = useMemo(
    () => new Set(byokModels.map(model => model.id as string)),
    [byokModels]
  )

  // Keep stale custom IDs out of the management view while retaining every
  // non-BYOK ID already present in settings.json.
  const visibleFavorites = useMemo(
    () =>
      favorites.filter(
        favorite => !favorite.startsWith('custom:') || byokIds.has(favorite)
      ),
    [favorites, byokIds]
  )
  const favoriteIds = useMemo(
    () => new Set(visibleFavorites),
    [visibleFavorites]
  )

  const options = useMemo(() => {
    // Built-in rows are limited to IDs already persisted in settings. The
    // current BYOK list remains available so its models can be favorited here.
    const registryOptions = getAllRegistryModels()
      .filter(model => favoriteIds.has(model.id))
      .map(model => ({
        id: model.id,
        name: model.name,
        isByok: false,
      }))
    const registryIds = new Set(registryOptions.map(option => option.id))
    const customOptions = byokModels.map(model => ({
      id: model.id as string,
      name: model.displayName || model.model || (model.id as string),
      isByok: true,
    }))
    const settingsOnlyOptions = visibleFavorites
      .filter(favorite => !registryIds.has(favorite) && !byokIds.has(favorite))
      .map(id => ({ id, name: id, isByok: false }))

    const allOptions = uniqueOptions([
      ...settingsOnlyOptions,
      ...registryOptions,
      ...customOptions,
    ])

    const optionsById = new Map(
      allOptions.map(option => [option.id, option] as const)
    )
    const favoriteOptions = visibleFavorites.flatMap(favorite => {
      const option = optionsById.get(favorite)
      return option ? [option] : []
    })
    const displayedFavoriteIds = new Set(
      favoriteOptions.map(option => option.id)
    )

    return [
      ...favoriteOptions,
      ...allOptions.filter(option => !displayedFavoriteIds.has(option.id)),
    ]
  }, [byokIds, byokModels, favoriteIds, visibleFavorites])

  useEffect(() => {
    if (!open) return
    setSelectionMode(false)
    setSelectedIds(new Set())
  }, [open, favorites])

  const persist = async (nextFavorites: string[]) => {
    setIsSaving(true)
    try {
      await onSave(nextFavorites)
      setSelectedIds(new Set())
    } finally {
      setIsSaving(false)
    }
  }

  const handleFavoriteToggle = async (modelId: string, checked: boolean) => {
    const nextFavorites = checked
      ? [...visibleFavorites, modelId]
      : visibleFavorites.filter(favorite => favorite !== modelId)
    await persist(nextFavorites)
  }

  const handleSelect = (modelId: string, checked: boolean) => {
    setSelectedIds(previous => {
      const next = new Set(previous)
      if (checked) {
        next.add(modelId)
      } else {
        next.delete(modelId)
      }
      return next
    })
  }

  const handleDeleteSelected = async () => {
    if (selectedIds.size === 0) return
    await persist(
      visibleFavorites.filter(favorite => !selectedIds.has(favorite))
    )
    setSelectionMode(false)
  }

  const handleDeleteOne = async (modelId: string) => {
    await persist(visibleFavorites.filter(favorite => favorite !== modelId))
  }

  const allFavoritesSelected =
    visibleFavorites.length > 0 &&
    visibleFavorites.every(favorite => selectedIds.has(favorite))

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent
        className="max-w-2xl"
        onCloseAutoFocus={event => event.preventDefault()}
      >
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Heart className="h-4 w-4 text-rose-500" fill="currentColor" />
            {t('models.favorites.title')}
          </DialogTitle>
          <DialogDescription>
            {t('models.favorites.description')}
          </DialogDescription>
        </DialogHeader>

        <div className="flex flex-wrap items-center gap-2">
          {!selectionMode ? (
            <Button
              variant="outline"
              size="sm"
              onClick={() => setSelectionMode(true)}
              disabled={visibleFavorites.length === 0 || isSaving}
            >
              <CheckSquare className="mr-1.5 h-4 w-4" />
              {t('models.favorites.batchDelete')}
            </Button>
          ) : (
            <>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setSelectedIds(new Set(visibleFavorites))}
                disabled={allFavoritesSelected || isSaving}
              >
                {t('models.favorites.selectAll')}
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setSelectedIds(new Set())}
                disabled={selectedIds.size === 0 || isSaving}
              >
                <X className="mr-1 h-3.5 w-3.5" />
                {t('models.favorites.deselectAll')}
              </Button>
              <Button
                variant="destructive"
                size="sm"
                onClick={() => void handleDeleteSelected()}
                disabled={selectedIds.size === 0 || isSaving}
              >
                <Trash2 className="mr-1 h-3.5 w-3.5" />
                {t('models.favorites.deleteSelected', {
                  count: selectedIds.size,
                })}
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => {
                  setSelectionMode(false)
                  setSelectedIds(new Set())
                }}
              >
                <X className="mr-1 h-3.5 w-3.5" />
                {t('common.cancel')}
              </Button>
            </>
          )}
          <div className="flex-1" />
          <Badge variant="secondary">
            {t('models.favorites.count', { count: visibleFavorites.length })}
          </Badge>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => void persist([])}
            disabled={visibleFavorites.length === 0 || isSaving}
          >
            {t('models.favorites.clearAll')}
          </Button>
        </div>

        <div className="max-h-[min(60vh,520px)] overflow-y-auto rounded-md border">
          {options.length === 0 ? (
            <p className="p-6 text-center text-sm text-muted-foreground">
              {t('models.favorites.empty')}
            </p>
          ) : (
            <div className="divide-y">
              {options.map(option => {
                const isFavorite = favoriteIds.has(option.id)
                return (
                  <div
                    key={option.id}
                    className="flex items-center gap-3 p-3 hover:bg-muted/40"
                  >
                    {selectionMode ? (
                      <Checkbox
                        checked={selectedIds.has(option.id)}
                        onCheckedChange={checked =>
                          handleSelect(option.id, checked === true)
                        }
                        disabled={!isFavorite || isSaving}
                        aria-label={t('models.favorites.selectItem', {
                          model: option.id,
                        })}
                      />
                    ) : (
                      <Checkbox
                        checked={isFavorite}
                        onCheckedChange={checked =>
                          void handleFavoriteToggle(option.id, checked === true)
                        }
                        disabled={isSaving}
                        aria-label={t(
                          isFavorite
                            ? 'models.favorites.remove'
                            : 'models.favorites.add'
                        )}
                      />
                    )}
                    <Heart
                      className={`h-4 w-4 shrink-0 ${
                        isFavorite
                          ? 'text-rose-500'
                          : 'text-muted-foreground/40'
                      }`}
                      fill={isFavorite ? 'currentColor' : 'none'}
                    />
                    <div className="min-w-0 flex-1">
                      <div className="flex items-center gap-2">
                        <span className="truncate font-medium">
                          {option.name}
                        </span>
                        <Badge
                          variant="outline"
                          className="shrink-0 text-[10px]"
                        >
                          {option.isByok
                            ? t('models.favorites.byok')
                            : t('models.favorites.builtin')}
                        </Badge>
                      </div>
                      <div className="truncate text-xs text-muted-foreground">
                        {option.id}
                      </div>
                    </div>
                    {isFavorite && !selectionMode && (
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => void handleDeleteOne(option.id)}
                        disabled={isSaving}
                        title={t('models.favorites.deleteOne')}
                        aria-label={t('models.favorites.deleteOne')}
                      >
                        <Trash2 className="h-4 w-4 text-destructive" />
                      </Button>
                    )}
                  </div>
                )
              })}
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t('common.cancel')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
