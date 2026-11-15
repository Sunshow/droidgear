import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Download, Loader2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { SecretInput } from '@/components/ui/secret-input'
import { Switch } from '@/components/ui/switch'
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  ResizableDialog,
  ResizableDialogBody,
  ResizableDialogContent,
  ResizableDialogFooter,
  ResizableDialogHeader,
  ResizableDialogTitle,
} from '@/components/ui/resizable-dialog'
import { useChannelStore } from '@/store/channel-store'
import {
  ModelImportDialog,
  type ModelImportSelection,
  type ProtocolOption,
} from './model-import-dialog'

export type ModelSourceKind = 'channel' | 'custom'

/** Raw form state; each page maps it to its own model draft. */
export interface PlaygroundModelFormValues {
  name: string
  modelId: string
  protocol: string
  sourceKind: ModelSourceKind
  channelId: string
  tokenId: number | null
  baseUrl: string
  apiKey: string
  supportsImageInput: boolean
  /** Raw text of the feature-specific option lists, keyed by field key. */
  optionTexts: Record<string, string>
}

export interface PlaygroundOptionField {
  key: string
  label: string
  placeholder: string
  initial: string
}

export interface ModelFormDialogProps<TDraft> {
  open: boolean
  onOpenChange: (open: boolean) => void
  /** Present when editing; absent when adding. */
  editing: boolean
  /** Remounts the form when the edited model changes. */
  formKey: string
  title: string
  description: string
  protocolLabel: string
  protocolOptions: ProtocolOption[]
  defaultProtocol: string
  protocolHint: string
  optionFields: PlaygroundOptionField[]
  /** Replaces the option fields' text when the protocol changes (add mode). */
  optionDefaultsForProtocol?: (
    protocol: string
  ) => Record<string, string> | undefined
  optionsHint: string
  supportsImageInputLabel: string
  initial?: Partial<PlaygroundModelFormValues>
  importCopy: {
    title: string
    hint: string
    action: string
    dialogTitle: string
    dialogDescription: string
  }
  importProtocolLabel: string
  importProtocolHint: string
  importProtocolOptions: ProtocolOption[]
  defaultImportProtocol: string
  buildImportedDrafts: (selection: ModelImportSelection) => TDraft[]
  onSubmit: (values: PlaygroundModelFormValues) => unknown
  onImport: (drafts: TDraft[]) => unknown
}

