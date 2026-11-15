import { useTranslation } from 'react-i18next'
import {
  CircleAlert,
  CircleCheck,
  Loader2,
  type LucideIcon,
} from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { cn } from '@/lib/utils'
import type { ImageTaskStatus } from '@/store/image-playground-store'

const STATUS_STYLE: Record<
  ImageTaskStatus,
  { icon: LucideIcon; className: string }
> = {
  running: {
    icon: Loader2,
    className: 'animate-spin text-blue-600 dark:text-blue-400',
  },
  completed: {
    icon: CircleCheck,
    className: 'text-green-600 dark:text-green-400',
  },
  failed: { icon: CircleAlert, className: 'text-destructive' },
}

export function ImageTaskStatusIcon({
  status,
  className,
}: {
  status: ImageTaskStatus
  className?: string
}) {
  const { icon: Icon, className: tone } = STATUS_STYLE[status]
  return <Icon className={cn('size-4 shrink-0', tone, className)} />
}

export function ImageTaskStatusBadge({ status }: { status: ImageTaskStatus }) {
  const { t } = useTranslation()
  return (
    <Badge variant="outline" className="gap-1">
      <ImageTaskStatusIcon status={status} className="size-3" />
      {t(`playground.image.status.${status}`)}
    </Badge>
  )
}
