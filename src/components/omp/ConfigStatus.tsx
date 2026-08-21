import { useTranslation } from 'react-i18next'
import { CheckCircle, XCircle, Database } from 'lucide-react'
import type { OmpConfigStatus } from '@/lib/bindings'

interface ConfigStatusProps {
  status: OmpConfigStatus | null
}

export function ConfigStatus({ status }: ConfigStatusProps) {
  const { t } = useTranslation()

  if (!status) return null

  return (
    <div className="rounded-lg border p-3">
      <h3 className="text-sm font-medium text-muted-foreground">
        {t('omp.configStatus.title')}
      </h3>
      <div className="space-y-1 text-sm">
        <div className="flex items-center gap-2">
          {status.configExists ? (
            <CheckCircle className="h-3.5 w-3.5 text-green-500" />
          ) : (
            <XCircle className="h-3.5 w-3.5 text-muted-foreground" />
          )}
          <span>config.yml</span>
          <span className="text-muted-foreground text-xs truncate">
            {status.configPath}
          </span>
        </div>
        <div className="flex items-center gap-2">
          {status.modelsDbExists ? (
            <CheckCircle className="h-3.5 w-3.5 text-green-500" />
          ) : (
            <XCircle className="h-3.5 w-3.5 text-muted-foreground" />
          )}
          <Database className="h-3.5 w-3.5" />
          <span>models.db</span>
        </div>
        <div className="flex items-center gap-2">
          {status.agentDbExists ? (
            <CheckCircle className="h-3.5 w-3.5 text-green-500" />
          ) : (
            <XCircle className="h-3.5 w-3.5 text-muted-foreground" />
          )}
          <Database className="h-3.5 w-3.5" />
          <span>agent.db</span>
        </div>
      </div>
    </div>
  )
}
