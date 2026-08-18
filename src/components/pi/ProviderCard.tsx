import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  Key,
  KeyRound,
  LoaderCircle,
  Pencil,
  Trash2,
  Wifi,
  WifiOff,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import {
  commands,
  type PiProviderConfig,
  type PiProviderTestResult,
} from '@/lib/bindings'

interface ProviderCardProps {
  providerId: string
  config: PiProviderConfig | undefined
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
  const [isTesting, setIsTesting] = useState(false)
  const [testResult, setTestResult] = useState<PiProviderTestResult | null>(
    null
  )
  const [testError, setTestError] = useState<string | null>(null)

  const hasApiKey = config?.apiKey && config.apiKey.length > 0
  const canTest = Boolean(
    config?.models.some(model => model.id.trim().length > 0)
  )

  const handleTestConnection = async () => {
    if (!config || !canTest || isTesting) return

    setIsTesting(true)
    setTestResult(null)
    setTestError(null)
    try {
      const result = await commands.testPiProviderConnection(providerId, config)
      if (result.status === 'ok') {
        setTestResult(result.data)
        if (!result.data.success) {
          setTestError(result.data.error ?? t('connectivity.unknownError'))
        }
      } else {
        setTestError(result.error)
      }
    } catch (error) {
      setTestError(String(error))
    } finally {
      setIsTesting(false)
    }
  }

  const renderTestIcon = () => {
    if (isTesting) {
      return <LoaderCircle data-icon="inline-start" className="animate-spin" />
    }
    if (testResult?.success) {
      return (
        <Wifi
          data-icon="inline-start"
          className="text-green-600 dark:text-green-400"
        />
      )
    }
    if (testError) {
      return <WifiOff data-icon="inline-start" className="text-destructive" />
    }
    return <Wifi data-icon="inline-start" />
  }

  return (
    <div className="flex items-center justify-between p-3 border rounded-lg hover:bg-muted/50 transition-colors">
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="font-medium">{providerId}</span>
          {config?.api && (
            <Badge variant="outline" className="text-xs">
              {config.api}
            </Badge>
          )}
        </div>
        <div className="text-sm text-muted-foreground mt-1 space-y-0.5">
          {config?.baseUrl && <div className="truncate">{config.baseUrl}</div>}
          {config?.models && config.models.length > 0 && (
            <div className="text-xs">
              {t('pi.providers.modelsCount', {
                count: config.models.length,
              })}
            </div>
          )}
        </div>
        <div className="flex items-center gap-2 mt-2">
          {hasApiKey ? (
            <Badge variant="secondary" className="text-xs">
              <Key className="h-3 w-3 mr-1" />
              {t('pi.provider.apiKeyConfigured')}
            </Badge>
          ) : (
            <Badge variant="outline" className="text-xs text-muted-foreground">
              <KeyRound className="h-3 w-3 mr-1" />
              {t('pi.provider.apiKeyNotConfigured')}
            </Badge>
          )}
        </div>
      </div>
      <div className="flex items-center gap-1 ml-2">
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              onClick={handleTestConnection}
              disabled={!canTest || isTesting}
              aria-label={
                isTesting
                  ? t('connectivity.testing')
                  : t('connectivity.testConnection')
              }
            >
              {renderTestIcon()}
            </Button>
          </TooltipTrigger>
          <TooltipContent className="max-w-xs">
            {isTesting ? (
              <p>{t('connectivity.testing')}</p>
            ) : testResult?.success ? (
              <div className="flex flex-col gap-1">
                <p>{t('connectivity.connected')}</p>
                <p className="opacity-80">
                  {t('pi.provider.modelId')}: {testResult.modelId}
                </p>
                <p className="opacity-80">
                  {t('connectivity.latency')}: {testResult.latencyMs}ms
                </p>
                {testResult.responseText ? (
                  <p className="break-words opacity-80">
                    {t('connectivity.responseText')}: {testResult.responseText}
                  </p>
                ) : null}
              </div>
            ) : testError ? (
              <div className="flex flex-col gap-1">
                <p>{t('connectivity.disconnected')}</p>
                <p className="break-words opacity-80">{testError}</p>
              </div>
            ) : (
              <p>
                {canTest
                  ? t('connectivity.testConnection')
                  : t('pi.provider.noModels')}
              </p>
            )}
          </TooltipContent>
        </Tooltip>
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
