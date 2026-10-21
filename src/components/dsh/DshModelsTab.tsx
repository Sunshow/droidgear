import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Plus } from 'lucide-react'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
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
import { useDshStore } from '@/store/dsh-store'
import { ConfigStatus } from './ConfigStatus'
import { ProviderCard } from './ProviderCard'
import { ProviderDialog } from './ProviderDialog'

/** Models tab: the existing provider CRUD plus config status. */
export function DshModelsTab() {
  const { t } = useTranslation()
  const providers = useDshStore(state => state.providers)
  const isLoading = useDshStore(state => state.isLoading)
  const configStatus = useDshStore(state => state.configStatus)
  const deleteProvider = useDshStore(state => state.deleteProvider)

  const [providerDialogOpen, setProviderDialogOpen] = useState(false)
  const [editingProviderId, setEditingProviderId] = useState<string | null>(
    null
  )
  const [deleteProviderId, setDeleteProviderId] = useState<string | null>(null)

  const handleConfirmDeleteProvider = async () => {
    if (!deleteProviderId) return
    await deleteProvider(deleteProviderId)
    toast.success(t('dsh.provider.deleted'))
    setDeleteProviderId(null)
  }

  const providerEntries = Object.entries(providers)

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-end gap-2">
        <Button
          variant="outline"
          size="sm"
          onClick={() => {
            setEditingProviderId(null)
            setProviderDialogOpen(true)
          }}
          disabled={isLoading}
        >
          <Plus aria-hidden="true" />
          {t('dsh.provider.add')}
        </Button>
      </div>

      {providerEntries.length === 0 ? (
        <p className="py-8 text-center text-sm text-muted-foreground">
          {t('dsh.provider.noProviders')}
        </p>
      ) : (
        <div className="space-y-2">
          {providerEntries.map(([providerId, config]) => (
            <ProviderCard
              key={providerId}
              providerId={providerId}
              config={config ?? undefined}
              onEdit={() => {
                setEditingProviderId(providerId)
                setProviderDialogOpen(true)
              }}
              onDelete={() => setDeleteProviderId(providerId)}
            />
          ))}
        </div>
      )}

      <ConfigStatus status={configStatus} />

      <ProviderDialog
        open={providerDialogOpen}
        onOpenChange={setProviderDialogOpen}
        editingProviderId={editingProviderId}
      />

      <AlertDialog
        open={deleteProviderId !== null}
        onOpenChange={() => setDeleteProviderId(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('dsh.provider.delete')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('dsh.provider.deleteConfirm')}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                void handleConfirmDeleteProvider()
              }}
            >
              {t('common.delete')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
