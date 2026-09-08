import { useState, useEffect, useCallback, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import type { BundledTheme } from 'shiki'
import {
  RefreshCw,
  MessageSquare,
  Clock,
  List,
  FolderTree,
  ChevronRight,
  ChevronDown,
  User,
  Bot,
  ArrowDownToLine,
  Brain,
  Eye,
  EyeOff,
  Server,
  Check,
} from 'lucide-react'
import { Streamdown } from 'streamdown'
import { listen } from '@tauri-apps/api/event'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Badge } from '@/components/ui/badge'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import {
  ResizablePanelGroup,
  ResizablePanel,
  ResizableHandle,
} from '@/components/ui/resizable'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { cn } from '@/lib/utils'
import {
  commands,
  type CodexSessionSummary,
  type CodexSessionDetail,
  type CodexSessionProvider,
} from '@/lib/bindings'
import { useTheme } from '@/hooks/use-theme'
import { showContextMenu } from '@/lib/context-menu'

type ViewMode = 'list' | 'grouped'

function projectLabel(project: string): string {
  if (!project) return ''
  const parts = project.replace(/\\/g, '/').split('/').filter(Boolean)
  return parts[parts.length - 1] ?? project
}

export function CodexSessionsPage() {
  const { t } = useTranslation()
  const { theme } = useTheme()
  const [sessions, setSessions] = useState<CodexSessionSummary[]>([])
  const [providers, setProviders] = useState<CodexSessionProvider[]>([])
  const [selectedSession, setSelectedSession] =
    useState<CodexSessionDetail | null>(null)
  const [selectedSessionPath, setSelectedSessionPath] = useState<string | null>(
    null
  )
  const [loading, setLoading] = useState(true)
  const [detailLoading, setDetailLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [viewMode, setViewMode] = useState<ViewMode>('list')
  const [expandedProjects, setExpandedProjects] = useState<Set<string>>(
    new Set()
  )
  const [followMode, setFollowMode] = useState(false)
  const [expandThinking, setExpandThinking] = useState(() => {
    const saved = localStorage.getItem('codex-sessions-expand-thinking')
    return saved === 'true'
  })
  const [hideEmptySessions, setHideEmptySessions] = useState(() => {
    const saved = localStorage.getItem('codex-sessions-hide-empty')
    return saved !== 'false' // 默认为 true
  })

  const [systemPrefersDark, setSystemPrefersDark] = useState(
    () => window.matchMedia('(prefers-color-scheme: dark)').matches
  )

  useEffect(() => {
    if (theme !== 'system') return
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    const handleChange = (e: MediaQueryListEvent) => {
      setSystemPrefersDark(e.matches)
    }
    mediaQuery.addEventListener('change', handleChange)
    return () => mediaQuery.removeEventListener('change', handleChange)
  }, [theme])

  const isDark = theme === 'dark' || (theme === 'system' && systemPrefersDark)
  const shikiTheme: [BundledTheme, BundledTheme] = isDark
    ? ['github-light', 'github-dark']
    : ['github-light', 'github-light']

  const contentScrollRef = useRef<HTMLDivElement>(null)
  const prevMessageCountRef = useRef(0)
  const prevSessionIdRef = useRef<string | null>(null)

  // Reset scroll to top only when switching to a different session
  const selectedSessionId = selectedSession?.id
  useEffect(() => {
    if (selectedSessionId && selectedSessionId !== prevSessionIdRef.current) {
      if (contentScrollRef.current) {
        const viewport = contentScrollRef.current.querySelector(
          '[data-slot="scroll-area-viewport"]'
        )
        if (viewport) {
          viewport.scrollTo(0, 0)
        }
      }
      // Reset message count when switching sessions
      prevMessageCountRef.current = 0
      prevSessionIdRef.current = selectedSessionId
    }
  }, [selectedSessionId])

  // Scroll to bottom when new messages arrive and followMode is enabled
  const messageCount = selectedSession?.messages.length ?? 0
  useEffect(() => {
    if (
      followMode &&
      messageCount > prevMessageCountRef.current &&
      contentScrollRef.current
    ) {
      // Use setTimeout to ensure DOM is fully rendered before scrolling
      const timeoutId = setTimeout(() => {
        const viewport = contentScrollRef.current?.querySelector(
          '[data-slot="scroll-area-viewport"]'
        )
        if (viewport) {
          viewport.scrollTo({ top: viewport.scrollHeight, behavior: 'smooth' })
        }
      }, 50)
      prevMessageCountRef.current = messageCount
      return () => clearTimeout(timeoutId)
    }
    prevMessageCountRef.current = messageCount
  }, [followMode, messageCount])

  const loadSessions = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const result = await commands.listCodexSessions()
      if (result.status === 'ok') {
        setSessions(result.data)
      } else {
        setError(result.error)
      }
    } catch (err) {
      setError(String(err))
    } finally {
      setLoading(false)
    }
  }, [])

  const loadProviders = useCallback(async () => {
    try {
      const result = await commands.listCodexSessionProviders()
      if (result.status === 'ok') {
        setProviders(result.data)
      }
    } catch (err) {
      console.error('Failed to load codex session providers:', err)
    }
  }, [])

  const loadSessionDetail = useCallback(
    async (path: string, isRefresh = false) => {
      // Don't show loading when refreshing the same session to avoid unmounting ScrollArea
      if (!isRefresh) {
        setDetailLoading(true)
      }
      setSelectedSessionPath(path)
      try {
        const result = await commands.getCodexSessionDetail(path)
        if (result.status === 'ok') {
          setSelectedSession(result.data)
        } else {
          setError(result.error)
        }
      } catch (err) {
        setError(String(err))
      } finally {
        if (!isRefresh) {
          setDetailLoading(false)
        }
      }
    },
    []
  )

  // Store selectedSessionPath in a ref for use in event listener
  const selectedSessionPathRef = useRef<string | null>(null)
  useEffect(() => {
    selectedSessionPathRef.current = selectedSessionPath
  }, [selectedSessionPath])

  useEffect(() => {
    loadSessions()
    loadProviders()
    commands.startCodexSessionsWatcher().catch(err => {
      console.error('Failed to start codex sessions watcher:', err)
    })

    const unlisten = listen('codex-sessions-changed', () => {
      loadSessions()
      // Also refresh current session detail if one is selected
      if (selectedSessionPathRef.current) {
        loadSessionDetail(selectedSessionPathRef.current, true)
      }
    })

    return () => {
      commands.stopCodexSessionsWatcher().catch(err => {
        console.error('Failed to stop codex sessions watcher:', err)
      })
      unlisten.then(fn => fn())
    }
  }, [loadSessions, loadProviders, loadSessionDetail])

  const handleRefresh = () => {
    loadSessions()
  }

  const handleSessionSelect = (session: CodexSessionSummary) => {
    loadSessionDetail(session.path)
  }

  // Resolve a display entry for a provider id, falling back to the raw id.
  const providerEntry = (providerId: string): CodexSessionProvider => {
    const found = providers.find(provider => provider.id === providerId)
    if (found) return found
    return { id: providerId, name: providerId }
  }

  const handleProviderSwitch = async (providerId: string) => {
    if (!selectedSessionPath) return
    if (providerId === selectedSession?.modelProvider) return
    const result = await commands.setCodexSessionProvider(
      selectedSessionPath,
      providerId
    )
    if (result.status === 'ok') {
      loadSessions()
      loadSessionDetail(selectedSessionPath, true)
    } else {
      setError(result.error)
    }
  }

  const handleFollowToggle = () => {
    const newFollowMode = !followMode
    setFollowMode(newFollowMode)
    // Scroll to bottom when enabling follow mode
    if (newFollowMode && contentScrollRef.current) {
      const viewport = contentScrollRef.current.querySelector(
        '[data-slot="scroll-area-viewport"]'
      )
      if (viewport) {
        viewport.scrollTo({ top: viewport.scrollHeight, behavior: 'smooth' })
      }
    }
  }

  const handleExpandThinkingToggle = () => {
    const newExpandThinking = !expandThinking
    setExpandThinking(newExpandThinking)
    localStorage.setItem(
      'codex-sessions-expand-thinking',
      String(newExpandThinking)
    )
  }

  const handleSessionContextMenu = async (
    e: React.MouseEvent,
    session: CodexSessionSummary
  ) => {
    e.preventDefault()
    await showContextMenu([
      {
        id: 'delete',
        label: t('common.delete'),
        action: () => handleDeleteSession(session),
      },
    ])
  }

  const handleDeleteSession = async (session: CodexSessionSummary) => {
    const result = await commands.deleteCodexSession(session.path)
    if (result.status === 'ok') {
      // Clear selection if deleted session was selected
      if (selectedSession?.id === session.id) {
        setSelectedSession(null)
        setSelectedSessionPath(null)
      }
      loadSessions()
    } else {
      setError(result.error)
    }
  }

  const toggleProject = (projectName: string) => {
    setExpandedProjects(prev => {
      const next = new Set(prev)
      if (next.has(projectName)) {
        next.delete(projectName)
      } else {
        next.add(projectName)
      }
      return next
    })
  }

  const formatDate = (timestamp: number) => {
    const date = new Date(timestamp)
    return date.toLocaleDateString(undefined, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    })
  }

  const formatTokens = (tokens: number) => {
    if (tokens >= 1000000) {
      return `${(tokens / 1000000).toFixed(1)}M`
    }
    if (tokens >= 1000) {
      return `${(tokens / 1000).toFixed(1)}K`
    }
    return tokens.toString()
  }

  // Filter empty sessions (0 tokens and no real title source)
  const filteredSessions = hideEmptySessions
    ? sessions.filter(session => {
        const totalTokens =
          (session.tokenUsage?.inputTokens ?? 0) +
          (session.tokenUsage?.outputTokens ?? 0)
        const isNewSession = session.title === 'New Session'
        return !(totalTokens === 0 && isNewSession)
      })
    : sessions

  const groupedSessions = filteredSessions.reduce<
    Record<string, CodexSessionSummary[]>
  >((acc, session) => {
    const projectKey = session.project || 'unknown'
    const projectSessions = acc[projectKey] ?? []
    projectSessions.push(session)
    acc[projectKey] = projectSessions
    return acc
  }, {})

  const groupedProjectNames = Object.keys(groupedSessions).sort((a, b) => {
    const maxA = Math.max(...(groupedSessions[a] ?? []).map(s => s.modifiedAt))
    const maxB = Math.max(...(groupedSessions[b] ?? []).map(s => s.modifiedAt))
    return maxB - maxA
  })

  const renderSessionItem = (
    session: CodexSessionSummary,
    isSelected: boolean
  ) => (
    <button
      key={session.id}
      onClick={() => handleSessionSelect(session)}
      onContextMenu={e => handleSessionContextMenu(e, session)}
      className={cn(
        'w-full text-start p-2 rounded-md hover:bg-accent transition-colors',
        isSelected && 'bg-accent'
      )}
    >
      <div className="font-medium text-sm truncate">{session.title}</div>
      <div className="flex items-center gap-2 text-xs text-muted-foreground mt-1">
        <Clock className="h-3 w-3" />
        {formatDate(session.modifiedAt)}
      </div>
      <div className="flex items-center gap-2 text-xs text-muted-foreground mt-1">
        {projectLabel(session.project) && (
          <span className="truncate">{projectLabel(session.project)}</span>
        )}
        <Badge variant="outline" className="text-xs px-1 py-0">
          {formatTokens(
            (session.tokenUsage?.inputTokens ?? 0) +
              (session.tokenUsage?.outputTokens ?? 0)
          )}{' '}
          tokens
        </Badge>
      </div>
    </button>
  )

  const renderListView = () => (
    <div className="p-2 space-y-1">
      {filteredSessions.map(session =>
        renderSessionItem(session, selectedSession?.id === session.id)
      )}
    </div>
  )

  const renderGroupedView = () => (
    <div className="p-2 space-y-1">
      {groupedProjectNames.map(projectName => {
        const projectSessions = groupedSessions[projectName] || []
        const isExpanded = expandedProjects.has(projectName)

        return (
          <div key={projectName}>
            <button
              onClick={() => toggleProject(projectName)}
              className="w-full flex items-center gap-2 p-2 rounded-md hover:bg-accent transition-colors text-sm"
            >
              {isExpanded ? (
                <ChevronDown className="h-4 w-4" />
              ) : (
                <ChevronRight className="h-4 w-4" />
              )}
              <span className="truncate flex-1 text-start font-medium">
                {projectName === 'unknown'
                  ? t('codex.sessions.unknownProject')
                  : projectLabel(projectName)}
              </span>
              <Badge variant="secondary" className="text-xs">
                {projectSessions.length}
              </Badge>
            </button>
            {isExpanded && (
              <div className="ml-4 space-y-1">
                {projectSessions.map(session =>
                  renderSessionItem(session, selectedSession?.id === session.id)
                )}
              </div>
            )}
          </div>
        )
      })}
    </div>
  )

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between p-4 border-b">
        <h1 className="text-xl font-semibold">{t('codex.sessions.title')}</h1>
        <div className="flex items-center gap-2">
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant={hideEmptySessions ? 'secondary' : 'outline'}
                size="icon"
                className="h-8 w-8"
                onClick={() => {
                  const newValue = !hideEmptySessions
                  setHideEmptySessions(newValue)
                  localStorage.setItem(
                    'codex-sessions-hide-empty',
                    String(newValue)
                  )
                }}
              >
                {hideEmptySessions ? (
                  <EyeOff className="h-4 w-4" />
                ) : (
                  <Eye className="h-4 w-4" />
                )}
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              {hideEmptySessions
                ? t('codex.sessions.hideEmptySessionsOn')
                : t('codex.sessions.hideEmptySessionsOff')}
            </TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="outline"
                size="icon"
                className="h-8 w-8"
                onClick={() =>
                  setViewMode(viewMode === 'list' ? 'grouped' : 'list')
                }
              >
                {viewMode === 'list' ? (
                  <List className="h-4 w-4" />
                ) : (
                  <FolderTree className="h-4 w-4" />
                )}
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              {viewMode === 'list'
                ? t('codex.sessions.listView')
                : t('codex.sessions.groupedView')}
            </TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant={expandThinking ? 'secondary' : 'outline'}
                size="icon"
                className="h-8 w-8"
                onClick={handleExpandThinkingToggle}
              >
                <Brain className="h-4 w-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              {expandThinking
                ? t('codex.sessions.expandThinkingOn')
                : t('codex.sessions.expandThinkingOff')}
            </TooltipContent>
          </Tooltip>
          <Button
            variant="outline"
            size="sm"
            onClick={handleRefresh}
            disabled={loading}
          >
            <RefreshCw
              className={cn('h-4 w-4 mr-2', loading && 'animate-spin')}
            />
            {t('common.refresh')}
          </Button>
        </div>
      </div>

      <ResizablePanelGroup direction="horizontal">
        {/* Sessions List */}
        <ResizablePanel defaultSize={25} minSize={15}>
          <div className="flex flex-col h-full">
            <ScrollArea className="flex-1">
              {loading && sessions.length === 0 ? (
                <div className="flex items-center justify-center p-4 text-muted-foreground">
                  {t('common.loading')}
                </div>
              ) : error ? (
                <div className="p-4 text-destructive text-sm">{error}</div>
              ) : sessions.length === 0 ? (
                <div className="flex flex-col items-center justify-center p-4 text-muted-foreground text-sm">
                  <MessageSquare className="h-8 w-8 mb-2 opacity-50" />
                  <p>{t('codex.sessions.noSessions')}</p>
                </div>
              ) : viewMode === 'list' ? (
                renderListView()
              ) : (
                renderGroupedView()
              )}
            </ScrollArea>
          </div>
        </ResizablePanel>

        <ResizableHandle />

        {/* Session Detail */}
        <ResizablePanel defaultSize={75} minSize={30}>
          <div className="flex flex-col h-full min-w-0">
            {detailLoading ? (
              <div className="flex items-center justify-center h-full text-muted-foreground">
                {t('common.loading')}
              </div>
            ) : selectedSession ? (
              <>
                <div className="p-4 border-b flex items-start justify-between gap-4">
                  <div className="min-w-0 flex-1">
                    <h2 className="font-medium truncate">
                      {selectedSession.title}
                    </h2>
                    <div className="flex items-center gap-4 text-xs text-muted-foreground mt-1">
                      <span>{formatDate(selectedSession.modifiedAt)}</span>
                      <span>{selectedSession.model}</span>
                      <span>
                        {formatTokens(
                          selectedSession.tokenUsage.inputTokens +
                            selectedSession.tokenUsage.outputTokens
                        )}{' '}
                        tokens
                      </span>
                    </div>
                    {selectedSession.cwd && (
                      <div className="text-xs text-muted-foreground mt-1 truncate">
                        {selectedSession.cwd}
                      </div>
                    )}
                  </div>
                  <div className="flex items-center gap-2 shrink-0">
                    <DropdownMenu>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <DropdownMenuTrigger asChild>
                            <Button
                              variant="outline"
                              size="sm"
                              className="h-8 gap-1.5"
                              disabled={providers.length === 0}
                              title={t('codex.sessions.switchProvider')}
                            >
                              <Server className="h-3.5 w-3.5" />
                              <span className="max-w-[10rem] truncate">
                                {selectedSession.modelProvider
                                  ? providerEntry(selectedSession.modelProvider)
                                      .name
                                  : t('codex.sessions.unknownProvider')}
                              </span>
                            </Button>
                          </DropdownMenuTrigger>
                        </TooltipTrigger>
                        <TooltipContent>
                          {t('codex.sessions.switchProvider')}
                        </TooltipContent>
                      </Tooltip>
                      <DropdownMenuContent align="end">
                        <DropdownMenuLabel>
                          {t('codex.sessions.switchProvider')}
                        </DropdownMenuLabel>
                        <DropdownMenuSeparator />
                        {providers.map(provider => (
                          <DropdownMenuItem
                            key={provider.id}
                            onClick={() => handleProviderSwitch(provider.id)}
                          >
                            <span className="flex-1 truncate">
                              {provider.name}
                            </span>
                            {provider.id === selectedSession.modelProvider && (
                              <Check className="h-4 w-4 ml-2" />
                            )}
                          </DropdownMenuItem>
                        ))}
                      </DropdownMenuContent>
                    </DropdownMenu>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Button
                          variant={followMode ? 'default' : 'outline'}
                          size="icon"
                          className="h-8 w-8 shrink-0"
                          onClick={handleFollowToggle}
                        >
                          <ArrowDownToLine className="h-4 w-4" />
                        </Button>
                      </TooltipTrigger>
                      <TooltipContent>
                        {followMode
                          ? t('codex.sessions.followModeOn')
                          : t('codex.sessions.followModeOff')}
                      </TooltipContent>
                    </Tooltip>
                  </div>
                </div>
                <ScrollArea ref={contentScrollRef} className="flex-1 min-w-0">
                  <div className="p-4 space-y-4">
                    {selectedSession.messages.map(message => (
                      <div
                        key={message.id}
                        className={cn(
                          'flex gap-3',
                          message.role === 'user'
                            ? 'justify-end'
                            : 'justify-start'
                        )}
                      >
                        {message.role === 'assistant' && (
                          <div className="shrink-0 w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center">
                            <Bot className="h-4 w-4 text-primary" />
                          </div>
                        )}
                        <div
                          className={cn(
                            'max-w-[80%] rounded-lg p-3',
                            message.role === 'user'
                              ? 'bg-primary text-primary-foreground'
                              : 'bg-muted'
                          )}
                        >
                          {message.content.map((block, idx) => (
                            <div key={idx} className="select-text">
                              {block.type === 'thinking' && block.thinking ? (
                                <details
                                  open={expandThinking}
                                  className="text-xs opacity-70 mb-2"
                                >
                                  <summary className="cursor-pointer">
                                    {t('codex.sessions.thinking')}
                                  </summary>
                                  <div className="mt-1 pl-2 border-l-2 border-muted-foreground/30">
                                    {block.thinking}
                                  </div>
                                </details>
                              ) : block.text ? (
                                message.role === 'user' ? (
                                  <div className="whitespace-pre-wrap text-sm">
                                    {block.text}
                                  </div>
                                ) : (
                                  <Streamdown shikiTheme={shikiTheme}>
                                    {block.text}
                                  </Streamdown>
                                )
                              ) : null}
                            </div>
                          ))}
                        </div>
                        {message.role === 'user' && (
                          <div className="shrink-0 w-8 h-8 rounded-full bg-primary flex items-center justify-center">
                            <User className="h-4 w-4 text-primary-foreground" />
                          </div>
                        )}
                      </div>
                    ))}
                  </div>
                </ScrollArea>
              </>
            ) : (
              <div className="flex items-center justify-center h-full text-muted-foreground">
                <p>{t('codex.sessions.selectSessionHint')}</p>
              </div>
            )}
          </div>
        </ResizablePanel>
      </ResizablePanelGroup>
    </div>
  )
}
