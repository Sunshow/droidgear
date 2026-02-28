import {
  ResizablePanelGroup,
  ResizablePanel,
  ResizableHandle,
} from '@/components/ui/resizable'
import { TitleBar } from '@/components/titlebar/TitleBar'
import { LeftSideBar } from './LeftSideBar'
import { RightSideBar } from './RightSideBar'
import { MainWindowContent } from './MainWindowContent'
import { CommandPalette } from '@/components/command-palette/CommandPalette'
import { PreferencesDialog } from '@/components/preferences/PreferencesDialog'
import { Toaster } from '@/components/ui/sonner'
import { useUIStore } from '@/store/ui-store'
import { useMainWindowEventListeners } from '@/hooks/useMainWindowEventListeners'
import { cn } from '@/lib/utils'

/**
 * Layout sizing configuration for resizable panels.
 * All values are percentages of total width.
 */
const LAYOUT = {
  rightSidebar: { default: 20, min: 15, max: 40 },
  main: { min: 30 },
} as const

// Main content default when right sidebar is visible
const MAIN_CONTENT_DEFAULT = 100 - LAYOUT.rightSidebar.default

export function MainWindow() {
  const leftSidebarVisible = useUIStore(state => state.leftSidebarVisible)
  const rightSidebarVisible = useUIStore(state => state.rightSidebarVisible)

  // Set up global event listeners (keyboard shortcuts, etc.)
  useMainWindowEventListeners()

  return (
    <div className="flex h-screen w-full flex-col overflow-hidden bg-background">
      <TitleBar />

      <div className="flex flex-1 overflow-hidden">
        {/* Left sidebar with fixed width */}
        <div
          className={cn('w-[280px] shrink-0', !leftSidebarVisible && 'hidden')}
        >
          <LeftSideBar />
        </div>

        <ResizablePanelGroup direction="horizontal">
          <ResizablePanel
            defaultSize={MAIN_CONTENT_DEFAULT}
            minSize={LAYOUT.main.min}
          >
            <MainWindowContent />
          </ResizablePanel>

          <ResizableHandle className={cn(!rightSidebarVisible && 'hidden')} />

          <ResizablePanel
            defaultSize={LAYOUT.rightSidebar.default}
            minSize={LAYOUT.rightSidebar.min}
            maxSize={LAYOUT.rightSidebar.max}
            className={cn(!rightSidebarVisible && 'hidden')}
          >
            <RightSideBar />
          </ResizablePanel>
        </ResizablePanelGroup>
      </div>

      {/* Global UI Components (hidden until triggered) */}
      <CommandPalette />
      <PreferencesDialog />
      <Toaster position="bottom-right" />
    </div>
  )
}
