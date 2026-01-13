import {
  useEffect,
  useRef,
  useCallback,
  useState,
  forwardRef,
  useImperativeHandle,
} from 'react'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { WebLinksAddon } from '@xterm/addon-web-links'
import '@xterm/xterm/css/xterm.css'
import { spawn, type IPty } from 'tauri-pty'
import { useTheme } from '@/hooks/use-theme'
import { platform } from '@tauri-apps/plugin-os'
import { usePreferences } from '@/services/preferences'

// Default fallback fonts for terminal
const DEFAULT_TERMINAL_FONTS = 'Menlo, Monaco, "Courier New", monospace'

interface TerminalViewProps {
  terminalId: string
  cwd?: string
  forceDark?: boolean
  onExit?: (exitCode: number) => void
}

export interface TerminalViewRef {
  focus: () => void
}

export const TerminalView = forwardRef<TerminalViewRef, TerminalViewProps>(
  function TerminalView({ cwd, forceDark, onExit }, ref) {
    const containerRef = useRef<HTMLDivElement>(null)
    const terminalRef = useRef<Terminal | null>(null)
    const fitAddonRef = useRef<FitAddon | null>(null)
    const ptyRef = useRef<IPty | null>(null)
    const onExitRef = useRef(onExit)
    const initialCwdRef = useRef(cwd)
    const initialForceDarkRef = useRef(forceDark)
    const isInitializedRef = useRef(false)
    const { theme } = useTheme()
    const initialThemeRef = useRef(theme)
    const { data: preferences } = usePreferences()
    const initialFontFamilyRef = useRef<string | null | undefined>(undefined)

    // Capture initial font family from preferences (only once when preferences first loads)
    useEffect(() => {
      if (
        initialFontFamilyRef.current === undefined &&
        preferences !== undefined
      ) {
        initialFontFamilyRef.current = preferences.terminal_font_family ?? null
      }
    }, [preferences])

    // Expose focus method to parent
    useImperativeHandle(ref, () => ({
      focus: () => {
        terminalRef.current?.focus()
      },
    }))

    // Keep onExit ref updated
    useEffect(() => {
      onExitRef.current = onExit
    }, [onExit])

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

    const isDark =
      forceDark || theme === 'dark' || (theme === 'system' && systemPrefersDark)

    const getThemeColors = useCallback(() => {
      return {
        background: isDark ? '#1e1e1e' : '#ffffff',
        foreground: isDark ? '#d4d4d4' : '#1e1e1e',
        cursor: isDark ? '#d4d4d4' : '#1e1e1e',
        cursorAccent: isDark ? '#1e1e1e' : '#ffffff',
        selectionBackground: isDark
          ? 'rgba(255, 255, 255, 0.3)'
          : 'rgba(0, 0, 0, 0.3)',
      }
    }, [isDark])

    // Initialize terminal only once when component mounts
    useEffect(() => {
      // Skip if already initialized
      if (isInitializedRef.current) return
      if (!containerRef.current) return
      isInitializedRef.current = true

      // Build font family string: user preference first, then fallbacks
      const fontFamily = initialFontFamilyRef.current
        ? `"${initialFontFamilyRef.current}", ${DEFAULT_TERMINAL_FONTS}`
        : DEFAULT_TERMINAL_FONTS

      // Compute theme colors at initialization time
      const currentIsDark =
        initialForceDarkRef.current ||
        initialThemeRef.current === 'dark' ||
        (initialThemeRef.current === 'system' &&
          window.matchMedia('(prefers-color-scheme: dark)').matches)
      const themeColors = {
        background: currentIsDark ? '#1e1e1e' : '#ffffff',
        foreground: currentIsDark ? '#d4d4d4' : '#1e1e1e',
        cursor: currentIsDark ? '#d4d4d4' : '#1e1e1e',
        cursorAccent: currentIsDark ? '#1e1e1e' : '#ffffff',
        selectionBackground: currentIsDark
          ? 'rgba(255, 255, 255, 0.3)'
          : 'rgba(0, 0, 0, 0.3)',
      }

      const terminal = new Terminal({
        cursorBlink: true,
        fontSize: 14,
        fontFamily,
        theme: themeColors,
        allowProposedApi: true,
        scrollback: 10000,
      })

      const fitAddon = new FitAddon()
      const webLinksAddon = new WebLinksAddon()

      terminal.loadAddon(fitAddon)
      terminal.loadAddon(webLinksAddon)

      terminal.open(containerRef.current)
      fitAddon.fit()

      terminalRef.current = terminal
      fitAddonRef.current = fitAddon

      // Determine shell based on platform
      const currentPlatform = platform()
      const shell =
        currentPlatform === 'windows' ? 'powershell.exe' : '/bin/zsh'

      // Get initial dimensions
      const dims = fitAddon.proposeDimensions()
      const cols = dims?.cols || 80
      const rows = dims?.rows || 24

      // Spawn PTY using tauri-pty with initial cwd from ref
      const pty = spawn(shell, [], {
        cols,
        rows,
        cwd: initialCwdRef.current || undefined,
        env: {
          TERM: 'xterm-256color',
          COLORTERM: 'truecolor',
        },
      })

      ptyRef.current = pty

      // Connect PTY output to terminal
      pty.onData(data => {
        terminal.write(data)
      })

      // Connect terminal input to PTY
      terminal.onData(data => {
        pty.write(data)
      })

      // Handle PTY exit
      pty.onExit(({ exitCode }) => {
        terminal.write(`\r\n[Process exited with code ${exitCode}]\r\n`)
        onExitRef.current?.(exitCode)
      })

      // Handle resize
      const container = containerRef.current
      const resizeObserver = new ResizeObserver(() => {
        if (fitAddonRef.current && ptyRef.current) {
          fitAddonRef.current.fit()
          const newDims = fitAddonRef.current.proposeDimensions()
          if (newDims) {
            ptyRef.current.resize(newDims.cols, newDims.rows)
          }
        }
      })

      resizeObserver.observe(container)

      // Initial resize after a short delay
      setTimeout(() => {
        if (fitAddonRef.current && ptyRef.current) {
          fitAddonRef.current.fit()
          const newDims = fitAddonRef.current.proposeDimensions()
          if (newDims) {
            ptyRef.current.resize(newDims.cols, newDims.rows)
          }
        }
      }, 100)

      return () => {
        resizeObserver.disconnect()
        pty.kill()
        terminal.dispose()
        terminalRef.current = null
        fitAddonRef.current = null
        ptyRef.current = null
        isInitializedRef.current = false
      }
    }, [])

    // Update theme when it changes
    useEffect(() => {
      if (terminalRef.current) {
        terminalRef.current.options.theme = getThemeColors()
      }
    }, [getThemeColors])

    return (
      <div
        ref={containerRef}
        className="h-full w-full"
        style={{
          padding: '8px',
          backgroundColor: isDark ? '#1e1e1e' : '#ffffff',
        }}
      />
    )
  }
)
