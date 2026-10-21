import { Switch } from '@/components/ui/switch'
import { cn } from '@/lib/utils'
import { DirtyIndicator } from './DirtyIndicator'
import type { BaseFieldProps } from './field-props'

interface SwitchFieldProps extends BaseFieldProps {
  value: boolean
  onChange: (value: boolean) => void
  className?: string
}

/** Toggle for `boolean` config fields. */
export function SwitchField({
  value,
  onChange,
  disabled,
  dirty,
  id,
  className,
}: SwitchFieldProps) {
  return (
    <div
      className={cn('flex items-center gap-2', className)}
      data-dirty={dirty ? 'true' : undefined}
    >
      <Switch
        id={id}
        checked={value}
        onCheckedChange={onChange}
        disabled={disabled}
      />
      <DirtyIndicator dirty={dirty} />
    </div>
  )
}
