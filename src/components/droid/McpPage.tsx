import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { Plus, Pencil, Trash2, X } from 'lucide-react'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import { Badge } from '@/components/ui/badge'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  commands,
  type McpServer,
  type McpServerConfig,
  type McpServerType,
} from '@/lib/bindings'

// MCP Server Presets for Quick Add
interface McpPreset {
  id: string
  name: string
  descriptionKey: string
  config: Omit<McpServerConfig, 'disabled'>
}

const MCP_PRESETS: McpPreset[] = [
  {
    id: 'playwright',
    name: 'Playwright',
    descriptionKey: 'mcp.presets.playwright.description',
    config: {
      type: 'stdio',
      command: 'npx',
      args: ['@playwright/mcp@latest'],
    },
  },
  {
    id: 'chrome-devtools',
    name: 'Chrome DevTools',
    descriptionKey: 'mcp.presets.chromeDevtools.description',
    config: {
      type: 'stdio',
      command: 'npx',
      args: ['-y', 'chrome-devtools-mcp@latest'],
    },
  },
]

interface KeyValuePair {
  key: string
  value: string
}

export function McpPage() {
  const { t } = useTranslation()
  const [servers, setServers] = useState<McpServer[]>([])
  const [loading, setLoading] = useState(true)
  const [dialogOpen, setDialogOpen] = useState(false)
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false)
  const [serverToDelete, setServerToDelete] = useState<string | null>(null)
  const [editingServer, setEditingServer] = useState<McpServer | null>(null)

  // Form state
  const [serverName, setServerName] = useState('')
  const [serverType, setServerType] = useState<McpServerType>('stdio')
  const [command, setCommand] = useState('')
  const [args, setArgs] = useState<string[]>([])
  const [argInput, setArgInput] = useState('')
  const [envVars, setEnvVars] = useState<KeyValuePair[]>([])
  const [url, setUrl] = useState('')
  const [headers, setHeaders] = useState<KeyValuePair[]>([])

  useEffect(() => {
    let cancelled = false
    const fetchServers = async () => {
      setLoading(true)
      const result = await commands.loadMcpServers()
      if (cancelled) return
      if (result.status === 'ok') {
        setServers(result.data)
      } else {
        toast.error(t('toast.error.generic'))
      }
      setLoading(false)
    }
    fetchServers()
    return () => {
      cancelled = true
    }
  }, [t])

  const loadServers = async () => {
    setLoading(true)
    const result = await commands.loadMcpServers()
    if (result.status === 'ok') {
      setServers(result.data)
    } else {
      toast.error(t('toast.error.generic'))
    }
    setLoading(false)
  }

  const resetForm = () => {
    setServerName('')
    setServerType('stdio')
    setCommand('')
    setArgs([])
    setArgInput('')
    setEnvVars([])
    setUrl('')
    setHeaders([])
    setEditingServer(null)
  }

  const openAddDialog = () => {
    resetForm()
    setDialogOpen(true)
  }

  const openEditDialog = (server: McpServer) => {
    setEditingServer(server)
    setServerName(server.name)
    setServerType(server.config.type)
    if (server.config.type === 'stdio') {
      setCommand(server.config.command ?? '')
      setArgs(server.config.args ?? [])
      setEnvVars(
        Object.entries(server.config.env ?? {}).map(([key, value]) => ({
          key,
          value: value ?? '',
        }))
      )
    } else {
      setUrl(server.config.url ?? '')
      setHeaders(
        Object.entries(server.config.headers ?? {}).map(([key, value]) => ({
          key,
          value: value ?? '',
        }))
      )
    }
    setDialogOpen(true)
  }

  const handleSave = async () => {
    if (!serverName.trim()) {
      toast.error(t('mcp.validation.nameRequired'))
      return
    }

    const config: McpServerConfig =
      serverType === 'stdio'
        ? {
            type: 'stdio',
            disabled: editingServer?.config.disabled ?? false,
            command: command || null,
            args: args.length > 0 ? args : null,
            env:
              envVars.length > 0
                ? Object.fromEntries(envVars.map(e => [e.key, e.value]))
                : null,
          }
        : {
            type: 'http',
            disabled: editingServer?.config.disabled ?? false,
            url: url || null,
            headers:
              headers.length > 0
                ? Object.fromEntries(headers.map(h => [h.key, h.value]))
                : null,
          }

    const result = await commands.saveMcpServer({
      name: serverName.trim(),
      config,
    })

    if (result.status === 'ok') {
      toast.success(t('common.save'))
      setDialogOpen(false)
      resetForm()
      loadServers()
    } else {
      toast.error(result.error)
    }
  }

  const handleToggle = async (name: string, disabled: boolean) => {
    const result = await commands.toggleMcpServer(name, disabled)
    if (result.status === 'ok') {
      setServers(prev =>
        prev.map(s =>
          s.name === name ? { ...s, config: { ...s.config, disabled } } : s
        )
      )
    } else {
      toast.error(result.error)
    }
  }

  const handleDelete = async () => {
    if (!serverToDelete) return
    const result = await commands.deleteMcpServer(serverToDelete)
    if (result.status === 'ok') {
      toast.success(t('common.delete'))
      setServers(prev => prev.filter(s => s.name !== serverToDelete))
    } else {
      toast.error(result.error)
    }
    setDeleteDialogOpen(false)
    setServerToDelete(null)
  }

  const addArg = () => {
    if (argInput.trim()) {
      setArgs(prev => [...prev, argInput.trim()])
      setArgInput('')
    }
  }

  const removeArg = (index: number) => {
    setArgs(prev => prev.filter((_, i) => i !== index))
  }

  const addEnvVar = () => {
    setEnvVars(prev => [...prev, { key: '', value: '' }])
  }

  const updateEnvVar = (
    index: number,
    field: 'key' | 'value',
    value: string
  ) => {
    setEnvVars(prev =>
      prev.map((e, i) => (i === index ? { ...e, [field]: value } : e))
    )
  }

  const removeEnvVar = (index: number) => {
    setEnvVars(prev => prev.filter((_, i) => i !== index))
  }

  const addHeader = () => {
    setHeaders(prev => [...prev, { key: '', value: '' }])
  }

  const updateHeader = (
    index: number,
    field: 'key' | 'value',
    value: string
  ) => {
    setHeaders(prev =>
      prev.map((h, i) => (i === index ? { ...h, [field]: value } : h))
    )
  }

  const removeHeader = (index: number) => {
    setHeaders(prev => prev.filter((_, i) => i !== index))
  }

  const getServerDescription = (server: McpServer) => {
    if (server.config.type === 'stdio') {
      const cmd = server.config.command ?? ''
      const args = server.config.args?.join(' ') ?? ''
      return `${cmd} ${args}`.trim()
    }
    return server.config.url ?? ''
  }

  const handleAddPreset = async (preset: McpPreset) => {
    const result = await commands.saveMcpServer({
      name: preset.id,
      config: { ...preset.config, disabled: false } as McpServerConfig,
    })

    if (result.status === 'ok') {
      toast.success(t('common.save'))
      loadServers()
    } else {
      toast.error(result.error)
    }
  }

  // Filter presets that are not already added
  const availablePresets = MCP_PRESETS.filter(
    preset => !servers.some(s => s.name === preset.id)
  )

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between p-4 border-b">
        <h1 className="text-xl font-semibold">{t('mcp.title')}</h1>
        <Button size="sm" onClick={openAddDialog}>
          <Plus className="h-4 w-4 mr-2" />
          {t('mcp.addServer')}
        </Button>
      </div>

      <div className="flex-1 overflow-y-auto p-4">
        {loading ? (
          <div className="text-center text-muted-foreground py-8">
            {t('common.loading')}
          </div>
        ) : (
          <div className="space-y-6">
            {/* Configured Servers */}
            {servers.length === 0 ? (
              <div className="text-center text-muted-foreground py-8">
                <p>{t('mcp.noServers')}</p>
                <p className="text-sm mt-1">{t('mcp.noServersHint')}</p>
              </div>
            ) : (
              <div className="space-y-3">
                {servers.map(server => (
                  <div
                    key={server.name}
                    className="border rounded-lg p-4 space-y-2"
                  >
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <span className="font-medium">{server.name}</span>
                        <Badge variant="secondary">{server.config.type}</Badge>
                      </div>
                      <div className="flex items-center gap-2">
                        <Switch
                          checked={!server.config.disabled}
                          onCheckedChange={checked =>
                            handleToggle(server.name, !checked)
                          }
                        />
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => openEditDialog(server)}
                        >
                          <Pencil className="h-4 w-4" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => {
                            setServerToDelete(server.name)
                            setDeleteDialogOpen(true)
                          }}
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      </div>
                    </div>
                    <p className="text-sm text-muted-foreground font-mono truncate">
                      {getServerDescription(server)}
                    </p>
                  </div>
                ))}
              </div>
            )}

            {/* Quick Add Section - only show if there are available presets */}
            {availablePresets.length > 0 && (
              <div className="space-y-3">
                <h2 className="text-sm font-medium text-muted-foreground">
                  {t('mcp.presets.title')}
                </h2>
                {availablePresets.map(preset => (
                  <div
                    key={preset.id}
                    className="border rounded-lg p-4 flex items-center justify-between"
                  >
                    <div>
                      <p className="font-medium">{preset.name}</p>
                      <p className="text-sm text-muted-foreground">
                        {t(preset.descriptionKey)}
                      </p>
                    </div>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => handleAddPreset(preset)}
                    >
                      <Plus className="h-4 w-4 mr-1" />
                      {t('mcp.presets.add')}
                    </Button>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </div>

      {/* Add/Edit Dialog */}
      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogContent className="max-w-lg max-h-[80vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>
              {editingServer ? t('mcp.editServer') : t('mcp.addServer')}
            </DialogTitle>
            <DialogDescription>{t('mcp.dialogDescription')}</DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>{t('mcp.serverName')}</Label>
              <Input
                value={serverName}
                onChange={e => setServerName(e.target.value)}
                placeholder="my-server"
                disabled={!!editingServer}
              />
            </div>

            <div className="space-y-2">
              <Label>{t('mcp.serverType')}</Label>
              <Select
                value={serverType}
                onValueChange={v => setServerType(v as McpServerType)}
                disabled={!!editingServer}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="stdio">{t('mcp.type.stdio')}</SelectItem>
                  <SelectItem value="http">{t('mcp.type.http')}</SelectItem>
                </SelectContent>
              </Select>
            </div>

            {serverType === 'stdio' ? (
              <>
                <div className="space-y-2">
                  <Label>{t('mcp.command')}</Label>
                  <Input
                    value={command}
                    onChange={e => setCommand(e.target.value)}
                    placeholder="npx"
                  />
                </div>

                <div className="space-y-2">
                  <Label>{t('mcp.args')}</Label>
                  <div className="flex gap-2">
                    <Input
                      value={argInput}
                      onChange={e => setArgInput(e.target.value)}
                      placeholder="-y"
                      onKeyDown={e => e.key === 'Enter' && addArg()}
                    />
                    <Button type="button" variant="outline" onClick={addArg}>
                      {t('common.add')}
                    </Button>
                  </div>
                  {args.length > 0 && (
                    <div className="flex flex-wrap gap-1 mt-2">
                      {args.map((arg, i) => (
                        <Badge key={i} variant="secondary" className="gap-1">
                          {arg}
                          <X
                            className="h-3 w-3 cursor-pointer"
                            onClick={() => removeArg(i)}
                          />
                        </Badge>
                      ))}
                    </div>
                  )}
                </div>

                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <Label>{t('mcp.envVars')}</Label>
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      onClick={addEnvVar}
                    >
                      <Plus className="h-4 w-4 mr-1" />
                      {t('mcp.addEnvVar')}
                    </Button>
                  </div>
                  {envVars.map((env, i) => (
                    <div key={i} className="flex gap-2">
                      <Input
                        placeholder="KEY"
                        value={env.key}
                        onChange={e => updateEnvVar(i, 'key', e.target.value)}
                      />
                      <Input
                        placeholder="value"
                        value={env.value}
                        onChange={e => updateEnvVar(i, 'value', e.target.value)}
                      />
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => removeEnvVar(i)}
                      >
                        <X className="h-4 w-4" />
                      </Button>
                    </div>
                  ))}
                </div>
              </>
            ) : (
              <>
                <div className="space-y-2">
                  <Label>{t('mcp.url')}</Label>
                  <Input
                    value={url}
                    onChange={e => setUrl(e.target.value)}
                    placeholder="https://mcp.example.com/mcp"
                  />
                </div>

                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <Label>{t('mcp.headers')}</Label>
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      onClick={addHeader}
                    >
                      <Plus className="h-4 w-4 mr-1" />
                      {t('mcp.addHeader')}
                    </Button>
                  </div>
                  {headers.map((header, i) => (
                    <div key={i} className="flex gap-2">
                      <Input
                        placeholder="Header-Name"
                        value={header.key}
                        onChange={e => updateHeader(i, 'key', e.target.value)}
                      />
                      <Input
                        placeholder="value"
                        value={header.value}
                        onChange={e => updateHeader(i, 'value', e.target.value)}
                      />
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => removeHeader(i)}
                      >
                        <X className="h-4 w-4" />
                      </Button>
                    </div>
                  ))}
                </div>
              </>
            )}
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setDialogOpen(false)}>
              {t('common.cancel')}
            </Button>
            <Button onClick={handleSave}>{t('common.save')}</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Delete Confirmation */}
      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('mcp.deleteServer')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('mcp.deleteConfirm', { name: serverToDelete })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <Button variant="destructive" onClick={handleDelete}>
              {t('common.delete')}
            </Button>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
