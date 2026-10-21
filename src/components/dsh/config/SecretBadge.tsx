import { useTranslation } from 'react-i18next'
import { KeyRound } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { cn } from '@/lib/utils'

interface SecretBadgeProps {
  className?: string
}

/** Marks a config field whose value is a credential and stays read-only. */
export function SecretBadge({ className }: SecretBadgeProps) {
  const { t } = useTranslation()

  return (
    <Badge
      variant="secondary"
      className={cn('text-muted-foreground', className)}
    >
      <KeyRound aria-hidden="true" />
      {t('dsh.config.secretReadonly')}
    </Badge>
  )
}
