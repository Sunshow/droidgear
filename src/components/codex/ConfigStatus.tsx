import { useTranslation } from 'react-i18next'
import { CheckCircle, XCircle, FileText } from 'lucide-react'
import type { CodexConfigStatus } from '@/lib/bindings'

interface ConfigStatusProps {
  status: CodexConfigStatus | null
}

export function ConfigStatus({ status }: ConfigStatusProps) {
  const { t } = useTranslation()

  if (!status) return null

  return (
    <div className="p-4 border rounded-lg space-y-2">
      <h3 className="text-sm font-medium text-muted-foreground">
        {t('codex.configStatus.title')}
      </h3>
      <div className="space-y-1 text-sm">
        <div className="flex items-center gap-2">
          <FileText className="h-4 w-4 text-muted-foreground" />
          <span className="flex-1 truncate">{status.configPath}</span>
          {status.configExists ? (
            <CheckCircle className="h-4 w-4 text-green-500" />
          ) : (
            <XCircle className="h-4 w-4 text-muted-foreground" />
          )}
        </div>
        <div className="flex items-center gap-2">
          <FileText className="h-4 w-4 text-muted-foreground" />
          <span className="flex-1 truncate">{status.authPath}</span>
          {status.authExists ? (
            <CheckCircle className="h-4 w-4 text-green-500" />
          ) : (
            <XCircle className="h-4 w-4 text-muted-foreground" />
          )}
        </div>
      </div>
    </div>
  )
}
