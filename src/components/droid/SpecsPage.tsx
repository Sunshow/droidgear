import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { RefreshCw, FileText, Clock } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { cn } from '@/lib/utils'
import { commands, type SpecFile } from '@/lib/bindings'

export function SpecsPage() {
  const { t } = useTranslation()
  const [specs, setSpecs] = useState<SpecFile[]>([])
  const [selectedSpec, setSelectedSpec] = useState<SpecFile | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const loadSpecs = async () => {
    setLoading(true)
    setError(null)
    try {
      const result = await commands.listSpecs()
      if (result.status === 'ok') {
        setSpecs(result.data)
        // Auto-select first spec if none selected
        if (result.data.length > 0 && !selectedSpec) {
          setSelectedSpec(result.data[0])
        }
      } else {
        setError(result.error)
      }
    } catch (err) {
      setError(String(err))
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadSpecs()
  }, [])

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

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between p-4 border-b">
        <h1 className="text-xl font-semibold">{t('droid.specs.title')}</h1>
        <Button
          variant="outline"
          size="sm"
          onClick={loadSpecs}
          disabled={loading}
        >
          <RefreshCw className={cn('h-4 w-4 mr-2', loading && 'animate-spin')} />
          {t('common.refresh')}
        </Button>
      </div>

      <div className="flex flex-1 overflow-hidden">
        {/* Specs List */}
        <div className="w-64 border-r flex flex-col">
          <ScrollArea className="flex-1">
            <div className="p-2 space-y-1">
              {loading && specs.length === 0 ? (
                <div className="flex items-center justify-center p-4 text-muted-foreground">
                  {t('common.loading')}
                </div>
              ) : error ? (
                <div className="p-4 text-destructive text-sm">{error}</div>
              ) : specs.length === 0 ? (
                <div className="flex flex-col items-center justify-center p-4 text-muted-foreground text-sm">
                  <FileText className="h-8 w-8 mb-2 opacity-50" />
                  <p>{t('droid.specs.noSpecs')}</p>
                  <p className="text-xs mt-1">{t('droid.specs.noSpecsHint')}</p>
                </div>
              ) : (
                specs.map(spec => (
                  <button
                    key={spec.path}
                    onClick={() => setSelectedSpec(spec)}
                    className={cn(
                      'w-full text-start p-2 rounded-md hover:bg-accent transition-colors',
                      selectedSpec?.path === spec.path && 'bg-accent'
                    )}
                  >
                    <div className="font-medium text-sm truncate">{spec.name}</div>
                    <div className="flex items-center gap-1 text-xs text-muted-foreground mt-1">
                      <Clock className="h-3 w-3" />
                      {formatDate(spec.modifiedAt)}
                    </div>
                  </button>
                ))
              )}
            </div>
          </ScrollArea>
        </div>

        {/* Spec Content */}
        <div className="flex-1 flex flex-col">
          {selectedSpec ? (
            <>
              <div className="p-4 border-b">
                <h2 className="font-medium">{selectedSpec.name}</h2>
                <p className="text-xs text-muted-foreground mt-1">
                  {formatDate(selectedSpec.modifiedAt)}
                </p>
              </div>
              <ScrollArea className="flex-1">
                <div className="p-4">
                  <pre className="whitespace-pre-wrap text-sm font-mono">
                    {selectedSpec.content}
                  </pre>
                </div>
              </ScrollArea>
            </>
          ) : (
            <div className="flex items-center justify-center h-full text-muted-foreground">
              <p>{t('droid.specs.selectSpecHint')}</p>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
