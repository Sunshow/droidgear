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
import { cn } from '@/lib/utils'
import {
  commands,
  type DshSessionMessage,
  type DshSessionSummary,
  type DshSessionDetail,
} from '@/lib/bindings'
import { useTheme } from '@/hooks/use-theme'
import { showContextMenu } from '@/lib/context-menu'

type ViewMode = 'list' | 'grouped'

function projectLabel(project: string): string {
  if (!project) return ''
  const parts = project.replace(/\\/g, '/').split('/').filter(Boolean)
  return parts[parts.length - 1] ?? project
}

type RenderItem =
  | { kind: 'message'; message: DshSessionMessage }
  | { kind: 'thinking'; id: string; texts: string[] }

/** Thinking texts when a message renders as nothing but reasoning, else null. */
function thinkingOnlyTexts(message: DshSessionMessage): string[] | null {
  if (message.role !== 'assistant' || message.content.length === 0) {
    return null
  }
  const texts: string[] = []
  for (const block of message.content) {
    if (block.type !== 'thinking' || !block.thinking) {
      return null
    }
    texts.push(block.thinking)
  }
  return texts
}

/**
 * Merge runs of reasoning-only assistant messages into one collapsible group.
 * Dsh writes one assistant message per step, and a step that only calls tools
 * renders as nothing but reasoning — without grouping the transcript becomes a
 * long ladder of separate "Thinking" boxes.
 */
function groupConsecutiveThinking(messages: DshSessionMessage[]): RenderItem[] {
  const items: RenderItem[] = []
  let pending: { id: string; texts: string[] } | null = null

  const flush = () => {
    if (pending) {
      items.push({ kind: 'thinking', ...pending })
      pending = null
    }
  }

  for (const message of messages) {
    const texts = thinkingOnlyTexts(message)
    if (texts) {
      if (pending) {
        pending.texts.push(...texts)
      } else {
        pending = { id: message.id, texts: [...texts] }
      }
    } else {
      flush()
      items.push({ kind: 'message', message })
    }
  }
  flush()

  return items
}

