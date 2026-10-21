import { useTranslation } from 'react-i18next'
import { Layers } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { cn } from '@/lib/utils'

interface OverrideBadgeProps {
  className?: string
}

/** Marks a config value that overrides the inherited layer value. */
export function OverrideBadge({ className }: OverrideBadgeProps) {
  const { t } = useTranslation()

  return (
    <Badge variant="outline" className={cn('text-muted-foreground', className)}>
      <Layers aria-hidden="true" />
      {t('dsh.config.overridden')}
    </Badge>
  )
}
