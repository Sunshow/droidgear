import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { RefreshCw, Copy, Check, Loader2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { useChannelStore } from '@/store/channel-store'
import { platformColors } from '@/lib/platform-colors'
import type { ChannelToken, ChannelType } from '@/lib/bindings'

interface KeyListProps {
  channelId: string
  channelType: ChannelType
  baseUrl: string
  onSelectKey: (key: ChannelToken) => void
}

function formatQuota(
  quota: number,
  unlimited: boolean,
  t: (key: string) => string
): string {
  if (unlimited) return t('keys.unlimited')
  if (quota >= 1000000) return `${(quota / 1000000).toFixed(2)}M`
  if (quota >= 1000) return `${(quota / 1000).toFixed(2)}K`
  return quota.toString()
}

function getStatusBadge(status: number, t: (key: string) => string) {
  switch (status) {
    case 1:
      return <Badge variant="default">{t('keys.status.enabled')}</Badge>
    case 2:
      return <Badge variant="secondary">{t('keys.status.disabled')}</Badge>
    case 3:
      return <Badge variant="destructive">{t('keys.status.expired')}</Badge>
    case 4:
      return <Badge variant="outline">{t('keys.status.exhausted')}</Badge>
    default:
      return <Badge variant="outline">{t('keys.status.unknown')}</Badge>
  }
}

export function KeyList({
  channelId,
  channelType,
  baseUrl,
  onSelectKey,
}: KeyListProps) {
  const { t } = useTranslation()
  const keysMap = useChannelStore(state => state.keys)
  const isLoading = useChannelStore(state => state.isLoading)
  const fetchKeys = useChannelStore(state => state.fetchKeys)
  const [copiedId, setCopiedId] = useState<number | null>(null)

  // Safely get keys array
  const keys: ChannelToken[] = keysMap?.[channelId] ?? []

  const handleRefresh = () => {
    fetchKeys(channelId, channelType, baseUrl)
  }

  // Auto refresh keys when channel changes and no keys exist
  useEffect(() => {
    if (keys.length === 0 && !isLoading) {
      fetchKeys(channelId, channelType, baseUrl)
    }
  }, [channelId, keys.length, isLoading, fetchKeys, channelType, baseUrl])

  const handleCopyKey = async (apiKey: ChannelToken) => {
    await navigator.clipboard.writeText(apiKey.key)
    setCopiedId(apiKey.id)
    setTimeout(() => setCopiedId(null), 2000)
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-medium">{t('keys.title')}</h3>
        <Button
          variant="outline"
          size="sm"
          onClick={handleRefresh}
          disabled={isLoading}
        >
          {isLoading ? (
            <Loader2 className="h-4 w-4 animate-spin mr-2" />
          ) : (
            <RefreshCw className="h-4 w-4 mr-2" />
          )}
          {t('common.refresh')}
        </Button>
      </div>

      {keys.length === 0 ? (
        <div className="text-center py-8 text-muted-foreground">
          <p>{t('keys.noKeys')}</p>
          <p className="text-sm mt-1">{t('keys.noKeysHint')}</p>
        </div>
      ) : (
        <div className="border rounded-md">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{t('common.name')}</TableHead>
                <TableHead>{t('common.status')}</TableHead>
                <TableHead>{t('keys.platform')}</TableHead>
                <TableHead>{t('keys.remaining')}</TableHead>
                <TableHead>{t('keys.used')}</TableHead>
                <TableHead className="w-[100px]">
                  {t('common.actions')}
                </TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {keys.map(apiKey => (
                <TableRow key={apiKey.id}>
                  <TableCell className="font-medium">{apiKey.name}</TableCell>
                  <TableCell>{getStatusBadge(apiKey.status, t)}</TableCell>
                  <TableCell>
                    {apiKey.platform ? (
                      <Badge
                        variant="outline"
                        className={platformColors[apiKey.platform] || ''}
                      >
                        {apiKey.platform}
                      </Badge>
                    ) : (
                      <span className="text-muted-foreground">-</span>
                    )}
                  </TableCell>
                  <TableCell>
                    {formatQuota(apiKey.remainQuota, apiKey.unlimitedQuota, t)}
                  </TableCell>
                  <TableCell>
                    {formatQuota(apiKey.usedQuota, false, t)}
                  </TableCell>
                  <TableCell>
                    <div className="flex items-center gap-1">
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8"
                        onClick={() => handleCopyKey(apiKey)}
                        title={t('keys.copyKey')}
                      >
                        {copiedId === apiKey.id ? (
                          <Check className="h-4 w-4 text-green-500" />
                        ) : (
                          <Copy className="h-4 w-4" />
                        )}
                      </Button>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => onSelectKey(apiKey)}
                        disabled={apiKey.status !== 1}
                      >
                        {t('sidebar.models')}
                      </Button>
                    </div>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      )}
    </div>
  )
}
