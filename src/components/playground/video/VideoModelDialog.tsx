import { useTranslation } from 'react-i18next'
import { parseList } from '../shared/format'
import { ModelFormDialog } from '../shared/model-form-dialog'
import type {
  VideoModel,
  VideoModelDraft,
  VideoProtocol,
} from '@/store/playground-store'
import {
  buildImportedVideoModels,
  videoProtocolDefaults,
  videoProtocolOptions,
} from './video-protocol'

interface VideoModelDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  /** Present when editing; absent when adding */
  model?: VideoModel
  onSave: (draft: VideoModelDraft) => unknown
  onSaveMany: (drafts: VideoModelDraft[]) => unknown
}

export function VideoModelDialog({
  open,
  onOpenChange,
  model,
  onSave,
  onSaveMany,
}: VideoModelDialogProps) {
  const { t } = useTranslation()
  const defaults = videoProtocolDefaults(model?.protocol)

  return (
    <ModelFormDialog<VideoModelDraft>
      open={open}
      onOpenChange={onOpenChange}
      editing={model !== undefined}
      formKey={model?.id ?? 'new'}
      title={
        model
          ? t('playground.video.modelDialog.editTitle')
          : t('playground.video.modelDialog.addTitle')
      }
      description={t('playground.video.modelDialog.description')}
      protocolLabel={t('playground.video.modelDialog.protocol')}
      protocolOptions={videoProtocolOptions(t)}
      defaultProtocol="sora"
      protocolHint={t('playground.video.modelDialog.protocolHint')}
      optionFields={[
        {
          key: 'durations',
          label: t('playground.video.modelDialog.durations'),
          placeholder: defaults.durations.join(', '),
          // A model saved with empty lists gets the shared suggestions back,
          // matching what the create form's picker offers.
          initial: (model?.durations.length
            ? model.durations
            : defaults.durations
          ).join(', '),
        },
        {
          key: 'resolutions',
          label: t('playground.video.modelDialog.resolutions'),
          placeholder: defaults.resolutions.join(', '),
          initial: (model?.resolutions.length
            ? model.resolutions
            : defaults.resolutions
          ).join(', '),
        },
      ]}
      optionsHint={t('playground.video.modelDialog.optionsHint')}
      supportsImageInputLabel={t(
        'playground.video.modelDialog.supportsImageInput'
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
        title: t('playground.video.modelDialog.importTitle'),
        hint: t('playground.video.modelDialog.importHint'),
        action: t('playground.video.modelDialog.importAction'),
        dialogTitle: t('playground.video.import.title'),
        dialogDescription: t('playground.video.import.description'),
      }}
      importProtocolLabel={t('playground.video.import.protocol')}
      importProtocolHint={t('playground.video.import.protocolHint')}
      importProtocolOptions={videoProtocolOptions(t)}
      defaultImportProtocol="sora"
      buildImportedDrafts={selection =>
        buildImportedVideoModels({
          ...selection,
          protocols: selection.protocols as ReadonlyMap<string, VideoProtocol>,
        })
      }
      onSubmit={values =>
        onSave({
          name: values.name,
          modelId: values.modelId,
          protocol: values.protocol as VideoProtocol,
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
          durations: parseList(values.optionTexts.durations ?? '')
            .map(Number)
            .filter(number => Number.isFinite(number) && number > 0),
          resolutions: parseList(values.optionTexts.resolutions ?? ''),
          supportsImageInput: values.supportsImageInput,
        })
      }
      onImport={onSaveMany}
    />
  )
}
