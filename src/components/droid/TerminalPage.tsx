import { useState, useRef, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import {
  Plus,
  X,
  Circle,
  Pencil,
  FolderOpen,
  Copy,
  Moon,
  RotateCw,
} from 'lucide-react'
import { open } from '@tauri-apps/plugin-dialog'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Switch } from '@/components/ui/switch'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from '@/components/ui/context-menu'
import { cn } from '@/lib/utils'
import { TerminalView, type TerminalViewRef } from './TerminalView'
import { useTerminalStore } from '@/store/terminal-store'

export function TerminalPage() {
  const { t } = useTranslation()
  const terminals = useTerminalStore(state => state.terminals)
  const selectedTerminalId = useTerminalStore(state => state.selectedTerminalId)
  const createTerminal = useTerminalStore(state => state.createTerminal)
  const closeTerminal = useTerminalStore(state => state.closeTerminal)
  const renameTerminal = useTerminalStore(state => state.renameTerminal)
  const selectTerminal = useTerminalStore(state => state.selectTerminal)
  const setTerminalNotification = useTerminalStore(
    state => state.setTerminalNotification
  )
  const terminalForceDark = useTerminalStore(state => state.terminalForceDark)
  const setTerminalForceDark = useTerminalStore(
    state => state.setTerminalForceDark
  )

  const [editingId, setEditingId] = useState<string | null>(null)
  const [editingName, setEditingName] = useState('')
  const terminalRefs = useRef<Map<string, TerminalViewRef>>(new Map())

  // Focus terminal when selection changes
  useEffect(() => {
    if (selectedTerminalId) {
      // Small delay to ensure terminal is visible
      setTimeout(() => {
        terminalRefs.current.get(selectedTerminalId)?.focus()
      }, 50)
    }
  }, [selectedTerminalId])

  const handleCreateTerminal = () => {
    createTerminal()
  }

  const handleCreateTerminalWithDir = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: t('droid.terminal.selectDirectory'),
    })
    if (selected) {
      createTerminal(undefined, selected as string)
    }
  }

  const handleCloseTerminal = (id: string) => {
    closeTerminal(id)
  }

  const handleReloadTerminal = (id: string) => {
    terminalRefs.current.get(id)?.reload()
  }

  const handleStartRename = (id: string, currentName: string) => {
    setEditingId(id)
    setEditingName(currentName)
  }

  const handleFinishRename = () => {
    if (editingId && editingName.trim()) {
      renameTerminal(editingId, editingName.trim())
    }
    setEditingId(null)
    setEditingName('')
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleFinishRename()
    } else if (e.key === 'Escape') {
      setEditingId(null)
      setEditingName('')
    }
  }

  const handleTerminalExit = (terminalId: string, exitCode: number) => {
    // Show notification if this terminal is not currently selected
    if (selectedTerminalId !== terminalId) {
      setTerminalNotification(terminalId, true)
    }
    console.log(`Terminal ${terminalId} exited with code ${exitCode}`)
  }

  const getStatusIcon = (hasNotification: boolean) => {
    if (hasNotification) {
      return <Circle className="h-3 w-3 fill-primary text-primary" />
    }
    return <Circle className="h-3 w-3 text-muted-foreground" />
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b">
        <h1 className="text-xl font-semibold">{t('droid.terminal.title')}</h1>
        <div className="flex items-center gap-2">
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="flex items-center gap-2">
                <Moon className="h-4 w-4 text-muted-foreground" />
                <Switch
                  checked={terminalForceDark}
                  onCheckedChange={setTerminalForceDark}
                />
              </div>
            </TooltipTrigger>
            <TooltipContent>{t('droid.terminal.forceDark')}</TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="outline"
                size="icon"
                className="h-8 w-8"
                onClick={handleCreateTerminalWithDir}
              >
                <FolderOpen className="h-4 w-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              {t('droid.terminal.newTerminalInDir')}
            </TooltipContent>
          </Tooltip>
          <Button variant="outline" size="sm" onClick={handleCreateTerminal}>
            <Plus className="h-4 w-4 mr-2" />
            {t('droid.terminal.newTerminal')}
          </Button>
        </div>
      </div>

      <div className="flex flex-1 overflow-hidden">
        {/* Terminal List */}
        <div className="w-56 border-r flex flex-col overflow-hidden">
          <div className="flex-1 overflow-y-auto">
            <div className="p-2 flex flex-col gap-1.5">
              {terminals.length === 0 ? (
                <div className="text-center text-muted-foreground text-sm py-8">
                  {t('droid.terminal.noTerminals')}
                </div>
              ) : (
                terminals.map(terminal => (
                  <ContextMenu key={terminal.id}>
                    <ContextMenuTrigger>
                      <button
                        onClick={() => selectTerminal(terminal.id)}
                        className={cn(
                          'w-full text-start p-2 rounded-lg hover:bg-accent transition-colors overflow-hidden',
                          selectedTerminalId === terminal.id && 'bg-accent'
                        )}
                      >
                        <div className="flex items-center gap-2">
                          {getStatusIcon(terminal.hasNotification)}
                          {editingId === terminal.id ? (
                            <Input
                              value={editingName}
                              onChange={e => setEditingName(e.target.value)}
                              onBlur={handleFinishRename}
                              onKeyDown={handleKeyDown}
                              className="h-6 text-sm"
                              autoFocus
                              onClick={e => e.stopPropagation()}
                            />
                          ) : (
                            <span className="font-medium text-sm truncate flex-1">
                              {terminal.name}
                            </span>
                          )}
                        </div>
                        {terminal.cwd && (
                          <div className="text-xs text-muted-foreground mt-1 truncate pl-5">
                            {terminal.cwd}
                          </div>
                        )}
                      </button>
                    </ContextMenuTrigger>
                    <ContextMenuContent>
                      <ContextMenuItem
                        onClick={() =>
                          handleStartRename(terminal.id, terminal.name)
                        }
                      >
                        <Pencil className="h-4 w-4 mr-2" />
                        {t('droid.terminal.rename')}
                      </ContextMenuItem>
                      <ContextMenuItem
                        onClick={() => handleReloadTerminal(terminal.id)}
                      >
                        <RotateCw className="h-4 w-4 mr-2" />
                        {t('droid.terminal.reload')}
                      </ContextMenuItem>
                      {terminal.cwd && (
                        <ContextMenuItem
                          onClick={() => writeText(terminal.cwd)}
                        >
                          <Copy className="h-4 w-4 mr-2" />
                          {t('droid.terminal.copyPath')}
                        </ContextMenuItem>
                      )}
                      <ContextMenuItem
                        onClick={() => handleCloseTerminal(terminal.id)}
                        className="text-destructive"
                      >
                        <X className="h-4 w-4 mr-2" />
                        {t('droid.terminal.close')}
                      </ContextMenuItem>
                    </ContextMenuContent>
                  </ContextMenu>
                ))
              )}
            </div>
          </div>
        </div>

        {/* Terminal View */}
        <div className="flex-1 flex flex-col min-w-0">
          {terminals.length > 0 ? (
            terminals.map(terminal => (
              <div
                key={terminal.id}
                className={cn(
                  'h-full w-full',
                  terminal.id !== selectedTerminalId && 'hidden'
                )}
              >
                <TerminalView
                  ref={ref => {
                    if (ref) {
                      terminalRefs.current.set(terminal.id, ref)
                    } else {
                      terminalRefs.current.delete(terminal.id)
                    }
                  }}
                  terminalId={terminal.id}
                  cwd={terminal.cwd || undefined}
                  forceDark={terminalForceDark}
                  onExit={exitCode => handleTerminalExit(terminal.id, exitCode)}
                />
              </div>
            ))
          ) : (
            <div className="flex items-center justify-center h-full text-muted-foreground">
              <p>{t('droid.terminal.selectOrCreate')}</p>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
