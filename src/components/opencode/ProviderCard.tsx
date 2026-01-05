import { useTranslation } from 'react-i18next'
import { Pencil, Trash2, Key, KeyRound } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import type { OpenCodeProviderConfig, JsonValue } from '@/lib/bindings'

interface ProviderCardProps {
  providerId: string
  config: OpenCodeProviderConfig | undefined
  auth: JsonValue | undefined
  onEdit: () => void
  onDelete: () => void
}

export function ProviderCard({
  providerId,
  config,
  auth,
  onEdit,
  onDelete,
}: ProviderCardProps) {
  const { t } = useTranslation()

  const hasApiKey = auth && typeof auth === 'object' && 'apiKey' in auth

  return (
    <div className="flex items-center justify-between p-3 border rounded-lg hover:bg-muted/50 transition-colors">
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="font-medium">{providerId}</span>
          {config?.name && (
            <span className="text-muted-foreground text-sm">
              ({config.name})
            </span>
          )}
        </div>
        <div className="text-sm text-muted-foreground mt-1 space-y-0.5">
          {config?.options?.baseUrl && (
            <div className="truncate">{config.options.baseUrl}</div>
          )}
          {config?.npm && <div className="text-xs">npm: {config.npm}</div>}
        </div>
        <div className="flex items-center gap-2 mt-2">
          {hasApiKey ? (
            <Badge variant="secondary" className="text-xs">
              <Key className="h-3 w-3 mr-1" />
              {t('opencode.provider.apiKeyConfigured')}
            </Badge>
          ) : (
            <Badge variant="outline" className="text-xs text-muted-foreground">
              <KeyRound className="h-3 w-3 mr-1" />
              {t('opencode.provider.apiKeyNotConfigured')}
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
