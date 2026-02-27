import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { AlertCircle, Download } from 'lucide-react'
import { openUrl } from '@tauri-apps/plugin-opener'
import { arch } from '@tauri-apps/plugin-os'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { usePlatform } from '@/hooks/use-platform'
import { useUIStore } from '@/store/ui-store'

interface PlatformOption {
  value: string
  label: string
  os: string
  arch: string
  ext: string
}

const platformOptions: PlatformOption[] = [
  {
    value: 'darwin-arm64',
    label: 'macOS (Apple Silicon)',
    os: 'darwin',
    arch: 'arm64',
    ext: '',
  },
  {
    value: 'darwin-x64',
    label: 'macOS (Intel)',
    os: 'darwin',
    arch: 'x64',
    ext: '',
  },
  {
    value: 'linux-x64',
    label: 'Linux (x64)',
    os: 'linux',
    arch: 'x64',
    ext: '',
  },
  {
    value: 'windows-x64',
    label: 'Windows (x64)',
    os: 'windows',
    arch: 'x64',
    ext: '.exe',
  },
]

function getDefaultPlatformValue(
  appPlatform: string,
  systemArch: string
): string {
  if (appPlatform === 'macos') {
    return systemArch === 'aarch64' ? 'darwin-arm64' : 'darwin-x64'
  }
  if (appPlatform === 'windows') return 'windows-x64'
  return 'linux-x64'
}

function buildDownloadUrl(option: PlatformOption, version: string): string {
  return `https://downloads.factory.ai/factory-cli/releases/${version}/${option.os}/${option.arch}/droid${option.ext}`
}

export function LegacyVersionsPage() {
  const { t } = useTranslation()
  const appPlatform = usePlatform()
  const setDroidSubView = useUIStore(state => state.setDroidSubView)
  const setDroidHelpersScrollTarget = useUIStore(
    state => state.setDroidHelpersScrollTarget
  )

  let systemArch = 'x86_64'
  try {
    systemArch = arch()
  } catch {
    // fallback
  }

  const [selectedPlatform, setSelectedPlatform] = useState(
    getDefaultPlatformValue(appPlatform, systemArch)
  )
  const [version, setVersion] = useState('')

  const handleDownload = async () => {
    const trimmed = version.trim()
    if (!trimmed) {
      toast.error(t('droid.legacyVersions.versionRequired'))
      return
    }

    const option = platformOptions.find(o => o.value === selectedPlatform)
    if (!option) return

    const url = buildDownloadUrl(option, trimmed)
    await openUrl(url)
  }

  const handleGoDisableAutoUpdate = () => {
    setDroidHelpersScrollTarget('disable-auto-update')
    setDroidSubView('helpers')
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between p-4 border-b">
        <h1 className="text-xl font-semibold">
          {t('droid.legacyVersions.title')}
        </h1>
      </div>

      <div className="flex-1 overflow-y-auto p-4">
        <div className="space-y-6 max-w-md">
          {/* Platform Select */}
          <div className="space-y-2">
            <Label>{t('droid.legacyVersions.platform')}</Label>
            <Select
              value={selectedPlatform}
              onValueChange={setSelectedPlatform}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {platformOptions.map(opt => (
                  <SelectItem key={opt.value} value={opt.value}>
                    {opt.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Version Input */}
          <div className="space-y-2">
            <Label>{t('droid.legacyVersions.version')}</Label>
            <Input
              value={version}
              onChange={e => setVersion(e.target.value)}
              placeholder={t('droid.legacyVersions.versionPlaceholder')}
            />
          </div>

          {/* Download Button */}
          <Button onClick={handleDownload} disabled={!version.trim()}>
            <Download className="h-4 w-4 mr-2" />
            {t('droid.legacyVersions.download')}
          </Button>

          {/* Auto Update Hint */}
          <div className="p-4 bg-blue-500/10 border border-blue-500/20 rounded-md space-y-2">
            <div className="flex items-center gap-2 text-blue-600 dark:text-blue-400">
              <AlertCircle className="h-5 w-5 shrink-0" />
              <span className="text-sm">
                {t('droid.legacyVersions.autoUpdateHint')}
              </span>
            </div>
            <Button
              variant="link"
              className="h-auto p-0 text-blue-600 dark:text-blue-400"
              onClick={handleGoDisableAutoUpdate}
            >
              {t('droid.legacyVersions.goDisableAutoUpdate')}
            </Button>
          </div>
        </div>
      </div>
    </div>
  )
}
