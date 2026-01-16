import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Plus, X, Terminal as TerminalIcon, GitBranch } from 'lucide-react'
import { Button } from '@/components/ui/button'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { cn } from '@/lib/utils'
import type { DerivedTerminal } from '@/store/terminal-store'
import { CreateDerivedDialog } from './CreateDerivedDialog'

interface DerivedTerminalBarProps {
  terminalId: string
  derivedTerminals: DerivedTerminal[]
  selectedDerivedId: string | null
  onSelectDerived: (derivedId: string | null) => void
  onCloseDerived: (derivedId: string) => void
  onCreateDerived: (command?: string, name?: string) => void
}

export function DerivedTerminalBar({
  terminalId: _terminalId,
  derivedTerminals,
  selectedDerivedId,
  onSelectDerived,
  onCloseDerived,
  onCreateDerived,
}: DerivedTerminalBarProps) {
  const { t } = useTranslation()
  const [dialogOpen, setDialogOpen] = useState(false)

  const handleCreate = (command?: string, name?: string) => {
    onCreateDerived(command, name)
    setDialogOpen(false)
  }

  return (
    <div className="flex items-center gap-1 px-2 py-1 border-b bg-muted/30 overflow-x-auto">
      {/* Main terminal tab - always first, with distinct styling */}
      <button
        onMouseDown={e => {
          e.preventDefault()
          onSelectDerived(null)
        }}
        className={cn(
          'flex items-center gap-1.5 px-3 py-1 rounded text-sm whitespace-nowrap transition-colors',
          selectedDerivedId === null
            ? 'bg-primary/10 text-primary border border-primary/30 shadow-sm font-medium'
            : 'text-muted-foreground hover:text-foreground hover:bg-background/50'
        )}
      >
        <TerminalIcon className="h-3.5 w-3.5" />
        <span>{t('droid.terminal.mainTerminal')}</span>
      </button>

      {/* Separator when there are derived terminals */}
      {derivedTerminals.length > 0 && (
        <div className="h-4 w-px bg-border mx-1" />
      )}

      {/* Derived terminal tabs */}
      {derivedTerminals.map(derived => (
        <div
          key={derived.id}
          onMouseDown={e => {
            e.preventDefault()
            onSelectDerived(derived.id)
          }}
          className={cn(
            'group relative flex items-center gap-1.5 px-3 py-1 rounded text-sm whitespace-nowrap transition-colors cursor-pointer',
            selectedDerivedId === derived.id
              ? 'bg-accent text-accent-foreground shadow-sm font-medium'
              : 'text-muted-foreground hover:text-foreground hover:bg-background/50'
          )}
        >
          {derived.hasNotification && (
            <span className="absolute -top-0.5 -right-0.5 h-2 w-2 rounded-full bg-orange-500" />
          )}
          <GitBranch className="h-3 w-3 flex-shrink-0" />
          {derived.command ? (
            <Tooltip>
              <TooltipTrigger asChild>
                <span className="max-w-[120px] truncate">{derived.name}</span>
              </TooltipTrigger>
              <TooltipContent side="bottom">
                <p className="font-mono text-xs">{derived.command}</p>
              </TooltipContent>
            </Tooltip>
          ) : (
            <span className="max-w-[120px] truncate">{derived.name}</span>
          )}
          <button
            onClick={e => {
              e.stopPropagation()
              onCloseDerived(derived.id)
            }}
            className="ml-0.5 opacity-0 group-hover:opacity-100 hover:text-destructive transition-opacity"
          >
            <X className="h-3 w-3" />
          </button>
        </div>
      ))}

      {/* Add derived terminal button */}
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant="ghost"
            size="icon"
            className="h-6 w-6 ml-1"
            onMouseDown={e => {
              e.preventDefault()
              setDialogOpen(true)
            }}
          >
            <Plus className="h-3.5 w-3.5" />
          </Button>
        </TooltipTrigger>
        <TooltipContent>{t('droid.terminal.newDerived')}</TooltipContent>
      </Tooltip>

      <CreateDerivedDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        onConfirm={handleCreate}
      />
    </div>
  )
}
