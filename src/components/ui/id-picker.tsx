import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Check, ChevronsUpDown } from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/ui/command'
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover'

/**
 * IdPicker is the dropdown half of a "pick or type" id field: the caller keeps
 * its Input for free text (proxy providers, not-yet-applied values), and this
 * button lists the ids the current profile / live config already know.
 *
 * `label` doubles as the accessible name and the search placeholder, so give it
 * something unique per field — tests and screen readers match on it.
 */
export function IdPicker({
  label,
  value,
  options,
  onPick,
}: {
  label: string
  value: string
  options: string[]
  onPick: (value: string) => void
}) {
  const { t } = useTranslation()
  const [open, setOpen] = useState(false)
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          type="button"
          variant="outline"
          size="icon"
          aria-label={label}
          title={label}
          disabled={options.length === 0}
        >
          <ChevronsUpDown className="h-4 w-4" />
        </Button>
      </PopoverTrigger>
      <PopoverContent align="end" className="w-64 p-0">
        <Command>
          <CommandInput placeholder={label} />
          <CommandList>
            <CommandEmpty>{t('common.noChoices')}</CommandEmpty>
            <CommandGroup>
              {options.map(option => (
                <CommandItem
                  key={option}
                  value={option}
                  onSelect={() => {
                    onPick(option)
                    setOpen(false)
                  }}
                >
                  <Check
                    className={`mr-2 h-4 w-4 ${option === value ? 'opacity-100' : 'opacity-0'}`}
                  />
                  <span className="font-mono text-sm">{option}</span>
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  )
}
