import type { ReactNode } from 'react'
import { Label } from '@/components/ui/label'
import { cn } from '@/lib/utils'

interface SettingsRowProps {
  label: ReactNode
  description?: ReactNode
  /** Badges rendered next to the label. */
  badges?: ReactNode
  /** Control rendered beside (or below) the label. */
  control?: ReactNode
  /** Shorthand for `control`. */
  children?: ReactNode
  /**
   * `horizontal` places the control beside the label; `vertical` stacks it
   * under the label and spans the full row width.
   */
  layout?: 'horizontal' | 'vertical'
  /** Associates the label with the control's DOM id. */
  htmlFor?: string
  className?: string
}

/** One labelled settings row with optional description and badges. */
export function SettingsRow({
  label,
  description,
  badges,
  control,
  children,
  layout = 'horizontal',
  htmlFor,
  className,
}: SettingsRowProps) {
  const content = control ?? children

  return (
    <div
      data-layout={layout}
      className={cn(
        'flex gap-3',
        layout === 'horizontal'
          ? 'items-center justify-between'
          : 'flex-col items-stretch',
        className
      )}
    >
      <div className="min-w-0 space-y-1">
        <div className="flex flex-wrap items-center gap-1.5">
          <Label htmlFor={htmlFor}>{label}</Label>
          {badges}
        </div>
        {description ? (
          <p className="text-xs text-muted-foreground">{description}</p>
        ) : null}
      </div>
      {content ? (
        <div
          className={cn(
            layout === 'horizontal' ? 'shrink-0' : 'w-full min-w-0'
          )}
        >
          {content}
        </div>
      ) : null}
    </div>
  )
}
