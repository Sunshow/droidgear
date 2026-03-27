interface UpdateNotificationContentProps {
  message: string
  releaseUrl: string
  releaseLabel: string
  onOpenRelease: () => void
}

export function UpdateNotificationContent({
  message,
  releaseUrl,
  releaseLabel,
  onOpenRelease,
}: UpdateNotificationContentProps) {
  return (
    <div className="space-y-2">
      <p>{message}</p>
      <button
        type="button"
        onClick={onOpenRelease}
        className="text-xs text-primary underline underline-offset-2"
        title={releaseUrl}
      >
        {releaseLabel}
      </button>
    </div>
  )
}
