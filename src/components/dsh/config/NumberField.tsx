import { useTranslation } from 'react-i18next'
import { Input } from '@/components/ui/input'
import { cn } from '@/lib/utils'
import { DirtyIndicator } from './DirtyIndicator'
import type { BaseFieldProps } from './field-props'

interface NumberFieldProps extends BaseFieldProps {
  value: number | null
  onChange: (value: number | null) => void
  min?: number
  max?: number
  step?: number
  /** Suffix rendered after the input, such as `ms` or `bytes`. */
  unit?: string
  placeholder?: string
  className?: string
}

/**
 * Number input for `number` config fields. Blank input reports `null` so the
 * caller can fall back to the inherited default.
 */
export function NumberField({
  value,
  onChange,
  min,
  max,
  step,
  unit,
  placeholder,
  disabled,
  dirty,
  id,
  className,
}: NumberFieldProps) {
  const { t } = useTranslation()

  const rangeHint =
    min !== undefined && max !== undefined
      ? t('dsh.field.numberRange', { min, max })
      : min !== undefined
        ? t('dsh.field.numberRangeMin', { min })
        : max !== undefined
          ? t('dsh.field.numberRangeMax', { max })
          : null

  return (
    <div
      className={cn('flex flex-col gap-1', className)}
      data-dirty={dirty ? 'true' : undefined}
    >
      <div className="flex items-center gap-2">
        <Input
          id={id}
          type="number"
          className="w-40"
          value={value ?? ''}
          min={min}
          max={max}
          step={step}
          placeholder={placeholder}
          disabled={disabled}
          onChange={event => {
            const raw = event.target.value
            if (raw === '') {
              onChange(null)
              return
            }
            const parsed = Number(raw)
            onChange(Number.isFinite(parsed) ? parsed : null)
          }}
        />
        {unit ? (
          <span className="text-sm text-muted-foreground">{unit}</span>
        ) : null}
        <DirtyIndicator dirty={dirty} />
      </div>
      {rangeHint ? (
        <span className="text-xs text-muted-foreground">{rangeHint}</span>
      ) : null}
    </div>
  )
}
