import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Plus, AlertCircle, RefreshCw } from 'lucide-react'
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

export function DshConfigPage() {
  const { t } = useTranslation()
  const providers = useDshStore(state => state.providers)
  const isLoading = useDshStore(state => state.isLoading)
  const error = useDshStore(state => state.error)
  const configStatus = useDshStore(state => state.configStatus)

  const loadProviders = useDshStore(state => state.loadProviders)
  const loadCredentials = useDshStore(state => state.loadCredentials)
  const loadConfigStatus = useDshStore(state => state.loadConfigStatus)
  const deleteProvider = useDshStore(state => state.deleteProvider)
  const setError = useDshStore(state => state.setError)

  const [providerDialogOpen, setProviderDialogOpen] = useState(false)
  const [editingProviderId, setEditingProviderId] = useState<string | null>(
    null
  )
  const [deleteProviderId, setDeleteProviderId] = useState<string | null>(null)

  useEffect(() => {
    loadProviders()
    loadCredentials()
    loadConfigStatus()
  }, [loadProviders, loadCredentials, loadConfigStatus])

  const handleAddProvider = () => {
    setEditingProviderId(null)
    setProviderDialogOpen(true)
  }

  const handleEditProvider = (providerId: string) => {
    setEditingProviderId(providerId)
    setProviderDialogOpen(true)
  }

  const handleConfirmDeleteProvider = async () => {
    if (deleteProviderId) {
      await deleteProvider(deleteProviderId)
      toast.success(t('dsh.provider.deleted'))
      setDeleteProviderId(null)
    }
  }

  const providerEntries = Object.entries(providers)

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between gap-2 p-4 border-b">
        <div className="min-w-0 flex-1">
          <h1 className="text-xl font-semibold">{t('dsh.title')}</h1>
        </div>
        <div className="flex items-center gap-2 shrink-0">
          <Button
            variant="outline"
            size="icon"
            onClick={() => {
              loadProviders()
              loadCredentials()
              loadConfigStatus()
            }}
            disabled={isLoading}
            title={t('common.refresh')}
          >
            <RefreshCw className="h-4 w-4" />
          </Button>
        </div>
      </div>

      {/* Error Alert */}
      {error && (
        <div className="mx-4 mt-4 p-3 bg-destructive/10 border border-destructive/20 rounded-md flex items-center gap-2">
          <AlertCircle className="h-4 w-4 text-destructive" />
          <span className="text-sm text-destructive">{error}</span>
          <Button
            variant="ghost"
            size="sm"
            className="ml-auto"
            onClick={() => setError(null)}
          >
            {t('common.dismiss')}
          </Button>
        </div>
      )}

      {/* Main Content */}
      <div className="flex-1 overflow-auto p-4 space-y-4">
        {/* Providers Section */}
        <div className="space-y-3 p-4 border rounded-lg">
          <div className="flex items-center justify-between">
            <h2 className="text-lg font-medium">{t('dsh.providers.title')}</h2>
            <Button
              variant="outline"
              size="sm"
              onClick={handleAddProvider}
              disabled={isLoading}
            >
              <Plus className="h-4 w-4 mr-2" />
              {t('dsh.provider.add')}
            </Button>
          </div>

          {providerEntries.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground">
              {t('dsh.provider.noProviders')}
            </div>
          ) : (
            <div className="space-y-2">
              {providerEntries.map(([providerId, config]) => (
                <ProviderCard
                  key={providerId}
                  providerId={providerId}
                  config={config ?? undefined}
                  onEdit={() => handleEditProvider(providerId)}
                  onDelete={() => setDeleteProviderId(providerId)}
                />
              ))}
            </div>
          )}
        </div>

        {/* Config Status */}
        <ConfigStatus status={configStatus} />
      </div>

      {/* Provider Dialog */}
      <ProviderDialog
        open={providerDialogOpen}
        onOpenChange={setProviderDialogOpen}
        editingProviderId={editingProviderId}
      />

      {/* Delete Provider Confirmation */}
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
            <AlertDialogAction onClick={handleConfirmDeleteProvider}>
              {t('common.delete')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
