import type { ReactNode } from 'react'
import { cn } from '@/lib/utils'

interface SettingsSectionProps {
  title: ReactNode
  description?: ReactNode
  /** Optional control rendered at the end of the section header. */
  action?: ReactNode
  children: ReactNode
  className?: string
}

/** Bordered group of settings rows with a title and optional description. */
export function SettingsSection({
  title,
  description,
  action,
  children,
  className,
}: SettingsSectionProps) {
  return (
    <section className={cn('space-y-4 rounded-lg border p-4', className)}>
      <header className="flex items-start justify-between gap-2">
        <div className="min-w-0 space-y-1">
          <h3 className="text-sm font-medium">{title}</h3>
          {description ? (
            <p className="text-sm text-muted-foreground">{description}</p>
          ) : null}
        </div>
        {action ? <div className="shrink-0">{action}</div> : null}
      </header>
      <div className="space-y-4">{children}</div>
    </section>
  )
}
