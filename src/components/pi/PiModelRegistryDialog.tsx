import { useTranslation } from 'react-i18next'
import { Database } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/ui/command'
import { getAllRegistryModels } from '@/lib/model-registry'
import { createPiModelFromRegistry } from '@/lib/pi-model-metadata'
import type { PiModel } from '@/lib/bindings'

interface PiModelRegistryDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  existingModelIds: string[]
  onSelect: (model: PiModel) => void
}

export function PiModelRegistryDialog({
  open,
  onOpenChange,
  existingModelIds,
  onSelect,
}: PiModelRegistryDialogProps) {
  const { t } = useTranslation()
  const models = getAllRegistryModels()
  const existingIds = new Set(existingModelIds)

  return (
    <CommandDialog
      open={open}
      onOpenChange={onOpenChange}
      title={t('pi.provider.registry.title')}
      description={t('pi.provider.registry.description')}
      className="max-w-2xl"
    >
      <CommandInput placeholder={t('pi.provider.registry.search')} />
      <CommandList className="max-h-[min(60vh,480px)]">
        <CommandEmpty>{t('pi.provider.registry.noResults')}</CommandEmpty>
        <CommandGroup heading={t('pi.provider.registry.title')}>
          {models.map(model => {
            const exists = existingIds.has(model.id)
            return (
              <CommandItem
                key={model.id}
                value={`${model.id} ${model.name} ${model.aliases.join(' ')}`}
                disabled={exists}
                onSelect={() => {
                  onSelect(createPiModelFromRegistry(model))
                  onOpenChange(false)
                }}
              >
                <Database className="size-4" />
                <div className="min-w-0 flex-1">
                  <div className="truncate font-medium">{model.id}</div>
                  <div className="truncate text-xs text-muted-foreground">
                    {model.name}
                  </div>
                </div>
                <Badge variant="outline" className="shrink-0 text-xs">
                  {model.contextWindow.toLocaleString()}
                </Badge>
                {exists && (
                  <span className="shrink-0 text-xs text-muted-foreground">
                    {t('pi.provider.registry.added')}
                  </span>
                )}
              </CommandItem>
            )
          })}
        </CommandGroup>
      </CommandList>
    </CommandDialog>
  )
}
