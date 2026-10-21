import { useTranslation } from 'react-i18next'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { cn } from '@/lib/utils'
import { DirtyIndicator } from './DirtyIndicator'
import type { BaseFieldProps } from './field-props'

/** One selectable value in an {@link EnumField}. */
export interface EnumOption {
  value: string
  label: string
}

interface EnumFieldProps extends BaseFieldProps {
  value: string | null
  onChange: (value: string) => void
  options: EnumOption[]
  placeholder?: string
  className?: string
}

/**
 * Select control for `enum` config fields. A null value renders the
 * placeholder instead of a selection.
 */
export function EnumField({
  value,
  onChange,
  options,
  placeholder,
  disabled,
  dirty,
  id,
  className,
}: EnumFieldProps) {
  const { t } = useTranslation()

  return (
    <div
      className={cn('flex items-center gap-2', className)}
      data-dirty={dirty ? 'true' : undefined}
    >
      <Select
        value={value ?? undefined}
        onValueChange={onChange}
        disabled={disabled}
      >
        <SelectTrigger id={id} className="min-w-40">
          <SelectValue placeholder={placeholder ?? t('dsh.field.enumEmpty')} />
        </SelectTrigger>
        <SelectContent>
          {options.map(option => (
            <SelectItem key={option.value} value={option.value}>
              {option.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      <DirtyIndicator dirty={dirty} />
    </div>
  )
}
