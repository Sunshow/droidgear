import { useTranslation } from 'react-i18next'
import {
  ResizableDialog,
  ResizableDialogContent,
  ResizableDialogDescription,
  ResizableDialogHeader,
  ResizableDialogBody,
  ResizableDialogTitle,
} from '@/components/ui/resizable-dialog'
import {
  ChannelModelPicker,
  type ChannelProviderContext,
} from './ChannelModelPicker'
import type { CustomModel } from '@/lib/bindings'

interface ChannelModelPickerDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  mode: 'single' | 'multiple'
  existingModels?: CustomModel[]
  onSelect: (models: CustomModel[]) => void
  onSelectWithContext?: (
    models: CustomModel[],
    context: ChannelProviderContext
  ) => void
  showBatchConfig?: boolean
  platformFilter?: (platform: string | null) => boolean
}

export function ChannelModelPickerDialog({
  open,
  onOpenChange,
  mode,
  existingModels,
  onSelect,
  onSelectWithContext,
  showBatchConfig = false,
  platformFilter,
}: ChannelModelPickerDialogProps) {
  const { t } = useTranslation()

  const handleSelect = (models: CustomModel[]) => {
    onSelect(models)
    onOpenChange(false)
  }

  const handleSelectWithContext = onSelectWithContext
    ? (models: CustomModel[], context: ChannelProviderContext) => {
        onSelectWithContext(models, context)
        onOpenChange(false)
      }
    : undefined

  return (
    <ResizableDialog open={open} onOpenChange={onOpenChange}>
      <ResizableDialogContent
        defaultWidth={600}
        defaultHeight={mode === 'multiple' && showBatchConfig ? 700 : 550}
        minWidth={500}
        minHeight={400}
      >
        <ResizableDialogHeader>
          <ResizableDialogTitle>
            {t('channels.importFromChannel')}
          </ResizableDialogTitle>
          <ResizableDialogDescription>
            {t('channels.importFromChannelDescription')}
          </ResizableDialogDescription>
        </ResizableDialogHeader>

        <ResizableDialogBody>
          {open && (
            <ChannelModelPicker
              mode={mode}
              existingModels={existingModels}
              onSelect={handleSelect}
              onSelectWithContext={handleSelectWithContext}
              showBatchConfig={showBatchConfig}
              platformFilter={platformFilter}
            />
          )}
        </ResizableDialogBody>
      </ResizableDialogContent>
    </ResizableDialog>
  )
}
