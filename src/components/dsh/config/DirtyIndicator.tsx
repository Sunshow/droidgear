import { useTranslation } from 'react-i18next'

interface DirtyIndicatorProps {
  dirty?: boolean
}

/**
 * Dot marking a control whose staged value has not been saved yet.
 * Rendered next to the control so the dirty flag is visible per field.
 */
export function DirtyIndicator({ dirty }: DirtyIndicatorProps) {
  const { t } = useTranslation()

  if (!dirty) return null

  return (
    <span
      role="img"
      aria-label={t('common.unsaved')}
      title={t('common.unsaved')}
      data-slot="dirty-indicator"
      className="size-2 shrink-0 rounded-full bg-amber-500"
    />
  )
}
