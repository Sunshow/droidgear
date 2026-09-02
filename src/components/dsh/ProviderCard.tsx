import { useTranslation } from 'react-i18next'
import { Key, KeyRound, Pencil, Trash2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { useDshStore } from '@/store/dsh-store'
import type { DshProviderConfig } from '@/lib/bindings'

interface ProviderCardProps {
  providerId: string
  config: DshProviderConfig | undefined
  onEdit: () => void
  onDelete: () => void
}

export function ProviderCard({
  providerId,
  config,
  onEdit,
  onDelete,
}: ProviderCardProps) {
  const { t } = useTranslation()
  const credentials = useDshStore(state => state.credentials)

  const envName = config?.apiKeyEnv ?? ''
  const hasApiKeyValue = Boolean(envName && credentials[envName])

  return (
    <div className="flex items-center justify-between p-3 border rounded-lg hover:bg-muted/50 transition-colors">
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="font-medium">{providerId}</span>
          {config?.displayName && config.displayName !== providerId && (
            <span className="text-xs text-muted-foreground truncate">
              {config.displayName}
            </span>
          )}
          {config?.api && (
            <Badge variant="outline" className="text-xs">
              {config.api}
            </Badge>
          )}
        </div>
        <div className="text-sm text-muted-foreground mt-1 space-y-0.5">
          {config?.baseURL && <div className="truncate">{config.baseURL}</div>}
          {config?.models && config.models.length > 0 && (
            <div className="text-xs">
              {t('dsh.providers.modelsCount', {
                count: config.models.length,
              })}
            </div>
          )}
        </div>
        <div className="flex items-center gap-2 mt-2">
          {hasApiKeyValue ? (
            <Badge variant="secondary" className="text-xs">
              <Key className="h-3 w-3 mr-1" />
              {t('dsh.provider.apiKeyConfigured')}
            </Badge>
          ) : (
            <Badge variant="outline" className="text-xs text-muted-foreground">
              <KeyRound className="h-3 w-3 mr-1" />
              {t('dsh.provider.apiKeyNotConfigured')}
            </Badge>
          )}
        </div>
      </div>
      <div className="flex items-center gap-1 ml-2">
        <Button
          variant="ghost"
          size="icon"
          onClick={onEdit}
          title={t('common.edit')}
        >
          <Pencil className="h-4 w-4" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          onClick={onDelete}
          title={t('common.delete')}
        >
          <Trash2 className="h-4 w-4" />
        </Button>
      </div>
    </div>
  )
}
