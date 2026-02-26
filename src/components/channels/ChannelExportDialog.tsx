import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { Checkbox } from '@/components/ui/checkbox'
import { Label } from '@/components/ui/label'

interface ChannelExportDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onExport: (includeCredentials: boolean) => void
}

export function ChannelExportDialog({
  open,
  onOpenChange,
  onExport,
}: ChannelExportDialogProps) {
  const { t } = useTranslation()
  const [includeCredentials, setIncludeCredentials] = useState(false)

  const handleExport = () => {
    onExport(includeCredentials)
    onOpenChange(false)
    setIncludeCredentials(false)
  }

  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{t('channels.export.title')}</AlertDialogTitle>
          <AlertDialogDescription>
            {t('channels.export.description')}
          </AlertDialogDescription>
        </AlertDialogHeader>

        <div className="flex items-center gap-2 py-2">
          <Checkbox
            id="include-credentials"
            checked={includeCredentials}
            onCheckedChange={checked => setIncludeCredentials(checked === true)}
          />
          <Label htmlFor="include-credentials" className="text-sm">
            {t('channels.export.includeCredentials')}
          </Label>
        </div>
        {includeCredentials && (
          <p className="text-xs text-yellow-600 dark:text-yellow-400">
            {t('channels.export.credentialsWarning')}
          </p>
        )}

        <AlertDialogFooter>
          <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
          <AlertDialogAction onClick={handleExport}>
            {t('common.export')}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
