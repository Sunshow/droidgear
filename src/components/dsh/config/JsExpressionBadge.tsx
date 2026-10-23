import { useTranslation } from 'react-i18next'
import { Braces } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { cn } from '@/lib/utils'

interface JsExpressionBadgeProps {
  className?: string
}

/** Marks a config field holding a `!!js` expression, which stays read-only. */
export function JsExpressionBadge({ className }: JsExpressionBadgeProps) {
  const { t } = useTranslation()

  return (
    <Badge
      variant="secondary"
      className={cn('text-muted-foreground', className)}
    >
      <Braces aria-hidden="true" />
      {t('dsh.config.jsReadonly')}
    </Badge>
  )
}
