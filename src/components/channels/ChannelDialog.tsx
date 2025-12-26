import { useState, useEffect } from 'react'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'
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
  'one-api': '',
}

interface ChannelFormProps {
  channel?: Channel
  onSave: (channel: Channel, username: string, password: string) => void
  onCancel: () => void
}

function ChannelForm({ channel, onSave, onCancel }: ChannelFormProps) {
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
    if (!channel) {
      setBaseUrl(defaultBaseUrls[value])
    }
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
      <div className="grid gap-4 py-4">
        <div className="grid gap-2">
          <Label htmlFor="name">Name</Label>
          <Input
            id="name"
            value={name}
            onChange={e => setName(e.target.value)}
            placeholder="My API Channel"
          />
        </div>

        <div className="grid gap-2">
          <Label htmlFor="type">Type</Label>
          <Select value={channelType} onValueChange={handleTypeChange}>
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="new-api">New API</SelectItem>
              <SelectItem value="one-api">One API</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div className="grid gap-2">
          <Label htmlFor="baseUrl">API URL</Label>
          <Input
            id="baseUrl"
            value={baseUrl}
            onChange={e => setBaseUrl(e.target.value)}
            placeholder="https://api.example.com"
          />
        </div>

        <div className="grid gap-2">
          <Label htmlFor="username">Username</Label>
          <Input
            id="username"
            value={username}
            onChange={e => setUsername(e.target.value)}
            placeholder={isLoadingCredentials ? 'Loading...' : 'Enter username'}
            disabled={isLoadingCredentials}
          />
        </div>

        <div className="grid gap-2">
          <Label htmlFor="password">Password</Label>
          <Input
            id="password"
            type="password"
            value={password}
            onChange={e => setPassword(e.target.value)}
            placeholder={isLoadingCredentials ? 'Loading...' : 'Enter password'}
            disabled={isLoadingCredentials}
          />
          <p className="text-xs text-muted-foreground">
            Credentials are stored locally in ~/.droidgear/auth/
          </p>
        </div>

        <div className="flex items-center justify-between">
          <Label htmlFor="enabled">Enabled</Label>
          <Switch id="enabled" checked={enabled} onCheckedChange={setEnabled} />
        </div>
      </div>

      <DialogFooter>
        <Button variant="outline" onClick={onCancel}>
          Cancel
        </Button>
        <Button onClick={handleSave} disabled={!isValid || isLoadingCredentials}>
          {channel ? 'Save' : 'Add'}
        </Button>
      </DialogFooter>
    </>
  )
}

export function ChannelDialog({
  open,
  onOpenChange,
  channel,
  onSave,
}: ChannelDialogProps) {
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
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>{channel ? 'Edit Channel' : 'Add Channel'}</DialogTitle>
        </DialogHeader>
        {open && (
          <ChannelForm
            key={formKey}
            channel={channel}
            onSave={handleSave}
            onCancel={() => onOpenChange(false)}
          />
        )}
      </DialogContent>
    </Dialog>
  )
}
