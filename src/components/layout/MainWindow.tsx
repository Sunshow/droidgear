import { TitleBar } from '@/components/titlebar/TitleBar'
import { LeftSideBar } from './LeftSideBar'
import { MainWindowContent } from './MainWindowContent'
import { CommandPalette } from '@/components/command-palette/CommandPalette'
import { PreferencesDialog } from '@/components/preferences/PreferencesDialog'
import { Toaster } from '@/components/ui/sonner'
import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { Button } from '@/components/ui/button'
import { useUIStore } from '@/store/ui-store'
import { useMainWindowEventListeners } from '@/hooks/useMainWindowEventListeners'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { cn } from '@/lib/utils'

export function MainWindow() {
  const leftSidebarVisible = useUIStore(state => state.leftSidebarVisible)
  const closeConfirmOpen = useUIStore(state => state.closeConfirmOpen)

  // Set up global event listeners (keyboard shortcuts, etc.)
  useMainWindowEventListeners()

  const handleConfirmClose = async () => {
    useUIStore.getState().setCloseConfirmOpen(false)
    // Close naturally — onCloseRequested listener has been removed
    await getCurrentWindow().close()
  }

  const handleCancelClose = () => {
    useUIStore.getState().setCloseConfirmOpen(false)
  }

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

        <div className="min-w-0 flex-1">
          <MainWindowContent />
        </div>
      </div>

      {/* Global UI Components (hidden until triggered) */}
      <CommandPalette />
      <PreferencesDialog />
      <Toaster position="bottom-right" expand={false} />

      {/* Close confirmation dialog - shown when user has unsaved changes */}
      <AlertDialog open={closeConfirmOpen} onOpenChange={handleCancelClose}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Unsaved Changes</AlertDialogTitle>
            <AlertDialogDescription>
              You have unsaved model changes. Do you want to discard them and
              exit?
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={handleCancelClose}>
              Cancel
            </AlertDialogCancel>
            <Button variant="destructive" onClick={handleConfirmClose}>
              Discard & Exit
            </Button>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