export function DshSessionsPage() {
  const { t } = useTranslation()
  const { theme } = useTheme()
  const [sessions, setSessions] = useState<DshSessionSummary[]>([])
  const [selectedSession, setSelectedSession] =
    useState<DshSessionDetail | null>(null)
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
    return localStorage.getItem('dsh-sessions-expand-thinking') === 'true'
  })
  const [hideEmptySessions, setHideEmptySessions] = useState(() => {
    return localStorage.getItem('dsh-sessions-hide-empty') !== 'false'
  })
  const [systemPrefersDark, setSystemPrefersDark] = useState(
    () => window.matchMedia('(prefers-color-scheme: dark)').matches
  )

  useEffect(() => {
    if (theme !== 'system') return
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    const handleChange = (event: MediaQueryListEvent) => {
      setSystemPrefersDark(event.matches)
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
  const selectedSessionPathRef = useRef<string | null>(null)

  useEffect(() => {
    selectedSessionPathRef.current = selectedSessionPath
  }, [selectedSessionPath])

  const selectedSessionId = selectedSession?.summary.id
  useEffect(() => {
    if (selectedSessionId && selectedSessionId !== prevSessionIdRef.current) {
      const viewport = contentScrollRef.current?.querySelector(
        '[data-slot="scroll-area-viewport"]'
      )
      viewport?.scrollTo(0, 0)
      prevMessageCountRef.current = 0
      prevSessionIdRef.current = selectedSessionId
    }
  }, [selectedSessionId])

  const messageCount = selectedSession?.messages.length ?? 0
  useEffect(() => {
    if (
      followMode &&
      messageCount > prevMessageCountRef.current &&
      contentScrollRef.current
    ) {
      const timeoutId = setTimeout(() => {
        const viewport = contentScrollRef.current?.querySelector(
          '[data-slot="scroll-area-viewport"]'
        )
        viewport?.scrollTo({ top: viewport.scrollHeight, behavior: 'smooth' })
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
      const result = await commands.listDshSessions()
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

  const loadSessionDetail = useCallback(
    async (path: string, isRefresh = false) => {
      if (!isRefresh) setDetailLoading(true)
      setSelectedSessionPath(path)
      try {
        const result = await commands.getDshSessionDetail(path)
        if (result.status === 'ok') {
          setSelectedSession(result.data)
        } else {
          setError(result.error)
        }
      } catch (err) {
        setError(String(err))
      } finally {
        if (!isRefresh) setDetailLoading(false)
      }
    },
    []
  )

  useEffect(() => {
    loadSessions()
    commands.startDshSessionsWatcher().catch(err => {
      console.error('Failed to start Dsh sessions watcher:', err)
    })

    const unlisten = listen('dsh-sessions-changed', () => {
      loadSessions()
      if (selectedSessionPathRef.current) {
        loadSessionDetail(selectedSessionPathRef.current, true)
      }
    })

    return () => {
      commands.stopDshSessionsWatcher().catch(err => {
        console.error('Failed to stop Dsh sessions watcher:', err)
      })
      unlisten.then(unlistenFn => unlistenFn())
    }
  }, [loadSessions, loadSessionDetail])

  const handleFollowToggle = () => {
    const nextFollowMode = !followMode
    setFollowMode(nextFollowMode)
    if (nextFollowMode) {
      const viewport = contentScrollRef.current?.querySelector(
        '[data-slot="scroll-area-viewport"]'
      )
      viewport?.scrollTo({ top: viewport.scrollHeight, behavior: 'smooth' })
    }
  }

  const handleDeleteSession = async (session: DshSessionSummary) => {
    const result = await commands.deleteDshSession(session.path)
    if (result.status === 'ok') {
      if (selectedSession?.summary.id === session.id) {
        setSelectedSession(null)
        setSelectedSessionPath(null)
      }
      loadSessions()
    } else {
      setError(result.error)
    }
  }

  const handleSessionContextMenu = async (
    event: React.MouseEvent,
    session: DshSessionSummary
  ) => {
    event.preventDefault()
    await showContextMenu([
      {
        id: 'delete',
        label: t('common.delete'),
        action: () => handleDeleteSession(session),
      },
    ])
  }

  const formatDate = (timestamp: number) =>
    new Date(timestamp).toLocaleDateString(undefined, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    })

  const formatTokens = (tokens: number) => {
    if (tokens >= 1000000) return `${(tokens / 1000000).toFixed(1)}M`
    if (tokens >= 1000) return `${(tokens / 1000).toFixed(1)}K`
    return tokens.toString()
  }

  const filteredSessions = hideEmptySessions
    ? sessions.filter(session => {
        const totalTokens =
          (session.tokenUsage?.inputTokens ?? 0) +
          (session.tokenUsage?.outputTokens ?? 0)
        return !(totalTokens === 0 && session.title === 'New Session')
      })
    : sessions

  const groupedSessions = filteredSessions.reduce<
    Record<string, DshSessionSummary[]>
  >((acc, session) => {
    const projectKey = session.project || 'unknown'
    ;(acc[projectKey] ??= []).push(session)
    return acc
  }, {})

  const groupedProjectNames = Object.keys(groupedSessions).sort((a, b) => {
    const maxA = Math.max(...(groupedSessions[a] ?? []).map(s => s.modifiedAt))
    const maxB = Math.max(...(groupedSessions[b] ?? []).map(s => s.modifiedAt))
    return maxB - maxA
  })

  const renderSessionItem = (
    session: DshSessionSummary,
    isSelected: boolean
  ) => (
    <button
      key={session.id}
      onClick={() => loadSessionDetail(session.path)}
      onContextMenu={event => handleSessionContextMenu(event, session)}
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

  const renderGroupedView = () => (
    <div className="p-2 space-y-1">
      {groupedProjectNames.map(projectName => {
        const projectSessions = groupedSessions[projectName] ?? []
        const isExpanded = expandedProjects.has(projectName)
        return (
          <div key={projectName}>
            <button
              onClick={() =>
                setExpandedProjects(previous => {
                  const next = new Set(previous)
                  if (next.has(projectName)) next.delete(projectName)
                  else next.add(projectName)
                  return next
                })
              }
              className="w-full flex items-center gap-2 p-2 rounded-md hover:bg-accent transition-colors text-sm"
            >
              {isExpanded ? (
                <ChevronDown className="h-4 w-4" />
              ) : (
                <ChevronRight className="h-4 w-4" />
              )}
              <span className="truncate flex-1 text-start font-medium">
                {projectName === 'unknown'
                  ? t('dsh.sessions.unknownProject')
                  : projectLabel(projectName)}
              </span>
              <Badge variant="secondary" className="text-xs">
                {projectSessions.length}
              </Badge>
            </button>
            {isExpanded && (
              <div className="ml-4 space-y-1">
                {projectSessions.map(session =>
                  renderSessionItem(
                    session,
                    selectedSession?.summary.id === session.id
                  )
                )}
              </div>
            )}
          </div>
        )
      })}
    </div>
  )

  const summary = selectedSession?.summary
  const renderItems = selectedSession
    ? groupConsecutiveThinking(selectedSession.messages)
    : []

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between p-4 border-b">
        <h1 className="text-xl font-semibold">{t('dsh.sessions.title')}</h1>
        <div className="flex items-center gap-2">
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant={hideEmptySessions ? 'secondary' : 'outline'}
                size="icon"
                className="h-8 w-8"
                onClick={() => {
                  const next = !hideEmptySessions
                  setHideEmptySessions(next)
                  localStorage.setItem('dsh-sessions-hide-empty', String(next))
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
                ? t('dsh.sessions.hideEmptySessionsOn')
                : t('dsh.sessions.hideEmptySessionsOff')}
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
                ? t('dsh.sessions.listView')
                : t('dsh.sessions.groupedView')}
            </TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant={expandThinking ? 'secondary' : 'outline'}
                size="icon"
                className="h-8 w-8"
                onClick={() => {
                  const next = !expandThinking
                  setExpandThinking(next)
                  localStorage.setItem(
                    'dsh-sessions-expand-thinking',
                    String(next)
                  )
                }}
              >
                <Brain className="h-4 w-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              {expandThinking
                ? t('dsh.sessions.expandThinkingOn')
                : t('dsh.sessions.expandThinkingOff')}
            </TooltipContent>
          </Tooltip>
          <Button
            variant="outline"
            size="sm"
            onClick={loadSessions}
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
        <ResizablePanel defaultSize={25} minSize={15}>
          <ScrollArea className="h-full">
            {loading && sessions.length === 0 ? (
              <div className="flex items-center justify-center p-4 text-muted-foreground">
                {t('common.loading')}
              </div>
            ) : error ? (
              <div className="p-4 text-destructive text-sm">{error}</div>
            ) : filteredSessions.length === 0 ? (
              <div className="flex flex-col items-center justify-center p-4 text-muted-foreground text-sm">
                <MessageSquare className="h-8 w-8 mb-2 opacity-50" />
                <p>{t('dsh.sessions.noSessions')}</p>
              </div>
            ) : viewMode === 'list' ? (
              <div className="p-2 space-y-1">
                {filteredSessions.map(session =>
                  renderSessionItem(
                    session,
                    selectedSession?.summary.id === session.id
                  )
                )}
              </div>
            ) : (
              renderGroupedView()
            )}
          </ScrollArea>
        </ResizablePanel>

        <ResizableHandle />

        <ResizablePanel defaultSize={75} minSize={30}>
          <div className="flex flex-col h-full min-w-0">
            {detailLoading ? (
              <div className="flex items-center justify-center h-full text-muted-foreground">
                {t('common.loading')}
              </div>
            ) : summary ? (
              <>
                <div className="p-4 border-b flex items-start justify-between gap-4">
                  <div className="min-w-0 flex-1">
                    <h2 className="font-medium truncate">{summary.title}</h2>
                    <div className="flex items-center gap-4 text-xs text-muted-foreground mt-1">
                      <span>{formatDate(summary.modifiedAt)}</span>
                      <span>
                        {summary.modelProvider
                          ? `${summary.modelProvider}/`
                          : ''}
                        {summary.model}
                      </span>
                      <span>
                        {formatTokens(
                          summary.tokenUsage.inputTokens +
                            summary.tokenUsage.outputTokens
                        )}{' '}
                        tokens
                      </span>
                    </div>
                    {summary.project && (
                      <div className="text-xs text-muted-foreground mt-1 truncate">
                        {summary.project}
                      </div>
                    )}
                  </div>
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
                        ? t('dsh.sessions.followModeOn')
                        : t('dsh.sessions.followModeOff')}
                    </TooltipContent>
                  </Tooltip>
                </div>
                <ScrollArea ref={contentScrollRef} className="flex-1 min-w-0">
                  <div className="p-4 space-y-4">
                    {renderItems.map(item => {
                      if (item.kind === 'thinking') {
                        return (
                          <div
                            key={`thinking-${item.id}`}
                            className="flex gap-3 justify-start"
                          >
                            <div className="shrink-0 w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center">
                              <Bot className="h-4 w-4 text-primary" />
                            </div>
                            <div className="max-w-[80%] rounded-lg p-3 bg-muted">
                              <details
                                open={expandThinking}
                                className="text-xs opacity-70"
                              >
                                <summary className="cursor-pointer select-none">
                                  {t('dsh.sessions.thinkingGroup', {
                                    count: item.texts.length,
                                  })}
                                </summary>
                                <div className="mt-2 space-y-2">
                                  {item.texts.map((text, index) => (
                                    <div
                                      key={index}
                                      className="whitespace-pre-wrap pl-2 border-l-2 border-muted-foreground/30"
                                    >
                                      {text}
                                    </div>
                                  ))}
                                </div>
                              </details>
                            </div>
                          </div>
                        )
                      }
                      const message = item.message
                      return (
                        <div
                          key={message.id}
                          className={cn(
                            'flex gap-3',
                            message.role === 'user'
                              ? 'justify-end'
                              : message.role === 'assistant'
                                ? 'justify-start'
                                : 'justify-center'
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
                                : message.role === 'system'
                                  ? 'bg-muted/70 text-muted-foreground text-sm'
                                  : 'bg-muted'
                            )}
                          >
                            {message.content.map((block, index) => (
                              <div key={index} className="select-text">
                                {block.type === 'thinking' && block.thinking ? (
                                  <details
                                    open={expandThinking}
                                    className="text-xs opacity-70 mb-2"
                                  >
                                    <summary className="cursor-pointer">
                                      {t('dsh.sessions.thinking')}
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
                      )
                    })}
                  </div>
                </ScrollArea>
              </>
            ) : (
              <div className="flex items-center justify-center h-full text-muted-foreground">
                <p>{t('dsh.sessions.selectSessionHint')}</p>
              </div>
            )}
          </div>
        </ResizablePanel>
      </ResizablePanelGroup>
    </div>
  )
}
