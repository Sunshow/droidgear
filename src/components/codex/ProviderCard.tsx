import { useTranslation } from 'react-i18next'
import { Pencil, Trash2, CheckCircle2, Star, Key, KeyRound } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import type { CodexProviderConfig } from '@/lib/bindings'

interface ProviderCardProps {
  providerId: string
  config: CodexProviderConfig
  isActive: boolean
  onEdit: () => void
  onDelete: () => void
  onSetActive: () => void
}

export function ProviderCard({
  providerId,
  config,
  isActive,
  onEdit,
  onDelete,
  onSetActive,
}: ProviderCardProps) {
  const { t } = useTranslation()

  const hasApiKey = config.apiKey && config.apiKey.length > 0

  return (
    <div className="flex items-center justify-between p-3 border rounded-lg hover:bg-muted/50 transition-colors">
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="font-medium">{providerId}</span>
          {config.name && (
            <span className="text-muted-foreground text-sm">
              ({config.name})
            </span>
          )}
          {isActive && (
            <Badge variant="default" className="text-xs">
              <CheckCircle2 className="h-3 w-3 mr-1" />
              {t('codex.provider.active')}
            </Badge>
          )}
        </div>
        <div className="text-sm text-muted-foreground mt-1 space-y-0.5">
          {config.baseUrl && <div className="truncate">{config.baseUrl}</div>}
          {config.model && (
            <div className="text-xs">
              {t('codex.provider.model')}: {config.model}
            </div>
          )}
          <div className="flex items-center gap-2 flex-wrap">
            {config.wireApi && (
              <Badge variant="outline" className="text-xs">
                {config.wireApi}
              </Badge>
            )}
          </div>
          <div className="flex items-center gap-2 mt-1">
            {hasApiKey ? (
              <Badge variant="secondary" className="text-xs">
                <Key className="h-3 w-3 mr-1" />
                {t('codex.provider.apiKeyConfigured')}
              </Badge>
            ) : (
              <Badge
                variant="outline"
                className="text-xs text-muted-foreground"
              >
                <KeyRound className="h-3 w-3 mr-1" />
                {t('codex.provider.apiKeyNotConfigured')}
              </Badge>
            )}
          </div>
        </div>
      </div>
      <div className="flex items-center gap-1 ml-2">
        {!isActive && (
          <Button
            variant="ghost"
            size="icon"
            onClick={onSetActive}
            title={t('codex.provider.setActive')}
          >
            <Star className="h-4 w-4" />
          </Button>
        )}
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
