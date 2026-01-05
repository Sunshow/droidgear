import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { Loader2, CheckCircle, XCircle } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { useOpenCodeStore } from '@/store/opencode-store'
import {
  commands,
  type OpenCodeProfile,
  type OpenCodeProviderConfig,
} from '@/lib/bindings'

interface ProviderDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  editingProviderId: string | null
  currentProfile: OpenCodeProfile | null
}

export function ProviderDialog({
  open,
  onOpenChange,
  editingProviderId,
  currentProfile,
}: ProviderDialogProps) {
  const { t } = useTranslation()
  const providerTemplates = useOpenCodeStore(state => state.providerTemplates)
  const addProvider = useOpenCodeStore(state => state.addProvider)
  const updateProvider = useOpenCodeStore(state => state.updateProvider)

  const [providerId, setProviderId] = useState('')
  const [npm, setNpm] = useState('')
  const [name, setName] = useState('')
  const [baseUrl, setBaseUrl] = useState('')
  const [apiKey, setApiKey] = useState('')
  const [timeout, setTimeout] = useState('')
  const [isTesting, setIsTesting] = useState(false)
  const [testResult, setTestResult] = useState<'success' | 'error' | null>(null)

  const isEditing = editingProviderId !== null

  useEffect(() => {
    if (open) {
      setTestResult(null)
      if (editingProviderId && currentProfile) {
        const config = currentProfile.providers[editingProviderId]
        const auth = currentProfile.auth[editingProviderId]
        setProviderId(editingProviderId)
        setNpm(config?.npm ?? '')
        setName(config?.name ?? '')
        setBaseUrl(config?.options?.baseUrl ?? '')
        setTimeout(config?.options?.timeout?.toString() ?? '')
        setApiKey(
          auth && typeof auth === 'object' && 'apiKey' in auth
            ? String(auth.apiKey)
            : ''
        )
      } else {
        setProviderId('')
        setNpm('')
        setName('')
        setBaseUrl('')
        setApiKey('')
        setTimeout('')
      }
    }
  }, [open, editingProviderId, currentProfile])

  const handleTemplateSelect = (templateId: string) => {
    const template = providerTemplates.find(t => t.id === templateId)
    if (template) {
      setProviderId(template.id)
      setName(template.name)
      setBaseUrl(template.defaultBaseUrl ?? '')
    }
  }

  const handleTestConnection = async () => {
    if (!providerId || !baseUrl || !apiKey) return
    setIsTesting(true)
    setTestResult(null)
    try {
      const result = await commands.testOpencodeProviderConnection(
        providerId,
        baseUrl,
        apiKey
      )
      if (result.status === 'ok' && result.data) {
        setTestResult('success')
      } else {
        setTestResult('error')
      }
    } catch {
      setTestResult('error')
    } finally {
      setIsTesting(false)
    }
  }

  const handleSave = () => {
    if (!providerId.trim()) return

    const config: OpenCodeProviderConfig = {
      npm: npm.trim() || null,
      name: name.trim() || null,
      options: {
        baseUrl: baseUrl.trim() || null,
        apiKey: null,
        timeout: timeout ? parseInt(timeout, 10) : null,
        headers: null,
      },
    }

    const auth = apiKey.trim() ? { apiKey: apiKey.trim() } : undefined

    if (isEditing) {
      updateProvider(providerId, config, auth)
    } else {
      addProvider(providerId.trim(), config, auth)
    }

    onOpenChange(false)
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>
            {isEditing
              ? t('opencode.provider.edit')
              : t('opencode.provider.add')}
          </DialogTitle>
          <DialogDescription>
            {t('opencode.provider.dialogDescription')}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {/* Template Selection */}
          {!isEditing && providerTemplates.length > 0 && (
            <div className="space-y-2">
              <Label>{t('opencode.provider.template')}</Label>
              <Select onValueChange={handleTemplateSelect}>
                <SelectTrigger>
                  <SelectValue
                    placeholder={t('opencode.provider.selectTemplate')}
                  />
                </SelectTrigger>
                <SelectContent>
                  {providerTemplates.map(template => (
                    <SelectItem key={template.id} value={template.id}>
                      {template.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          )}

          {/* Provider ID */}
          <div className="space-y-2">
            <Label>{t('opencode.provider.id')} *</Label>
            <Input
              value={providerId}
              onChange={e => setProviderId(e.target.value)}
              placeholder="anthropic"
              disabled={isEditing}
            />
          </div>

          {/* Display Name */}
          <div className="space-y-2">
            <Label>{t('opencode.provider.displayName')}</Label>
            <Input
              value={name}
              onChange={e => setName(e.target.value)}
              placeholder="Anthropic"
            />
          </div>

          {/* NPM Package */}
          <div className="space-y-2">
            <Label>{t('opencode.provider.npm')}</Label>
            <Input
              value={npm}
              onChange={e => setNpm(e.target.value)}
              placeholder="@ai-sdk/openai-compatible"
            />
          </div>

          {/* Base URL */}
          <div className="space-y-2">
            <Label>{t('opencode.provider.baseUrl')}</Label>
            <Input
              value={baseUrl}
              onChange={e => setBaseUrl(e.target.value)}
              placeholder="https://api.anthropic.com"
            />
          </div>

          {/* API Key */}
          <div className="space-y-2">
            <Label>{t('opencode.provider.apiKey')}</Label>
            <Input
              type="password"
              value={apiKey}
              onChange={e => setApiKey(e.target.value)}
              placeholder="sk-ant-..."
            />
          </div>

          {/* Timeout */}
          <div className="space-y-2">
            <Label>{t('opencode.provider.timeout')}</Label>
            <Input
              type="number"
              value={timeout}
              onChange={e => setTimeout(e.target.value)}
              placeholder="300000"
            />
          </div>

          {/* Test Connection */}
          <div className="flex items-center gap-2">
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={handleTestConnection}
              disabled={!providerId || !baseUrl || !apiKey || isTesting}
            >
              {isTesting && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              {t('opencode.provider.testConnection')}
            </Button>
            {testResult === 'success' && (
              <span className="flex items-center text-sm text-green-600">
                <CheckCircle className="h-4 w-4 mr-1" />
                {t('opencode.provider.testSuccess')}
              </span>
            )}
            {testResult === 'error' && (
              <span className="flex items-center text-sm text-destructive">
                <XCircle className="h-4 w-4 mr-1" />
                {t('opencode.provider.testFailed')}
              </span>
            )}
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t('common.cancel')}
          </Button>
          <Button onClick={handleSave} disabled={!providerId.trim()}>
            {t('common.save')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
