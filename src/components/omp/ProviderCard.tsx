import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Key, KeyRound, LoaderCircle, Wifi, WifiOff } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import type { OmpProviderTestResult } from '@/lib/bindings'
import { useOmpStore } from '@/store/omp-store'

interface ProviderCardProps {
  providerId: string
  modelCount: number
  hasApiKey: boolean
}

export function ProviderCard({
  providerId,
  modelCount,
  hasApiKey,
}: ProviderCardProps) {
  const { t } = useTranslation()
  const testProvider = useOmpStore(state => state.testProvider)
  const [isTesting, setIsTesting] = useState(false)
  const [testResult, setTestResult] = useState<OmpProviderTestResult | null>(
    null
  )
  const [testError, setTestError] = useState<string | null>(null)

  const handleTestConnection = async () => {
    if (isTesting) return

    setIsTesting(true)
    setTestResult(null)
    setTestError(null)
    try {
      const result = await testProvider(providerId)
      if (result) {
        setTestResult(result)
        if (!result.success) {
          setTestError(result.error ?? t('connectivity.unknownError'))
        }
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

  const renderTooltipContent = () => {
    if (isTesting) {
      return t('connectivity.testing')
    }
    if (testResult?.success) {
      return (
        <div className="space-y-1">
          <div className="font-medium text-green-600 dark:text-green-400">
            {t('connectivity.connected')}
          </div>
          <div>
            {t('connectivity.provider')}: {testResult.providerId}
          </div>
          <div>
            {t('connectivity.model')}: {testResult.modelId}
          </div>
          <div>
            {t('connectivity.latency')}: {testResult.latencyMs}ms
          </div>
        </div>
      )
    }
    if (testError) {
      return (
        <div className="space-y-1">
          <div className="font-medium text-destructive">
            {t('connectivity.failed')}
          </div>
          <div className="text-xs">{testError}</div>
        </div>
      )
    }
    return t('connectivity.testConnection')
  }

  return (
    <div className="flex items-center justify-between p-3 border rounded-lg hover:bg-muted/50 transition-colors">
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="font-medium">{providerId}</span>
        </div>
        <div className="text-sm text-muted-foreground mt-1">
          {t('omp.provider.modelsCount', { count: modelCount })}
        </div>
        <div className="flex items-center gap-2 mt-2">
          {hasApiKey ? (
            <Badge variant="secondary" className="text-xs">
              <Key className="h-3 w-3 mr-1" />
              {t('omp.provider.apiKeyConfigured')}
            </Badge>
          ) : (
            <Badge variant="outline" className="text-xs text-muted-foreground">
              <KeyRound className="h-3 w-3 mr-1" />
              {t('omp.provider.apiKeyNotConfigured')}
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
              disabled={isTesting}
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
            {renderTooltipContent()}
          </TooltipContent>
        </Tooltip>
      </div>
    </div>
  )
}
