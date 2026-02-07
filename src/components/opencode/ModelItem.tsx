import { useTranslation } from 'react-i18next'
import { Pencil, Trash2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import type { OpenCodeModelConfig } from '@/lib/bindings'

interface ModelItemProps {
  modelId: string
  config: OpenCodeModelConfig
  onEdit: () => void
  onDelete: () => void
}

export function ModelItem({
  modelId,
  config,
  onEdit,
  onDelete,
}: ModelItemProps) {
  const { t } = useTranslation()

  return (
    <div className="flex items-center justify-between p-2 border rounded-md hover:bg-muted/50 transition-colors">
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="font-medium text-sm">{modelId}</span>
          {config.name && (
            <span className="text-muted-foreground text-xs">
              ({config.name})
            </span>
          )}
        </div>
        {config.limit && (
          <div className="flex items-center gap-2 mt-1">
            {config.limit.context && (
              <Badge variant="secondary" className="text-xs">
                Context: {config.limit.context.toLocaleString()}
              </Badge>
            )}
            {config.limit.output && (
              <Badge variant="secondary" className="text-xs">
                Output: {config.limit.output.toLocaleString()}
              </Badge>
            )}
          </div>
        )}
      </div>
      <div className="flex items-center gap-1 ml-2">
        <Button
          variant="ghost"
          size="icon"
          className="h-7 w-7"
          onClick={onEdit}
          title={t('common.edit')}
        >
          <Pencil className="h-3 w-3" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="h-7 w-7"
          onClick={onDelete}
          title={t('common.delete')}
        >
          <Trash2 className="h-3 w-3" />
        </Button>
      </div>
    </div>
  )
}
