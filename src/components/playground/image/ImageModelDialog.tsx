import { useTranslation } from 'react-i18next'
import { parseList } from '../shared/format'
import { ModelFormDialog } from '../shared/model-form-dialog'
import type {
  ImageModel,
  ImageModelDraft,
  ImageProtocol,
} from '@/store/image-playground-store'
import {
  buildImportedImageModels,
  imageProtocolDefaults,
  imageProtocolOptions,
} from './image-protocol'

interface ImageModelDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  /** Present when editing; absent when adding */
  model?: ImageModel
  onSave: (draft: ImageModelDraft) => unknown
  onSaveMany: (drafts: ImageModelDraft[]) => unknown
}

export function ImageModelDialog({
  open,
  onOpenChange,
  model,
  onSave,
  onSaveMany,
}: ImageModelDialogProps) {
  const { t } = useTranslation()
  const defaults = imageProtocolDefaults(model?.protocol)

  return (
    <ModelFormDialog<ImageModelDraft>
      open={open}
      onOpenChange={onOpenChange}
      editing={model !== undefined}
      formKey={model?.id ?? 'new'}
      title={
        model
          ? t('playground.image.modelDialog.editTitle')
          : t('playground.image.modelDialog.addTitle')
      }
      description={t('playground.image.modelDialog.description')}
      protocolLabel={t('playground.image.modelDialog.protocol')}
      protocolOptions={imageProtocolOptions(t)}
      defaultProtocol="chat"
      protocolHint={t('playground.image.modelDialog.protocolHint')}
      optionFields={[
        {
          key: 'sizes',
          label: t('playground.image.modelDialog.sizes'),
          placeholder: defaults.sizes.join(', '),
          initial: (model?.sizes ?? defaults.sizes).join(', '),
        },
        {
          key: 'qualities',
          label: t('playground.image.modelDialog.qualities'),
          placeholder:
            defaults.qualities.length > 0
              ? defaults.qualities.join(', ')
              : t('playground.image.modelDialog.qualitiesOptional'),
          initial: (model?.qualities ?? defaults.qualities).join(', '),
        },
      ]}
      optionDefaultsForProtocol={protocol => {
        const next = imageProtocolDefaults(protocol as ImageProtocol)
        return {
          sizes: next.sizes.join(', '),
          qualities: next.qualities.join(', '),
        }
      }}
      optionsHint={t('playground.image.modelDialog.optionsHint')}
      supportsImageInputLabel={t(
        'playground.image.modelDialog.supportsImageInput'
      )}
      initial={
        model
          ? {
              name: model.name,
              modelId: model.modelId,
              protocol: model.protocol,
              sourceKind: model.source.kind,
              channelId:
                model.source.kind === 'channel' ? model.source.channelId : '',
              tokenId:
                model.source.kind === 'channel' ? model.source.tokenId : null,
              baseUrl:
                model.source.kind === 'custom' ? model.source.baseUrl : '',
              apiKey: model.source.kind === 'custom' ? model.source.apiKey : '',
              supportsImageInput: model.supportsImageInput,
            }
          : undefined
      }
      importCopy={{
        title: t('playground.image.modelDialog.importTitle'),
        hint: t('playground.image.modelDialog.importHint'),
        action: t('playground.image.modelDialog.importAction'),
        dialogTitle: t('playground.image.import.title'),
        dialogDescription: t('playground.image.import.description'),
      }}
      importProtocolLabel={t('playground.image.import.protocol')}
      importProtocolHint={t('playground.image.import.protocolHint')}
      importProtocolOptions={imageProtocolOptions(t)}
      defaultImportProtocol="chat"
      buildImportedDrafts={selection =>
        buildImportedImageModels({
          ...selection,
          protocols: selection.protocols as ReadonlyMap<string, ImageProtocol>,
        })
      }
      onSubmit={values =>
        onSave({
          name: values.name,
          modelId: values.modelId,
          protocol: values.protocol as ImageProtocol,
          source:
            values.sourceKind === 'channel'
              ? {
                  kind: 'channel',
                  channelId: values.channelId,
                  tokenId: values.tokenId,
                }
              : {
                  kind: 'custom',
                  baseUrl: values.baseUrl,
                  apiKey: values.apiKey,
                },
          sizes: parseList(values.optionTexts.sizes ?? ''),
          qualities: parseList(values.optionTexts.qualities ?? ''),
          supportsImageInput: values.supportsImageInput,
        })
      }
      onImport={onSaveMany}
    />
  )
}
