import { useState, useEffect } from 'react'
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
import { commands } from '@/lib/tauri-bindings'
import { logger } from '@/lib/logger'

export function LegacyConfigDialog() {
  const [open, setOpen] = useState(false)
  const { t } = useTranslation()

  useEffect(() => {
    commands.checkLegacyConfig().then(result => {
      if (result.status === 'ok' && result.data) {
        setOpen(true)
      }
    })
  }, [])

  const handleDelete = async () => {
    const result = await commands.deleteLegacyConfig()
    if (result.status === 'ok') {
      logger.info('Legacy config.json deleted')
    } else {
      logger.error('Failed to delete legacy config', { error: result.error })
    }
    setOpen(false)
  }

  return (
    <AlertDialog open={open} onOpenChange={setOpen}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>
            {t('legacyConfig.title', 'Legacy Configuration Found')}
          </AlertDialogTitle>
          <AlertDialogDescription>
            {t(
              'legacyConfig.description',
              'The old config.json file is deprecated and is no longer used. Would you like to delete it?'
            )}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>
            {t('legacyConfig.keep', 'Keep')}
          </AlertDialogCancel>
          <AlertDialogAction onClick={handleDelete}>
            {t('legacyConfig.delete', 'Delete')}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
