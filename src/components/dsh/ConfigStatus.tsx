import { useTranslation } from 'react-i18next'
import { CheckCircle, XCircle, FileText } from 'lucide-react'
import type { DshConfigStatus } from '@/lib/bindings'

interface ConfigStatusProps {
  status: DshConfigStatus | null
}

export function ConfigStatus({ status }: ConfigStatusProps) {
  const { t } = useTranslation()

  if (!status) return null

  const rows = [
    {
      label: t('dsh.configStatus.settings'),
      path: status.configPath,
      exists: status.configExists,
    },
    {
      label: t('dsh.configStatus.credentials'),
      path: status.credentialsPath,
      exists: status.credentialsExists,
    },
  ]

  return (
    <div className="p-4 border rounded-lg space-y-2">
      <h3 className="text-sm font-medium text-muted-foreground">
        {t('dsh.configStatus.title')}
      </h3>
      <div className="space-y-1 text-sm">
        {rows.map(row => (
          <div key={row.label} className="flex items-center gap-2">
            <FileText className="h-4 w-4 text-muted-foreground shrink-0" />
            <span className="text-xs text-muted-foreground shrink-0 w-24">
              {row.label}
            </span>
            <code className="flex-1 truncate text-xs bg-muted px-1 py-0.5 rounded select-all cursor-text">
              {row.path}
            </code>
            {row.exists ? (
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
        ))}
      </div>
    </div>
  )
}
