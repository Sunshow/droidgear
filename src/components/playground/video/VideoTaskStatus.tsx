import { useTranslation } from 'react-i18next'
import {
  Ban,
  CircleAlert,
  CircleCheck,
  Clock,
  Loader2,
  type LucideIcon,
} from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { cn } from '@/lib/utils'
import type { VideoTaskStatus } from '@/store/playground-store'

const STATUS_STYLE: Record<
  VideoTaskStatus,
  { icon: LucideIcon; className: string }
> = {
  queued: { icon: Clock, className: 'text-muted-foreground' },
  in_progress: {
    icon: Loader2,
    className: 'animate-spin text-blue-600 dark:text-blue-400',
  },
  completed: {
    icon: CircleCheck,
    className: 'text-green-600 dark:text-green-400',
  },
  failed: { icon: CircleAlert, className: 'text-destructive' },
  cancelled: { icon: Ban, className: 'text-muted-foreground' },
}

export function VideoTaskStatusIcon({
  status,
  className,
}: {
  status: VideoTaskStatus
  className?: string
}) {
  const { icon: Icon, className: tone } = STATUS_STYLE[status]
  return <Icon className={cn('size-4 shrink-0', tone, className)} />
}

export function VideoTaskStatusBadge({ status }: { status: VideoTaskStatus }) {
  const { t } = useTranslation()
  return (
    <Badge variant="outline" className="gap-1">
      <VideoTaskStatusIcon status={status} className="size-3" />
      {t(`playground.video.status.${status}`)}
    </Badge>
  )
}

export function VideoTaskProgress({
  value,
  className,
}: {
  value: number
  className?: string
}) {
  return (
    <div
      role="progressbar"
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={value}
      className={cn(
        'h-1.5 w-full overflow-hidden rounded-full bg-muted',
        className
      )}
    >
      <div
        className="h-full rounded-full bg-primary transition-[width] duration-500"
        style={{ width: `${Math.max(value, 2)}%` }}
      />
    </div>
  )
}
