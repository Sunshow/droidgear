import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { Check, LoaderCircle, RotateCcw, Save } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

interface SettingsPageHeaderProps {
  title: ReactNode
  description?: ReactNode
  /** Whether the page has staged edits that a save would write. */
  dirty?: boolean
  /** Whether a save is crossing the wire. */
  saving?: boolean
  /** Shows a short confirmation after a successful save. */
  saved?: boolean
  onSave: () => void
  onReset: () => void
  /** Disables both actions, such as for a read-only deployment. */
  disabled?: boolean
  /** Extra controls rendered before the reset/save buttons. */
  children?: ReactNode
  className?: string
}

/** Page header for a settings surface with reset and save actions. */
export function SettingsPageHeader({
  title,
  description,
  dirty,
  saving,
  saved,
  onSave,
  onReset,
  disabled,
  children,
  className,
}: SettingsPageHeaderProps) {
  const { t } = useTranslation()
  const busy = saving === true
  const actionsBlocked = disabled === true || busy || dirty !== true

  return (
    <header
      className={cn(
        'flex items-start justify-between gap-3 border-b p-4',
        className
      )}
    >
      <div className="min-w-0 flex-1 space-y-1">
        <h2 className="text-xl font-semibold">{title}</h2>
        {description ? (
          <p className="text-sm text-muted-foreground">{description}</p>
        ) : null}
        {saved && dirty !== true && !busy ? (
          <p className="flex items-center gap-1 text-xs text-green-600 dark:text-green-400">
            <Check className="size-3.5" aria-hidden="true" />
            {t('dsh.config.saved')}
          </p>
        ) : null}
      </div>
      <div className="flex shrink-0 items-center gap-2">
        {children}
        <Button
          type="button"
          variant="outline"
          size="sm"
          disabled={actionsBlocked}
          onClick={onReset}
        >
          <RotateCcw aria-hidden="true" />
          {t('dsh.config.reset')}
        </Button>
        <Button
          type="button"
          size="sm"
          disabled={actionsBlocked}
          onClick={onSave}
        >
          {busy ? (
            <LoaderCircle
              data-icon="inline-start"
              className="animate-spin"
              aria-hidden="true"
            />
          ) : (
            <Save aria-hidden="true" />
          )}
          {busy ? t('dsh.config.saving') : t('dsh.config.save')}
        </Button>
      </div>
    </header>
  )
}
