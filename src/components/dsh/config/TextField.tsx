import { Input } from '@/components/ui/input'
import { cn } from '@/lib/utils'
import { DirtyIndicator } from './DirtyIndicator'
import type { BaseFieldProps } from './field-props'

interface TextFieldProps extends BaseFieldProps {
  value: string
  onChange: (value: string) => void
  placeholder?: string
  className?: string
}

/** Text input for `string` config fields. */
export function TextField({
  value,
  onChange,
  placeholder,
  disabled,
  dirty,
  id,
  className,
}: TextFieldProps) {
  return (
    <div
      className={cn('flex items-center gap-2', className)}
      data-dirty={dirty ? 'true' : undefined}
    >
      <Input
        id={id}
        value={value}
        placeholder={placeholder}
        disabled={disabled}
        onChange={event => onChange(event.target.value)}
      />
      <DirtyIndicator dirty={dirty} />
    </div>
  )
}
