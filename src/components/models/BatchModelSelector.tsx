import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { ChevronDown, ChevronRight, Search } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import type { CustomModel, Provider, ModelInfo } from '@/lib/bindings'
import {
  containsRegexSpecialChars,
  getDefaultMaxOutputTokens,
  hasOfficialModelNamePrefix,
} from '@/lib/utils'
import type { BatchModelConfig } from '@/lib/batch-model-utils'

interface BatchModelSelectorProps {
  models: ModelInfo[]
  apiKey: string
  existingModels: CustomModel[]
  defaultProvider: Provider
  inferProvider?: (modelId: string) => Provider

  prefix: string
  suffix: string
  batchMaxTokens: string
  batchSupportsImages: boolean

  selectedModels: Map<string, BatchModelConfig>

  onPrefixChange: (v: string) => void
  onSuffixChange: (v: string) => void
  onBatchMaxTokensChange: (v: string) => void
  onBatchSupportsImagesChange: (v: boolean) => void
  onToggleModel: (modelId: string) => void
  onConfigChange: (modelId: string, config: Partial<BatchModelConfig>) => void
  onSelectAll: () => void
  onDeselectAll: () => void
}

export function BatchModelSelector({
  models,
  apiKey,
  existingModels,
  defaultProvider,
  inferProvider,
  prefix,
  suffix,
  batchMaxTokens,
  batchSupportsImages,
  selectedModels,
  onPrefixChange,
  onSuffixChange,
  onBatchMaxTokensChange,
  onBatchSupportsImagesChange,
  onToggleModel,
  onConfigChange,
  onSelectAll,
  onDeselectAll,
}: BatchModelSelectorProps) {
  const { t } = useTranslation()
  const [filterText, setFilterText] = useState('')
  const [expandedModels, setExpandedModels] = useState<Set<string>>(new Set())

  const isModelKeyExisting = (modelId: string) => {
    return existingModels.some(m => m.model === modelId && m.apiKey === apiKey)
  }

  const selectableModels = models.filter(m => !isModelKeyExisting(m.id))

  const filteredModels = filterText
    ? models.filter(m => {
        const searchLower = filterText.toLowerCase()
        return (
          m.id.toLowerCase().includes(searchLower) ||
          m.name?.toLowerCase().includes(searchLower)
        )
      })
    : models

  const toggleExpanded = (modelId: string) => {
    setExpandedModels(prev => {
      const next = new Set(prev)
      if (next.has(modelId)) {
        next.delete(modelId)
      } else {
        next.add(modelId)
      }
      return next
    })
  }

  const handleSelectAllClick = () => {
    if (selectedModels.size === selectableModels.length) {
      onDeselectAll()
    } else {
      onSelectAll()
    }
  }

  return (
    <div className="space-y-4">
      {/* Batch settings row 1: Prefix and Suffix */}
      <div className="grid grid-cols-2 gap-4">
        <div className="space-y-2">
          <Label htmlFor="batch-prefix">{t('models.prefix')}</Label>
          <Input
            id="batch-prefix"
            value={prefix}
            onChange={e => onPrefixChange(e.target.value)}
            placeholder={t('models.prefixPlaceholder')}
          />
          {containsRegexSpecialChars(prefix) && (
            <p className="text-sm text-destructive">
              {t('validation.bracketsNotAllowed')}
            </p>
          )}
        </div>
        <div className="space-y-2">
          <Label htmlFor="batch-suffix">{t('models.suffix')}</Label>
          <Input
            id="batch-suffix"
            value={suffix}
            onChange={e => onSuffixChange(e.target.value)}
            placeholder={t('models.suffixPlaceholder')}
          />
          {containsRegexSpecialChars(suffix) && (
            <p className="text-sm text-destructive">
              {t('validation.bracketsNotAllowed')}
            </p>
          )}
        </div>
      </div>

      {/* Batch settings row 2: Max Tokens and Supports Images */}
      <div className="grid grid-cols-2 gap-4">
        <div className="space-y-2">
          <Label htmlFor="batch-max-tokens">{t('models.batchMaxTokens')}</Label>
          <Input
            id="batch-max-tokens"
            type="number"
            value={batchMaxTokens}
            onChange={e => onBatchMaxTokensChange(e.target.value)}
            placeholder={t('models.maxTokensPlaceholder')}
            step={8192}
          />
        </div>
        <div className="flex items-end gap-2 pb-2">
          <Checkbox
            id="batch-supports-images"
            checked={batchSupportsImages}
            onCheckedChange={checked =>
              onBatchSupportsImagesChange(checked === true)
            }
          />
          <Label htmlFor="batch-supports-images">
            {t('models.batchSupportsImages')}
          </Label>
        </div>
      </div>

      {/* Filter input */}
      <div className="relative">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
        <Input
          value={filterText}
          onChange={e => setFilterText(e.target.value)}
          placeholder={t('models.filterModels')}
          className="pl-9"
        />
      </div>

      {/* Select header */}
      <div className="flex items-center justify-between">
        <Label>
          {t('models.selectModelsToAdd', {
            count: selectedModels.size,
          })}
        </Label>
        <Button variant="ghost" size="sm" onClick={handleSelectAllClick}>
          {selectedModels.size === selectableModels.length
            ? t('common.deselectAll')
            : t('common.selectAll')}
        </Button>
      </div>

      {/* Model list */}
      <div className="flex-1 border rounded-md p-2 overflow-auto max-h-[240px]">
        <div className="space-y-1">
          {filteredModels.map(m => {
            const isExisting = isModelKeyExisting(m.id)
            const isSelected = selectedModels.has(m.id)
            const modelConfig = selectedModels.get(m.id)
            const isExpanded = expandedModels.has(m.id)
            const provider = inferProvider
              ? inferProvider(m.id)
              : defaultProvider

            return (
              <div key={m.id} className="rounded hover:bg-accent/50">
                {/* Main row */}
                <div className="flex items-center gap-2 p-2">
                  <Checkbox
                    id={`batch-model-${m.id}`}
                    checked={isSelected}
                    onCheckedChange={() => onToggleModel(m.id)}
                    disabled={isExisting}
                  />
                  <label
                    htmlFor={`batch-model-${m.id}`}
                    className="text-sm cursor-pointer min-w-[100px] shrink-0"
                  >
                    {m.name || m.id}
                    {isExisting && (
                      <span className="ml-2 text-xs text-muted-foreground">
                        {t('models.alreadyAddedForKey')}
                      </span>
                    )}
                  </label>
                  {isSelected && (
                    <>
                      <Select
                        value={modelConfig?.provider ?? provider}
                        onValueChange={(value: Provider) =>
                          onConfigChange(m.id, { provider: value })
                        }
                      >
                        <SelectTrigger className="h-7 w-[120px] text-sm shrink-0">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="anthropic">
                            {t('models.providerAnthropic')}
                          </SelectItem>
                          <SelectItem value="openai">
                            {t('models.providerOpenAI')}
                          </SelectItem>
                          <SelectItem value="generic-chat-completion-api">
                            {t('models.providerGeneric')}
                          </SelectItem>
                        </SelectContent>
                      </Select>
                      <div className="flex-1">
                        <Input
                          className="h-7 text-sm"
                          value={modelConfig?.alias ?? ''}
                          onChange={e =>
                            onConfigChange(m.id, { alias: e.target.value })
                          }
                          placeholder={t('models.aliasPlaceholder')}
                        />
                      </div>
                      <Button
                        variant="ghost"
                        size="sm"
                        className="h-7 w-7 p-0"
                        onClick={() => toggleExpanded(m.id)}
                      >
                        {isExpanded ? (
                          <ChevronDown className="h-4 w-4" />
                        ) : (
                          <ChevronRight className="h-4 w-4" />
                        )}
                      </Button>
                    </>
                  )}
                </div>

                {/* Validation errors for alias */}
                {isSelected && modelConfig?.alias && (
                  <div className="pl-8 pb-1">
                    {containsRegexSpecialChars(modelConfig.alias) && (
                      <p className="text-xs text-destructive">
                        {t('validation.bracketsNotAllowed')}
                      </p>
                    )}
                    {hasOfficialModelNamePrefix(modelConfig.alias) && (
                      <p className="text-xs text-destructive">
                        {t('validation.officialModelNameNotAllowed')}
                      </p>
                    )}
                  </div>
                )}

                {/* Expanded settings */}
                {isSelected && isExpanded && (
                  <div className="pl-8 pr-2 pb-2 flex items-center gap-4">
                    <div className="flex items-center gap-2">
                      <Label
                        htmlFor={`max-tokens-${m.id}`}
                        className="text-xs text-muted-foreground whitespace-nowrap"
                      >
                        {t('models.maxTokens')}:
                      </Label>
                      <Input
                        id={`max-tokens-${m.id}`}
                        type="number"
                        className="h-7 w-24 text-sm"
                        value={modelConfig?.maxTokens ?? ''}
                        onChange={e =>
                          onConfigChange(m.id, {
                            maxTokens: e.target.value
                              ? parseInt(e.target.value)
                              : undefined,
                          })
                        }
                        placeholder={String(getDefaultMaxOutputTokens(m.id))}
                        step={8192}
                      />
                    </div>
                    <div className="flex items-center gap-2">
                      <Checkbox
                        id={`supports-images-${m.id}`}
                        checked={modelConfig?.supportsImages ?? false}
                        onCheckedChange={checked =>
                          onConfigChange(m.id, {
                            supportsImages: checked === true ? true : undefined,
                          })
                        }
                      />
                      <Label
                        htmlFor={`supports-images-${m.id}`}
                        className="text-xs text-muted-foreground"
                      >
                        {t('models.supportsImages')}
                      </Label>
                    </div>
                  </div>
                )}
              </div>
            )
          })}
        </div>
      </div>
    </div>
  )
}
