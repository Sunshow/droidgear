import { cn } from '@/lib/utils'
import {
  TitleBarLeftActions,
  TitleBarRightActions,
  TitleBarTitle,
} from './TitleBarContent'
import { WindowsWindowControls } from './WindowsWindowControls'

interface LinuxTitleBarProps {
  className?: string
  title?: string
}

/**
 * Linux title bar with custom window controls.
 *
 * On Linux, native window decorations are NOT used (decorations: false in config)
 * to work around a WebKitGTK bug where native close/minimize/maximize buttons
 * become unclickable in non-maximized state. Instead, we use a custom title bar
 * with built-in window controls, matching the Windows approach.
 */
export function LinuxTitleBar({ className, title }: LinuxTitleBarProps) {
  return (
    <div
      data-tauri-drag-region
      className={cn(
        'relative flex h-8 w-full shrink-0 items-center justify-between border-b bg-background',
        className
      )}
    >
      {/* Left side - Actions */}
      <div className="flex items-center pl-2">
        <TitleBarLeftActions />
      </div>

      {/* Center - Title */}
      <TitleBarTitle title={title} />

      {/* Right side - Actions + Window Controls */}
      <div className="flex items-center">
        <TitleBarRightActions />
        <WindowsWindowControls />
      </div>
    </div>
  )
}