function ModelForm<TDraft>({
  editing,
  protocolLabel,
  protocolOptions,
  defaultProtocol,
  protocolHint,
  optionFields,
  optionDefaultsForProtocol,
  optionsHint,
  supportsImageInputLabel,
  initial,
  importCopy,
  importProtocolLabel,
  importProtocolHint,
  importProtocolOptions,
  defaultImportProtocol,
  buildImportedDrafts,
  onSubmit,
  onImport,
  onCancel,
}: Omit<ModelFormDialogProps<TDraft>, 'open' | 'onOpenChange'> & {
  onCancel: () => void
}) {
  const { t } = useTranslation()
  const channels = useChannelStore(state => state.channels)
  const loadChannels = useChannelStore(state => state.loadChannels)

  const [name, setName] = useState(initial?.name ?? '')
  const [modelId, setModelId] = useState(initial?.modelId ?? '')
  const [protocol, setProtocol] = useState(initial?.protocol ?? defaultProtocol)
  const [sourceKind, setSourceKind] = useState<ModelSourceKind>(
    initial?.sourceKind ?? 'channel'
  )
  const [channelId, setChannelId] = useState(initial?.channelId ?? '')
  const [tokenId, setTokenId] = useState<number | null>(
    initial?.tokenId ?? null
  )
  const [baseUrl, setBaseUrl] = useState(initial?.baseUrl ?? '')
  const [apiKey, setApiKey] = useState(initial?.apiKey ?? '')
  const [supportsImageInput, setSupportsImageInput] = useState(
    initial?.supportsImageInput ?? true
  )
  const [optionTexts, setOptionTexts] = useState<Record<string, string>>(() =>
    Object.fromEntries(optionFields.map(field => [field.key, field.initial]))
  )
  const [importOpen, setImportOpen] = useState(false)
  const [isSaving, setIsSaving] = useState(false)

  useEffect(() => {
    if (channels.length === 0) void loadChannels()
  }, [channels.length, loadChannels])

  const enabledChannels = channels.filter(channel => channel.enabled)
  const sourceValid =
    sourceKind === 'channel' ? channelId !== '' : baseUrl.trim() !== ''
  const isValid = name.trim() !== '' && modelId.trim() !== '' && sourceValid

  const handleSave = async () => {
    setIsSaving(true)
    try {
      await onSubmit({
        name: name.trim(),
        modelId: modelId.trim(),
        protocol,
        sourceKind,
        channelId,
        tokenId,
        baseUrl: baseUrl.trim(),
        apiKey: apiKey.trim(),
        supportsImageInput,
        optionTexts,
      })
      onCancel()
    } finally {
      setIsSaving(false)
    }
  }

  return (
    <>
      <ResizableDialogBody>
        <div className="grid gap-4">
          {!editing && (
            <div className="flex items-center justify-between gap-3 rounded-md border bg-muted/40 p-3">
              <div className="min-w-0">
                <p className="text-sm font-medium">{importCopy.title}</p>
                <p className="text-xs text-muted-foreground">
                  {importCopy.hint}
                </p>
              </div>
              <Button
                variant="outline"
                size="sm"
                className="shrink-0"
                onClick={() => setImportOpen(true)}
              >
                <Download />
                {importCopy.action}
              </Button>
            </div>
          )}

          <div className="grid grid-cols-2 gap-3">
            <div className="grid gap-2">
              <Label htmlFor="playground-model-name">
                {t('playground.modelDialog.name')}
              </Label>
              <Input
                id="playground-model-name"
                value={name}
                onChange={event => setName(event.target.value)}
                placeholder={t('playground.modelDialog.namePlaceholder')}
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="playground-model-id">
                {t('playground.modelDialog.modelId')}
              </Label>
              <Input
                id="playground-model-id"
                value={modelId}
                onChange={event => setModelId(event.target.value)}
                placeholder={t('playground.modelDialog.modelIdPlaceholder')}
                className="font-mono"
              />
            </div>
          </div>

          <div className="grid gap-2">
            <Label>{protocolLabel}</Label>
            <Select
              value={protocol}
              onValueChange={value => {
                setProtocol(value)
                if (!editing) {
                  const defaults = optionDefaultsForProtocol?.(value)
                  if (defaults) {
                    setOptionTexts(previous => ({ ...previous, ...defaults }))
                  }
                }
              }}
            >
              <SelectTrigger className="w-full" aria-label={protocolLabel}>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {protocolOptions.map(option => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <p className="text-xs text-muted-foreground">{protocolHint}</p>
          </div>

          <div className="grid gap-2">
            <Label>{t('playground.modelDialog.source')}</Label>
            <ToggleGroup
              type="single"
              variant="outline"
              size="sm"
              value={sourceKind}
              onValueChange={value => {
                if (value) setSourceKind(value as ModelSourceKind)
              }}
            >
              <ToggleGroupItem value="channel" className="px-4">
                {t('playground.modelDialog.sourceChannel')}
              </ToggleGroupItem>
              <ToggleGroupItem value="custom" className="px-4">
                {t('playground.modelDialog.sourceCustom')}
              </ToggleGroupItem>
            </ToggleGroup>
          </div>

          {sourceKind === 'channel' ? (
            <div className="grid gap-2">
              <Label>{t('playground.modelDialog.channel')}</Label>
              <Select
                value={channelId}
                onValueChange={value => {
                  setChannelId(value)
                  setTokenId(null)
                }}
              >
                <SelectTrigger
                  className="w-full"
                  aria-label={t('playground.modelDialog.channel')}
                >
                  <SelectValue
                    placeholder={t('playground.modelDialog.channelPlaceholder')}
                  />
                </SelectTrigger>
                <SelectContent>
                  {enabledChannels.map(channel => (
                    <SelectItem key={channel.id} value={channel.id}>
                      {channel.name}
                      <span className="text-xs text-muted-foreground">
                        {channel.baseUrl}
                      </span>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              {enabledChannels.length === 0 ? (
                <p className="text-xs text-muted-foreground">
                  {t('playground.modelDialog.noChannels')}
                </p>
              ) : channelId ? (
                <p className="text-xs text-muted-foreground">
                  {t('playground.modelDialog.channelTokenHint')}
                </p>
              ) : null}
            </div>
          ) : (
            <div className="grid grid-cols-2 gap-3">
              <div className="grid gap-2">
                <Label htmlFor="playground-model-base-url">
                  {t('playground.modelDialog.baseUrl')}
                </Label>
                <Input
                  id="playground-model-base-url"
                  value={baseUrl}
                  onChange={event => setBaseUrl(event.target.value)}
                  placeholder={t('playground.modelDialog.baseUrlPlaceholder')}
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="playground-model-api-key">
                  {t('playground.modelDialog.apiKey')}
                </Label>
                <SecretInput
                  id="playground-model-api-key"
                  value={apiKey}
                  onChange={event => setApiKey(event.target.value)}
                  placeholder="sk-..."
                />
              </div>
            </div>
          )}

          <div
            className={
              optionFields.length > 1 ? 'grid grid-cols-2 gap-3' : 'grid gap-3'
            }
          >
            {optionFields.map(field => (
              <div key={field.key} className="grid gap-2">
                <Label htmlFor={`playground-model-option-${field.key}`}>
                  {field.label}
                </Label>
                <Input
                  id={`playground-model-option-${field.key}`}
                  value={optionTexts[field.key] ?? ''}
                  onChange={event =>
                    setOptionTexts(previous => ({
                      ...previous,
                      [field.key]: event.target.value,
                    }))
                  }
                  placeholder={field.placeholder}
                />
              </div>
            ))}
          </div>
          <p className="-mt-2 text-xs text-muted-foreground">{optionsHint}</p>

          <div className="flex items-center justify-between">
            <Label htmlFor="playground-model-image-input">
              {supportsImageInputLabel}
            </Label>
            <Switch
              id="playground-model-image-input"
              checked={supportsImageInput}
              onCheckedChange={setSupportsImageInput}
            />
          </div>
        </div>
      </ResizableDialogBody>

      <ResizableDialogFooter>
        <Button variant="outline" onClick={onCancel}>
          {t('common.cancel')}
        </Button>
        <Button
          onClick={() => void handleSave()}
          disabled={!isValid || isSaving}
        >
          {isSaving && <Loader2 className="animate-spin" />}
          {editing ? t('common.save') : t('common.add')}
        </Button>
      </ResizableDialogFooter>

      <ModelImportDialog
        open={importOpen}
        onOpenChange={setImportOpen}
        title={importCopy.dialogTitle}
        description={importCopy.dialogDescription}
        protocolLabel={importProtocolLabel}
        protocolHint={importProtocolHint}
        protocolOptions={importProtocolOptions}
        defaultProtocol={defaultImportProtocol}
        buildDrafts={buildImportedDrafts}
        onImported={async drafts => {
          await onImport(drafts)
          onCancel()
        }}
      />
    </>
  )
}

/** Shared add/edit model dialog. Both Playground pages supply their protocol
 * options and their own option fields. */
export function ModelFormDialog<TDraft>(props: ModelFormDialogProps<TDraft>) {
  return (
    <ResizableDialog open={props.open} onOpenChange={props.onOpenChange}>
      <ResizableDialogContent
        defaultWidth={620}
        defaultHeight={660}
        minWidth={520}
        minHeight={480}
        onCloseAutoFocus={event => event.preventDefault()}
      >
        <ResizableDialogHeader>
          <ResizableDialogTitle>{props.title}</ResizableDialogTitle>
          <p className="text-sm text-muted-foreground">{props.description}</p>
        </ResizableDialogHeader>
        {props.open && (
          <ModelForm
            key={props.formKey}
            {...props}
            onCancel={() => props.onOpenChange(false)}
          />
        )}
      </ResizableDialogContent>
    </ResizableDialog>
  )
}
