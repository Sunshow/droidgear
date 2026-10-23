import { useTranslation } from 'react-i18next'
import { RotateCcw } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { cn } from '@/lib/utils'

interface RestartBadgeProps {
  className?: string
}

/** Marks a config field that only takes effect after DSH restarts. */
export function RestartBadge({ className }: RestartBadgeProps) {
  const { t } = useTranslation()

  return (
    <Badge
      variant="outline"
      className={cn('text-amber-600 dark:text-amber-400', className)}
    >
      <RotateCcw aria-hidden="true" />
      {t('dsh.config.restartHint')}
    </Badge>
  )
}
