import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

interface CreateDerivedDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onConfirm: (command?: string, name?: string) => void
}

export function CreateDerivedDialog({
  open,
  onOpenChange,
  onConfirm,
}: CreateDerivedDialogProps) {
  const { t } = useTranslation()
  const [command, setCommand] = useState('')
  const [name, setName] = useState('')

  const handleConfirm = () => {
    onConfirm(command.trim() || undefined, name.trim() || undefined)
    setCommand('')
    setName('')
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault()
      handleConfirm()
    }
  }

  const handleOpenChange = (newOpen: boolean) => {
    if (!newOpen) {
      setCommand('')
      setName('')
    }
    onOpenChange(newOpen)
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{t('droid.terminal.newDerived')}</DialogTitle>
        </DialogHeader>
        <div className="flex flex-col gap-4 py-4">
          <div className="flex flex-col gap-2">
            <Label htmlFor="command">
              {t('droid.terminal.derivedCommand')}
            </Label>
            <Input
              id="command"
              value={command}
              onChange={e => setCommand(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder={t('droid.terminal.derivedCommandPlaceholder')}
              autoFocus
              className="font-mono"
            />
          </div>
          <div className="flex flex-col gap-2">
            <Label htmlFor="name">{t('droid.terminal.derivedName')}</Label>
            <Input
              id="name"
              value={name}
              onChange={e => setName(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder={t('droid.terminal.derivedNamePlaceholder')}
            />
          </div>
        </div>
        <DialogFooter>
          <Button
            variant="outline"
            onMouseDown={e => {
              e.preventDefault()
              handleOpenChange(false)
            }}
          >
            {t('common.cancel')}
          </Button>
          <Button
            onMouseDown={e => {
              e.preventDefault()
              handleConfirm()
            }}
          >
            {t('common.create')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
