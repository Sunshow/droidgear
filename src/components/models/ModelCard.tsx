import { useSortable } from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import {
  GripVertical,
  Pencil,
  Trash2,
  Copy,
  Star,
  Heart,
  Wifi,
  WifiOff,
  CircleAlert,
} from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { providerColors, providerLabels } from '@/lib/platform-colors'
import type { CustomModel } from '@/lib/bindings'
import { useConnectivityStore } from '@/store/connectivity-store'

interface ModelCardProps {
  model: CustomModel
  index: number
  selectionMode?: boolean
  isSelected?: boolean
  isDefault?: boolean
  isSpecMode?: boolean
  isFavorite?: boolean
  onSelect?: (index: number, selected: boolean) => void
  onEdit: () => void
  onDelete: () => void
  onCopy: () => void
  onSetDefault?: () => void
  onToggleFavorite?: () => void
  onTestConnection?: () => void
}

export function ModelCard({
  model,
  index,
  selectionMode = false,
  isSelected = false,
  isDefault = false,
  isSpecMode = false,
  isFavorite = false,
  onSelect,
  onEdit,
  onDelete,
  onCopy,
  onSetDefault,
  onToggleFavorite,
  onTestConnection,
}: ModelCardProps) {
  const { t } = useTranslation()
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: `model-${index}` })

  const testResults = useConnectivityStore(state => state.testResults)
  const currentResult = testResults.find(r => r.modelId === model.id)
  const connectionStatus = currentResult?.diagnostics

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  }

  const displayName = model.displayName || model.model

  const getStatusIcon = () => {
    if (!connectionStatus) {
      return null
    }

    if (connectionStatus.success) {
      return (
        <Tooltip>
          <TooltipTrigger asChild>
            <div className="flex items-center text-green-600 dark:text-green-400">
              <Wifi className="h-3 w-3 mr-1" />
              <span className="text-xs">{connectionStatus.latencyMs}ms</span>
            </div>
          </TooltipTrigger>
          <TooltipContent>
            <p>{t('connectivity.connected')}</p>
            <p className="text-xs opacity-80">
              {t('connectivity.latency')}: {connectionStatus.latencyMs}ms
            </p>
            {connectionStatus.responseText && (
              <p className="text-xs opacity-80 max-w-xs break-words">
                {t('connectivity.responseText')}:{' '}
                {connectionStatus.responseText}
              </p>
            )}
            <p className="text-xs opacity-80">
              {t('connectivity.tested')}:{' '}
              {new Date(connectionStatus.timestamp).toLocaleTimeString()}
            </p>
          </TooltipContent>
        </Tooltip>
      )
    } else {
      return (
        <Tooltip>
          <TooltipTrigger asChild>
            <div className="flex items-center text-red-600 dark:text-red-400">
              <WifiOff className="h-3 w-3 mr-1" />
              <CircleAlert className="h-3 w-3" />
            </div>
          </TooltipTrigger>
          <TooltipContent>
            <p className="text-red-600 dark:text-red-400">
              {t('connectivity.disconnected')}
            </p>
            <p className="text-xs opacity-80 max-w-xs break-words">
              {connectionStatus.error || t('connectivity.unknownError')}
            </p>
            <p className="text-xs opacity-80">
              {t('connectivity.tested')}:{' '}
              {new Date(connectionStatus.timestamp).toLocaleTimeString()}
            </p>
          </TooltipContent>
        </Tooltip>
      )
    }
  }

  return (
    <Card
      ref={setNodeRef}
      style={style}
      className="flex items-center gap-3 p-3 mb-2"
    >
      {selectionMode ? (
        <Checkbox
          checked={isSelected}
          onCheckedChange={checked => onSelect?.(index, checked === true)}
          className="h-5 w-5"
        />
      ) : (
        <button
          className="cursor-grab touch-none text-muted-foreground hover:text-foreground"
          {...attributes}
          {...listeners}
        >
          <GripVertical className="h-5 w-5" />
        </button>
      )}

      <div className="flex-1 min-w-0">
        <div className="text-center mb-1">
          <span className="font-medium truncate">{displayName}</span>
        </div>
        <div className="flex items-center justify-center gap-2">
          <Badge variant="secondary" className={providerColors[model.provider]}>
            {providerLabels[model.provider]}
          </Badge>
          {isDefault && isSpecMode ? (
            <Badge
              variant="default"
              className="bg-gradient-to-r from-yellow-500 to-purple-500 text-white border-0"
            >
              {t('models.defaultAndSpecMode')}
            </Badge>
          ) : isDefault ? (
            <Badge variant="default" className="bg-yellow-500 text-white">
              {t('models.default')}
            </Badge>
          ) : isSpecMode ? (
            <Badge variant="default" className="bg-purple-500 text-white">
              {t('models.specMode')}
            </Badge>
          ) : null}
          {getStatusIcon()}
        </div>
        <div className="text-sm text-muted-foreground text-center truncate mt-1">
          {model.model} • {model.baseUrl}
        </div>
        {(() => {
          const reasoningEffortRaw =
            (model.extraArgs?.reasoning &&
            typeof model.extraArgs.reasoning === 'object' &&
            !Array.isArray(model.extraArgs.reasoning) &&
            model.extraArgs.reasoning !== null
              ? (model.extraArgs.reasoning as Record<string, unknown>).effort
              : undefined) ??
            (model.extraArgs?.output_config &&
            typeof model.extraArgs.output_config === 'object' &&
            !Array.isArray(model.extraArgs.output_config) &&
            model.extraArgs.output_config !== null
              ? (model.extraArgs.output_config as Record<string, unknown>)
                  .effort
              : undefined)
          const reasoningEffort =
            typeof reasoningEffortRaw === 'string'
              ? reasoningEffortRaw
              : undefined
          const hasOtherExtraArgs =
            model.extraArgs &&
            Object.keys(model.extraArgs).some(
              k => k !== 'reasoning' && k !== 'output_config'
            )
          const hasExtraHeaders = !!model.extraHeaders
          const is1MContext =
            model.extraHeaders?.['Anthropic-Beta']?.includes(
              'context-1m-2025-08-07'
            ) ?? false

          if (
            !reasoningEffort &&
            !hasOtherExtraArgs &&
            !hasExtraHeaders &&
            !is1MContext
          )
            return null

          return (
            <div className="flex items-center justify-center gap-1 mt-0.5">
              {reasoningEffort && (
                <Badge
                  variant="outline"
                  className={
                    reasoningEffort === 'max'
                      ? 'text-xs px-1.5 py-0 border-amber-400/50 bg-amber-500/10 text-amber-600 dark:text-amber-400'
                      : 'text-xs px-1.5 py-0 border-purple-400/50 bg-purple-500/10 text-purple-600 dark:text-purple-400'
                  }
                >
                  ✨ {String(reasoningEffort)}
                </Badge>
              )}
              {is1MContext && (
                <Badge
                  variant="outline"
                  className="text-xs px-1.5 py-0 border-blue-400/50 bg-blue-500/10 text-blue-600 dark:text-blue-400"
                >
                  1M Context
                </Badge>
              )}
              {hasOtherExtraArgs && (
                <Badge variant="outline" className="text-xs px-1.5 py-0">
                  extraArgs
                </Badge>
              )}
              {hasExtraHeaders && (
                <Badge variant="outline" className="text-xs px-1.5 py-0">
                  extraHeaders
                </Badge>
              )}
            </div>
          )
        })()}
        {(isDefault || isSpecMode) && (
          <div className="text-xs text-muted-foreground mt-1">
            {t('models.defaultHint')}
          </div>
        )}
      </div>

      <div className="flex items-center gap-1">
        {onTestConnection && (
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                onClick={onTestConnection}
                className={connectionStatus ? 'text-muted-foreground' : ''}
              >
                <Wifi className="h-4 w-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              <p>{t('connectivity.testConnection')}</p>
              {connectionStatus && (
                <p className="text-xs opacity-80">
                  {connectionStatus.success
                    ? t('connectivity.lastTested')
                    : t('connectivity.lastFailed')}
                  : {new Date(connectionStatus.timestamp).toLocaleTimeString()}
                </p>
              )}
            </TooltipContent>
          </Tooltip>
        )}
        {onToggleFavorite && (
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                onClick={onToggleFavorite}
                disabled={!model.id}
                className={
                  isFavorite ? 'text-rose-500 hover:text-rose-600' : ''
                }
                aria-label={t(
                  isFavorite
                    ? 'models.favorites.remove'
                    : 'models.favorites.add'
                )}
              >
                <Heart
                  className="h-4 w-4"
                  fill={isFavorite ? 'currentColor' : 'none'}
                />
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              <p>
                {t(
                  isFavorite
                    ? 'models.favorites.remove'
                    : 'models.favorites.add'
                )}
              </p>
            </TooltipContent>
          </Tooltip>
        )}
        {onSetDefault && (
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                onClick={onSetDefault}
                className={
                  isDefault || isSpecMode
                    ? 'text-yellow-500 hover:text-yellow-600'
                    : ''
                }
              >
                <Star
                  className="h-4 w-4"
                  fill={isDefault || isSpecMode ? 'currentColor' : 'none'}
                />
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              <p>{t('models.setAsDefault')}</p>
              <p className="text-xs opacity-80">
                {t('models.setAsDefaultHint')}
              </p>
            </TooltipContent>
          </Tooltip>
        )}
        <Button
          variant="ghost"
          size="icon"
          onClick={onCopy}
          title={t('models.duplicateModel')}
        >
          <Copy className="h-4 w-4" />
        </Button>
        <Button variant="ghost" size="icon" onClick={onEdit}>
          <Pencil className="h-4 w-4" />
        </Button>
        <Button variant="ghost" size="icon" onClick={onDelete}>
          <Trash2 className="h-4 w-4 text-destructive" />
        </Button>
      </div>
    </Card>
  )
}
