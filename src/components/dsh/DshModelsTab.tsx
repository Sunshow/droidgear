import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Plus, Star } from 'lucide-react'
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
import { DshOfficialDeepseekCard } from './DshOfficialDeepseekCard'
import { ProviderCard } from './ProviderCard'
import { ProviderDialog } from './ProviderDialog'
import { isProviderReasoningLevel, readDefaultModel } from './provider-defaults'

/** Models tab: provider CRUD, the current default model, and config status. */
export function DshModelsTab() {
  const { t } = useTranslation()
  const providers = useDshStore(state => state.providers)
  const isLoading = useDshStore(state => state.isLoading)
  const configStatus = useDshStore(state => state.configStatus)
  const deleteProvider = useDshStore(state => state.deleteProvider)
  const snapshot = useDshStore(state => state.snapshot)

  const [providerDialogOpen, setProviderDialogOpen] = useState(false)
  const [editingProviderId, setEditingProviderId] = useState<string | null>(
    null
  )
  const [deleteProviderId, setDeleteProviderId] = useState<string | null>(null)

  const defaultModel = readDefaultModel(snapshot)
  const defaultEffort =
    defaultModel?.reasoningEffort &&
    isProviderReasoningLevel(defaultModel.reasoningEffort)
      ? t(`dsh.provider.reasoningLevel.${defaultModel.reasoningEffort}`)
      : null

  const handleConfirmDeleteProvider = async () => {
    if (!deleteProviderId) return
    await deleteProvider(deleteProviderId)
    toast.success(t('dsh.provider.deleted'))
    setDeleteProviderId(null)
  }

  const providerEntries = Object.entries(providers)

  return (
    <div className="space-y-4">
      <DshOfficialDeepseekCard />

      <div className="flex flex-wrap items-center gap-2 rounded-lg border bg-muted/30 p-3">
        <Star className="size-4 shrink-0 text-amber-500" aria-hidden="true" />
        {defaultModel ? (
          <span className="text-sm">
            {defaultEffort
              ? t('dsh.models.currentDefaultWithEffort', {
                  provider: defaultModel.provider,
                  model: defaultModel.model,
                  effort: defaultEffort,
                })
              : t('dsh.models.currentDefault', {
                  provider: defaultModel.provider,
                  model: defaultModel.model,
                })}
          </span>
        ) : (
          <span className="text-sm text-muted-foreground">
            {t('dsh.models.noDefault')}
          </span>
        )}
        <div className="ms-auto flex items-center gap-2">
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
              defaultModel={
                defaultModel?.provider === providerId
                  ? {
                      model: defaultModel.model,
                      reasoningEffort: defaultModel.reasoningEffort,
                    }
                  : null
              }
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
