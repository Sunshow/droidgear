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
  ClipboardCopy,
  FileCode,
  Settings,
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
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  DropdownMenuShortcut,
} from '@/components/ui/dropdown-menu'
import { cn } from '@/lib/utils'
import { TerminalView, type TerminalViewRef } from './TerminalView'
import { DerivedTerminalBar } from './DerivedTerminalBar'
import { TerminalSnippetDialog } from './TerminalSnippetDialog'
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
  const terminalCopyOnSelect = useTerminalStore(
    state => state.terminalCopyOnSelect
  )
  const setTerminalCopyOnSelect = useTerminalStore(
    state => state.setTerminalCopyOnSelect
  )

  // Derived terminal actions
  const createDerivedTerminal = useTerminalStore(
    state => state.createDerivedTerminal
  )
  const closeDerivedTerminal = useTerminalStore(
    state => state.closeDerivedTerminal
  )
  const selectDerivedTerminal = useTerminalStore(
    state => state.selectDerivedTerminal
  )
  const setDerivedTerminalNotification = useTerminalStore(
    state => state.setDerivedTerminalNotification
  )

  // Snippets
  const snippets = useTerminalStore(state => state.snippets)

  const [editingId, setEditingId] = useState<string | null>(null)
  const [editingName, setEditingName] = useState('')
  const [snippetDialogOpen, setSnippetDialogOpen] = useState(false)
  const [snippetDropdownOpen, setSnippetDropdownOpen] = useState(false)
  const renameInputRef = useRef<HTMLInputElement>(null)
  const isEnteringRenameRef = useRef(false)
  // Map key format: terminalId or terminalId:derivedId
  const terminalRefs = useRef<Map<string, TerminalViewRef>>(new Map())

  // Track which derived terminals are newly created (for autoExecute)
  const [newlyCreatedDerived, setNewlyCreatedDerived] = useState<Set<string>>(
    new Set()
  )

  // Track which terminals are ready (for sequential derived terminal rendering)
  const [readyTerminals, setReadyTerminals] = useState<Set<string>>(new Set())

  // Get the currently selected terminal
  const selectedTerminal = terminals.find(t => t.id === selectedTerminalId)

  // Helper to check if a derived terminal can be rendered
  const canRenderDerived = (
    terminalId: string,
    derivedTerminals: (typeof terminals)[0]['derivedTerminals'],
    derivedIndex: number
  ) => {
    if (derivedIndex === 0) {
      // First derived can render when main terminal is ready
      return readyTerminals.has(terminalId)
    }
    // Subsequent derived can render when previous derived is ready
    const prevDerived = derivedTerminals[derivedIndex - 1]
    if (!prevDerived) return false
    return readyTerminals.has(`${terminalId}:${prevDerived.id}`)
  }

  // Handle terminal ready callback
  const handleTerminalReady = (refKey: string) => {
    setReadyTerminals(prev => new Set(prev).add(refKey))
  }

  // Focus terminal when selection changes
  useEffect(() => {
    if (selectedTerminalId) {
      const terminal = terminals.find(t => t.id === selectedTerminalId)
      const refKey = terminal?.selectedDerivedId
        ? `${selectedTerminalId}:${terminal.selectedDerivedId}`
        : selectedTerminalId
      // Small delay to ensure terminal is visible
      setTimeout(() => {
        terminalRefs.current.get(refKey)?.focus()
      }, 50)
    }
  }, [selectedTerminalId, selectedTerminal?.selectedDerivedId, terminals])

  // Focus rename input when editing starts (fallback for non-context-menu triggers)
  useEffect(() => {
    if (editingId && !isEnteringRenameRef.current) {
      renameInputRef.current?.focus()
      renameInputRef.current?.select()
    }
  }, [editingId])

  // Keyboard shortcut to open Snippets dropdown (Ctrl/Cmd + Shift + S)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Only respond when there's a selected terminal
      if (!selectedTerminalId) return

      // Ctrl/Cmd + Shift + S to open Snippets
      if (
        (e.metaKey || e.ctrlKey) &&
        e.shiftKey &&
        e.key.toLowerCase() === 's'
      ) {
        e.preventDefault()
        setSnippetDropdownOpen(true)
      }
    }

    document.addEventListener('keydown', handleKeyDown)
    return () => document.removeEventListener('keydown', handleKeyDown)
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
      const dirPath = selected as string
      const dirName = dirPath.split(/[/\\]/).filter(Boolean).pop() || undefined
      createTerminal(dirName, dirPath)
    }
  }

  const handleCloseTerminal = (id: string) => {
    closeTerminal(id)
  }

  const handleReloadTerminal = (id: string) => {
    terminalRefs.current.get(id)?.reload()
  }

  const handleStartRename = (id: string, currentName: string) => {
    isEnteringRenameRef.current = true
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

  const handleDerivedTerminalExit = (
    parentId: string,
    derivedId: string,
    exitCode: number
  ) => {
    const terminal = terminals.find(t => t.id === parentId)
    // Show notification if this derived terminal is not currently selected
    if (
      selectedTerminalId !== parentId ||
      terminal?.selectedDerivedId !== derivedId
    ) {
      setDerivedTerminalNotification(parentId, derivedId, true)
    }
    console.log(
      `Derived terminal ${derivedId} of ${parentId} exited with code ${exitCode}`
    )
  }

  const handleCreateDerived = (
    parentId: string,
    command: string,
    name?: string
  ) => {
    const derivedId = createDerivedTerminal(parentId, command, name)
    // Mark as newly created for autoExecute
    setNewlyCreatedDerived(prev => new Set(prev).add(derivedId))
  }

  const handleCloseDerived = (parentId: string, derivedId: string) => {
    closeDerivedTerminal(parentId, derivedId)
    // Clean up from newly created set
    setNewlyCreatedDerived(prev => {
      const next = new Set(prev)
      next.delete(derivedId)
      return next
    })
  }

  const handleSelectDerived = (parentId: string, derivedId: string | null) => {
    selectDerivedTerminal(parentId, derivedId)
  }

  // Execute a snippet by writing to the current terminal
  const handleExecuteSnippet = (snippetId: string) => {
    const snippet = snippets.find(s => s.id === snippetId)
    if (!snippet || !selectedTerminalId) return

    const terminal = terminals.find(t => t.id === selectedTerminalId)
    const refKey = terminal?.selectedDerivedId
      ? `${selectedTerminalId}:${terminal.selectedDerivedId}`
      : selectedTerminalId

    const terminalRef = terminalRefs.current.get(refKey)
    if (terminalRef) {
      terminalRef.write(snippet.content)
      if (snippet.autoExecute) {
        terminalRef.write('\r')
      }
      setSnippetDropdownOpen(false)
    }
  }

  // Handle keyboard shortcuts in snippet dropdown (1-9, 0 for 1-10)
  const handleSnippetDropdownKeyDown = (e: React.KeyboardEvent) => {
    const key = e.key
    if (key >= '1' && key <= '9') {
      const index = parseInt(key, 10) - 1
      const snippet = snippets[index]
      if (snippet) {
        e.preventDefault()
        handleExecuteSnippet(snippet.id)
      }
    } else if (key === '0') {
      // 0 maps to index 9 (10th item)
      const snippet = snippets[9]
      if (snippet) {
        e.preventDefault()
        handleExecuteSnippet(snippet.id)
      }
    }
  }

  const getStatusIcon = (isSelected: boolean) => {
    if (isSelected) {
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
                <ClipboardCopy className="h-4 w-4 text-muted-foreground" />
                <Switch
                  checked={terminalCopyOnSelect}
                  onCheckedChange={setTerminalCopyOnSelect}
                />
              </div>
            </TooltipTrigger>
            <TooltipContent>{t('droid.terminal.copyOnSelect')}</TooltipContent>
          </Tooltip>
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
          <DropdownMenu
            open={snippetDropdownOpen}
            onOpenChange={setSnippetDropdownOpen}
          >
            <Tooltip>
              <TooltipTrigger asChild>
                <DropdownMenuTrigger asChild>
                  <Button variant="outline" size="icon" className="h-8 w-8">
                    <FileCode className="h-4 w-4" />
                  </Button>
                </DropdownMenuTrigger>
              </TooltipTrigger>
              <TooltipContent>
                {t('droid.terminal.snippets.title')} (⌘⇧S)
              </TooltipContent>
            </Tooltip>
            <DropdownMenuContent
              align="end"
              onKeyDown={handleSnippetDropdownKeyDown}
              onCloseAutoFocus={e => {
                e.preventDefault()
                // Focus the current terminal
                const terminal = terminals.find(
                  t => t.id === selectedTerminalId
                )
                const refKey = terminal?.selectedDerivedId
                  ? `${selectedTerminalId}:${terminal.selectedDerivedId}`
                  : selectedTerminalId
                if (refKey) {
                  terminalRefs.current.get(refKey)?.focus()
                }
              }}
            >
              {snippets.length === 0 ? (
                <DropdownMenuItem disabled>
                  {t('droid.terminal.snippets.empty')}
                </DropdownMenuItem>
              ) : (
                snippets.slice(0, 10).map((snippet, index) => (
                  <DropdownMenuItem
                    key={snippet.id}
                    onClick={() => handleExecuteSnippet(snippet.id)}
                  >
                    <span className="truncate max-w-48">{snippet.name}</span>
                    <DropdownMenuShortcut>
                      {index < 9 ? index + 1 : 0}
                    </DropdownMenuShortcut>
                  </DropdownMenuItem>
                ))
              )}
              {snippets.length > 10 && (
                <>
                  <DropdownMenuSeparator />
                  {snippets.slice(10).map(snippet => (
                    <DropdownMenuItem
                      key={snippet.id}
                      onClick={() => handleExecuteSnippet(snippet.id)}
                    >
                      <span className="truncate max-w-48">{snippet.name}</span>
                    </DropdownMenuItem>
                  ))}
                </>
              )}
              <DropdownMenuSeparator />
              <DropdownMenuItem onClick={() => setSnippetDialogOpen(true)}>
                <Settings className="h-4 w-4 mr-2" />
                {t('droid.terminal.snippets.manage')}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
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
                          'relative w-full text-start p-2 rounded-lg hover:bg-accent transition-colors overflow-hidden',
                          selectedTerminalId === terminal.id && 'bg-accent'
                        )}
                      >
                        {terminal.hasNotification && (
                          <span className="absolute top-1 right-1 h-2 w-2 rounded-full bg-orange-500" />
                        )}
                        <div className="flex items-center gap-2">
                          {getStatusIcon(selectedTerminalId === terminal.id)}
                          {editingId === terminal.id ? (
                            <Input
                              ref={renameInputRef}
                              value={editingName}
                              onChange={e => setEditingName(e.target.value)}
                              onBlur={handleFinishRename}
                              onKeyDown={handleKeyDown}
                              className="h-6 text-sm"
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
                    <ContextMenuContent
                      onCloseAutoFocus={e => {
                        if (isEnteringRenameRef.current) {
                          e.preventDefault()
                          isEnteringRenameRef.current = false
                          renameInputRef.current?.focus()
                          renameInputRef.current?.select()
                        }
                      }}
                    >
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
                  'h-full w-full flex flex-col',
                  terminal.id !== selectedTerminalId && 'hidden'
                )}
              >
                {/* Derived Terminal Bar */}
                <DerivedTerminalBar
                  terminalId={terminal.id}
                  derivedTerminals={terminal.derivedTerminals}
                  selectedDerivedId={terminal.selectedDerivedId}
                  onSelectDerived={derivedId =>
                    handleSelectDerived(terminal.id, derivedId)
                  }
                  onCloseDerived={derivedId =>
                    handleCloseDerived(terminal.id, derivedId)
                  }
                  onCreateDerived={(command, name) =>
                    handleCreateDerived(terminal.id, command, name)
                  }
                />

                {/* Main Terminal */}
                <div
                  className={cn(
                    'flex-1 min-h-0',
                    terminal.selectedDerivedId !== null && 'hidden'
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
                    copyOnSelect={terminalCopyOnSelect}
                    onExit={exitCode =>
                      handleTerminalExit(terminal.id, exitCode)
                    }
                    onReady={() => handleTerminalReady(terminal.id)}
                  />
                </div>

                {/* Derived Terminals - rendered sequentially */}
                {terminal.derivedTerminals.map((derived, index) => {
                  const refKey = `${terminal.id}:${derived.id}`
                  const canRender = canRenderDerived(
                    terminal.id,
                    terminal.derivedTerminals,
                    index
                  )
                  if (!canRender) return null
                  return (
                    <div
                      key={derived.id}
                      className={cn(
                        'flex-1 min-h-0',
                        terminal.selectedDerivedId !== derived.id && 'hidden'
                      )}
                    >
                      <TerminalView
                        ref={ref => {
                          if (ref) {
                            terminalRefs.current.set(refKey, ref)
                          } else {
                            terminalRefs.current.delete(refKey)
                          }
                        }}
                        terminalId={derived.id}
                        cwd={terminal.cwd || undefined}
                        forceDark={terminalForceDark}
                        copyOnSelect={terminalCopyOnSelect}
                        prefillCommand={derived.command}
                        autoExecute={newlyCreatedDerived.has(derived.id)}
                        onExit={exitCode =>
                          handleDerivedTerminalExit(
                            terminal.id,
                            derived.id,
                            exitCode
                          )
                        }
                        onReady={() => handleTerminalReady(refKey)}
                      />
                    </div>
                  )
                })}
              </div>
            ))
          ) : (
            <div className="flex items-center justify-center h-full text-muted-foreground">
              <p>{t('droid.terminal.selectOrCreate')}</p>
            </div>
          )}
        </div>
      </div>

      {/* Snippet Management Dialog */}
      <TerminalSnippetDialog
        open={snippetDialogOpen}
        onOpenChange={setSnippetDialogOpen}
      />
    </div>
  )
}
