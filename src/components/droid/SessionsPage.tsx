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
} from 'lucide-react'
import { Streamdown } from 'streamdown'
import { listen } from '@tauri-apps/api/event'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { Label } from '@/components/ui/label'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Badge } from '@/components/ui/badge'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { cn } from '@/lib/utils'
import {
  commands,
  type SessionSummary,
  type SessionDetail,
  type SessionProject,
} from '@/lib/bindings'
import { useTheme } from '@/hooks/use-theme'

type ViewMode = 'list' | 'grouped'

export function SessionsPage() {
  const { t } = useTranslation()
  const { theme } = useTheme()
  const [sessions, setSessions] = useState<SessionSummary[]>([])
  const [projects, setProjects] = useState<SessionProject[]>([])
  const [selectedSession, setSelectedSession] = useState<SessionDetail | null>(
    null
  )
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
    const saved = localStorage.getItem('sessions-expand-thinking')
    return saved === 'true'
  })
  const [hideEmptySessions, setHideEmptySessions] = useState(() => {
    const saved = localStorage.getItem('sessions-hide-empty')
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
      const [sessionsResult, projectsResult] = await Promise.all([
        commands.listSessions(null),
        commands.listSessionProjects(),
      ])

      if (sessionsResult.status === 'ok') {
        setSessions(sessionsResult.data)
      } else {
        setError(sessionsResult.error)
      }

      if (projectsResult.status === 'ok') {
        setProjects(projectsResult.data)
      }
    } catch (err) {
      setError(String(err))
    } finally {
      setLoading(false)
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
        const result = await commands.getSessionDetail(path)
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
    commands.startSessionsWatcher().catch(err => {
      console.error('Failed to start sessions watcher:', err)
    })

    const unlisten = listen('sessions-changed', () => {
      loadSessions()
      // Also refresh current session detail if one is selected
      if (selectedSessionPathRef.current) {
        loadSessionDetail(selectedSessionPathRef.current, true)
      }
    })

    return () => {
      commands.stopSessionsWatcher().catch(err => {
        console.error('Failed to stop sessions watcher:', err)
      })
      unlisten.then(fn => fn())
    }
  }, [loadSessions, loadSessionDetail])

  const handleRefresh = () => {
    loadSessions()
  }

  const handleSessionSelect = (session: SessionSummary) => {
    loadSessionDetail(session.path)
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
    localStorage.setItem('sessions-expand-thinking', String(newExpandThinking))
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

  // Filter empty sessions (0 tokens and named "New Session")
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
    Record<string, SessionSummary[]>
  >((acc, session) => {
    const projectKey = session.project
    const projectSessions = acc[projectKey] ?? []
    projectSessions.push(session)
    acc[projectKey] = projectSessions
    return acc
  }, {})

  const renderSessionItem = (session: SessionSummary, isSelected: boolean) => (
    <button
      key={session.id}
      onClick={() => handleSessionSelect(session)}
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
      {projects.map(project => {
        const projectSessions = groupedSessions[project.name] || []
        const isExpanded = expandedProjects.has(project.name)

        return (
          <div key={project.name}>
            <button
              onClick={() => toggleProject(project.name)}
              className="w-full flex items-center gap-2 p-2 rounded-md hover:bg-accent transition-colors text-sm"
            >
              {isExpanded ? (
                <ChevronDown className="h-4 w-4" />
              ) : (
                <ChevronRight className="h-4 w-4" />
              )}
              <span className="truncate flex-1 text-start font-medium">
                {project.name}
              </span>
              <Badge variant="secondary" className="text-xs">
                {project.sessionCount}
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
        <h1 className="text-xl font-semibold">{t('droid.sessions.title')}</h1>
        <div className="flex items-center gap-2">
          <div className="flex items-center gap-2 mr-2">
            <Checkbox
              id="hide-empty"
              checked={hideEmptySessions}
              onCheckedChange={checked => {
                setHideEmptySessions(!!checked)
                localStorage.setItem('sessions-hide-empty', String(!!checked))
              }}
            />
            <Label
              htmlFor="hide-empty"
              className="text-sm cursor-pointer whitespace-nowrap"
            >
              {t('droid.sessions.hideEmptySessions')}
            </Label>
          </div>
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant={viewMode === 'list' ? 'secondary' : 'outline'}
                size="icon"
                className="h-8 w-8"
                onClick={() => setViewMode('list')}
              >
                <List className="h-4 w-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>{t('droid.sessions.listView')}</TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant={viewMode === 'grouped' ? 'secondary' : 'outline'}
                size="icon"
                className="h-8 w-8"
                onClick={() => setViewMode('grouped')}
              >
                <FolderTree className="h-4 w-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>{t('droid.sessions.groupedView')}</TooltipContent>
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
                ? t('droid.sessions.expandThinkingOn')
                : t('droid.sessions.expandThinkingOff')}
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

      <div className="flex flex-1 overflow-hidden">
        {/* Sessions List */}
        <div className="w-72 border-r flex flex-col">
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
                <p>{t('droid.sessions.noSessions')}</p>
              </div>
            ) : viewMode === 'list' ? (
              renderListView()
            ) : (
              renderGroupedView()
            )}
          </ScrollArea>
        </div>

        {/* Session Detail */}
        <div className="flex-1 flex flex-col min-w-0">
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
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant={followMode ? 'default' : 'outline'}
                      size="icon"
                      className="h-8 w-8 flex-shrink-0"
                      onClick={handleFollowToggle}
                    >
                      <ArrowDownToLine className="h-4 w-4" />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>
                    {followMode
                      ? t('droid.sessions.followModeOn')
                      : t('droid.sessions.followModeOff')}
                  </TooltipContent>
                </Tooltip>
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
                        <div className="flex-shrink-0 w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center">
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
                                  {t('droid.sessions.thinking')}
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
                        <div className="flex-shrink-0 w-8 h-8 rounded-full bg-primary flex items-center justify-center">
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
              <p>{t('droid.sessions.selectSessionHint')}</p>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
