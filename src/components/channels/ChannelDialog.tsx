import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { Loader2, ScanSearch } from 'lucide-react'
import {
  ResizableDialog,
  ResizableDialogContent,
  ResizableDialogHeader,
  ResizableDialogBody,
  ResizableDialogTitle,
  ResizableDialogFooter,
} from '@/components/ui/resizable-dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { commands, type Channel, type ChannelType } from '@/lib/bindings'

interface ChannelDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  channel?: Channel
  onSave: (channel: Channel, username: string, password: string) => void
}

const defaultBaseUrls: Record<ChannelType, string> = {
  'new-api': 'https://api.newapi.ai',
  'sub-2-api': '',
}

interface ChannelFormProps {
  channel?: Channel
  onSave: (channel: Channel, username: string, password: string) => void
  onCancel: () => void
}

function ChannelForm({ channel, onSave, onCancel }: ChannelFormProps) {
  const { t } = useTranslation()
  const [name, setName] = useState(channel?.name ?? '')
  const [channelType, setChannelType] = useState<ChannelType>(
    channel?.type ?? 'new-api'
  )
  const [baseUrl, setBaseUrl] = useState(
    channel?.baseUrl ?? defaultBaseUrls['new-api']
  )
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [enabled, setEnabled] = useState(channel?.enabled ?? true)
  const [isLoadingCredentials, setIsLoadingCredentials] = useState(!!channel)
  const [isDetecting, setIsDetecting] = useState(false)
  const [detectMessage, setDetectMessage] = useState<{
    type: 'success' | 'error'
    text: string
  } | null>(null)

  // Load credentials from storage for existing channels
  useEffect(() => {
    let cancelled = false
    if (channel) {
      commands.getChannelCredentials(channel.id).then(result => {
        if (cancelled) return
        if (result.status === 'ok' && result.data) {
          setUsername(result.data[0])
          setPassword(result.data[1])
        }
        setIsLoadingCredentials(false)
      })
    }
    return () => {
      cancelled = true
    }
  }, [channel])

  const handleTypeChange = (value: ChannelType) => {
    setChannelType(value)
    setDetectMessage(null)
    if (!channel) {
      setBaseUrl(defaultBaseUrls[value])
    }
  }

  const handleDetectType = async () => {
    if (!baseUrl.trim()) return

    setIsDetecting(true)
    setDetectMessage(null)

    const result = await commands.detectChannelType(baseUrl.trim())

    if (result.status === 'ok') {
      setChannelType(result.data)
      const typeName =
        result.data === 'new-api'
          ? t('channels.typeNewApi')
          : t('channels.typeSub2Api')
      setDetectMessage({
        type: 'success',
        text: t('channels.detectSuccess', { type: typeName }),
      })
    } else {
      setDetectMessage({
        type: 'error',
        text: t('channels.detectFailed'),
      })
    }

    setIsDetecting(false)
  }

  const handleSave = () => {
    if (!name || !baseUrl) return

    const newChannel: Channel = {
      id: channel?.id ?? crypto.randomUUID(),
      name,
      type: channelType,
      baseUrl,
      enabled,
      createdAt: channel?.createdAt ?? Date.now(),
    }

    onSave(newChannel, username, password)
  }

  const isValid =
    name.trim() && baseUrl.trim() && username.trim() && password.trim()

  return (
    <>
      <ResizableDialogBody>
        <div className="grid gap-4">
          <div className="grid gap-2">
            <Label htmlFor="name">{t('common.name')}</Label>
            <Input
              id="name"
              value={name}
              onChange={e => setName(e.target.value)}
              placeholder="My API Channel"
            />
          </div>

          <div className="grid gap-2">
            <Label htmlFor="type">{t('channels.type')}</Label>
            <Select value={channelType} onValueChange={handleTypeChange}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="new-api">
                  {t('channels.typeNewApi')}
                </SelectItem>
                <SelectItem value="sub-2-api">
                  {t('channels.typeSub2Api')}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="grid gap-2">
            <Label htmlFor="baseUrl">{t('channels.apiUrl')}</Label>
            <div className="flex gap-2">
              <Input
                id="baseUrl"
                value={baseUrl}
                onChange={e => {
                  setBaseUrl(e.target.value)
                  setDetectMessage(null)
                }}
                placeholder="https://api.example.com"
                className="flex-1"
              />
              <Button
                type="button"
                variant="outline"
                size="icon"
                onClick={handleDetectType}
                disabled={!baseUrl.trim() || isDetecting}
                title={t('channels.detectType')}
              >
                {isDetecting ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  <ScanSearch className="h-4 w-4" />
                )}
              </Button>
            </div>
            {detectMessage && (
              <p
                className={`text-xs ${detectMessage.type === 'success' ? 'text-green-600' : 'text-destructive'}`}
              >
                {detectMessage.text}
              </p>
            )}
          </div>

          <div className="grid gap-2">
            <Label htmlFor="username">{t('channels.username')}</Label>
            <Input
              id="username"
              value={username}
              onChange={e => setUsername(e.target.value)}
              placeholder={
                isLoadingCredentials
                  ? t('common.loading')
                  : t('channels.enterUsername')
              }
              disabled={isLoadingCredentials}
            />
          </div>

          <div className="grid gap-2">
            <Label htmlFor="password">{t('channels.password')}</Label>
            <Input
              id="password"
              type="password"
              value={password}
              onChange={e => setPassword(e.target.value)}
              placeholder={
                isLoadingCredentials
                  ? t('common.loading')
                  : t('channels.enterPassword')
              }
              disabled={isLoadingCredentials}
            />
            <p className="text-xs text-muted-foreground">
              {t('channels.credentialsHint')}
            </p>
          </div>

          <div className="flex items-center justify-between">
            <Label htmlFor="enabled">{t('common.enabled')}</Label>
            <Switch
              id="enabled"
              checked={enabled}
              onCheckedChange={setEnabled}
            />
          </div>
        </div>
      </ResizableDialogBody>

      <ResizableDialogFooter>
        <Button variant="outline" onClick={onCancel}>
          {t('common.cancel')}
        </Button>
        <Button
          onClick={handleSave}
          disabled={!isValid || isLoadingCredentials}
        >
          {channel ? t('common.save') : t('common.add')}
        </Button>
      </ResizableDialogFooter>
    </>
  )
}

export function ChannelDialog({
  open,
  onOpenChange,
  channel,
  onSave,
}: ChannelDialogProps) {
  const { t } = useTranslation()
  const formKey = channel ? `edit-${channel.id}` : 'new'

  const handleSave = (
    newChannel: Channel,
    username: string,
    password: string
  ) => {
    onSave(newChannel, username, password)
    onOpenChange(false)
  }

  return (
    <ResizableDialog open={open} onOpenChange={onOpenChange}>
      <ResizableDialogContent
        defaultWidth={600}
        defaultHeight={550}
        minWidth={500}
        minHeight={400}
      >
        <ResizableDialogHeader>
          <ResizableDialogTitle>
            {channel ? t('channels.editChannel') : t('channels.addChannel')}
          </ResizableDialogTitle>
          <p className="text-sm text-muted-foreground">
            {t('channels.privacyNotice')}
          </p>
        </ResizableDialogHeader>
        {open && (
          <ChannelForm
            key={formKey}
            channel={channel}
            onSave={handleSave}
            onCancel={() => onOpenChange(false)}
          />
        )}
      </ResizableDialogContent>
    </ResizableDialog>
  )
}
