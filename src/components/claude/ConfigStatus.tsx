import { useTranslation } from 'react-i18next'
import {
  AlertCircle,
  CheckCircle,
  FileText,
  FolderOpen,
  XCircle,
} from 'lucide-react'
import type { ClaudeConfigStatus } from '@/lib/bindings'

interface ConfigStatusProps {
  status: ClaudeConfigStatus | null
}

export function ConfigStatus({ status }: ConfigStatusProps) {
  const { t } = useTranslation()

  if (!status) return null

  return (
    <div className="p-4 border rounded-lg space-y-3">
      <h3 className="text-sm font-medium text-muted-foreground">
        {t('claude.configStatus.title')}
      </h3>
      <div className="space-y-1 text-sm">
        <div className="flex items-center gap-2">
          <FileText className="h-4 w-4 text-muted-foreground shrink-0" />
          <code className="flex-1 truncate text-xs bg-muted px-1 py-0.5 rounded select-all cursor-text">
            {status.settingsPath}
          </code>
          {status.settingsExists ? (
            <>
              <CheckCircle className="h-4 w-4 text-green-500 shrink-0" />
              <span className="text-xs text-green-600 shrink-0">
                {t('common.exists')}
              </span>
            </>
          ) : (
            <>
              <XCircle className="h-4 w-4 text-muted-foreground shrink-0" />
              <span className="text-xs text-muted-foreground shrink-0">
                {t('common.missing')}
              </span>
            </>
          )}
        </div>
        <div className="flex items-center gap-2">
          <FolderOpen className="h-4 w-4 text-muted-foreground shrink-0" />
          <code className="flex-1 truncate text-xs bg-muted px-1 py-0.5 rounded select-all cursor-text">
            {status.configDir}
          </code>
        </div>
      </div>
      {status.parseError && (
        <div className="p-3 bg-destructive/10 border border-destructive/20 rounded-md flex items-start gap-2">
          <AlertCircle className="h-4 w-4 text-destructive mt-0.5 shrink-0" />
          <span className="text-sm text-destructive">{status.parseError}</span>
        </div>
      )}
    </div>
  )
}
