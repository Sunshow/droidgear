import { Button } from '@/components/ui/button'

interface UpdateNotificationContentProps {
  message: string
  releaseUrl: string
  releaseLabel: string
  installLabel: string
  laterLabel: string
  onOpenRelease: () => void
  onInstallNow: () => void
  onLater: () => void
}

export function UpdateNotificationContent({
  message,
  releaseUrl,
  releaseLabel,
  installLabel,
  laterLabel,
  onOpenRelease,
  onInstallNow,
  onLater,
}: UpdateNotificationContentProps) {
  return (
    <div className="pointer-events-auto flex w-full max-w-[min(100vw-2rem,420px)] flex-col gap-3 rounded-lg border border-border bg-popover p-4 text-popover-foreground shadow-lg">
      <p className="text-sm leading-snug">{message}</p>
      <button
        type="button"
        onClick={event => {
          event.stopPropagation()
          onOpenRelease()
        }}
        className="cursor-pointer self-start text-xs text-primary underline underline-offset-2"
        title={releaseUrl}
      >
        {releaseLabel}
      </button>
      <div className="flex shrink-0 flex-wrap items-center justify-end gap-2">
        <Button
          type="button"
          variant="secondary"
          size="sm"
          className="cursor-pointer"
          onClick={event => {
            event.stopPropagation()
            onLater()
          }}
        >
          {laterLabel}
        </Button>
        <Button
          type="button"
          size="sm"
          className="cursor-pointer"
          onClick={event => {
            event.stopPropagation()
            onInstallNow()
          }}
        >
          {installLabel}
        </Button>
      </div>
    </div>
  )
}
