import { useTranslation } from 'react-i18next'
import { AlertCircle, AlertTriangle, CheckCircle2, Info } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { useDshStore } from '@/store/dsh-store'
import type { DshMigrationWarning } from '@/lib/bindings'
import { SettingsRow } from './config/SettingsRow'
import { SettingsSection } from './config/SettingsSection'
import { DshPathSection, DshVersionSection } from './DshConfigFacts'

type WarningSeverity = 'info' | 'warning' | 'error'

function severityOf(severity: string): WarningSeverity {
  if (severity === 'error') return 'error'
  if (severity === 'info') return 'info'
  return 'warning'
}

/** Overview tab: config status, versions, paths, and migration warnings. */
export function DshOverviewTab() {
  const { t } = useTranslation()
  const snapshot = useDshStore(state => state.snapshot)
  const isLoading = useDshStore(state => state.isLoadingSnapshot)

  if (!snapshot) {
    return (
      <p className="py-8 text-center text-sm text-muted-foreground">
        {isLoading ? t('dsh.config.loading') : t('dsh.config.noSnapshot')}
      </p>
    )
  }

  const warningTitle = (warning: DshMigrationWarning): string => {
    switch (warning.code) {
      case 'DSH_CONFIG_TOOLS_MODE_BOTH':
        return t('dsh.warning.DSH_CONFIG_TOOLS_MODE_BOTH.title')
      case 'DSH_CONFIG_SUBAGENT_BACKGROUND_MODE':
        return t('dsh.warning.DSH_CONFIG_SUBAGENT_BACKGROUND_MODE.title')
      case 'DSH_CONFIG_PRESET_ONLY_ROW':
        return t('dsh.warning.DSH_CONFIG_PRESET_ONLY_ROW.title')
      case 'DSH_CONFIG_AZURE_ROUTE_KEY':
        return t('dsh.warning.DSH_CONFIG_AZURE_ROUTE_KEY.title')
      case 'DSH_CONFIG_IMAGE_DETAIL':
        return t('dsh.warning.DSH_CONFIG_IMAGE_DETAIL.title')
      case 'DSH_CONFIG_VERSION_MISMATCH':
        return t('dsh.warning.DSH_CONFIG_VERSION_MISMATCH.title')
      case 'DSH_CONFIG_HOME_SHADOW':
        return t('dsh.warning.DSH_CONFIG_HOME_SHADOW.title')
      case 'DSH_CONFIG_LEGACY_SETTINGS':
        return t('dsh.warning.DSH_CONFIG_LEGACY_SETTINGS.title')
      case 'DSH_CONFIG_NO_PROFILE':
        return t('dsh.warning.DSH_CONFIG_NO_PROFILE.title')
      default:
        return warning.message
    }
  }

  const warningHint = (warning: DshMigrationWarning): string => {
    switch (warning.code) {
      case 'DSH_CONFIG_TOOLS_MODE_BOTH':
        return t('dsh.warning.DSH_CONFIG_TOOLS_MODE_BOTH.hint')
      case 'DSH_CONFIG_SUBAGENT_BACKGROUND_MODE':
        return t('dsh.warning.DSH_CONFIG_SUBAGENT_BACKGROUND_MODE.hint')
      case 'DSH_CONFIG_PRESET_ONLY_ROW':
        return t('dsh.warning.DSH_CONFIG_PRESET_ONLY_ROW.hint')
      case 'DSH_CONFIG_AZURE_ROUTE_KEY':
        return t('dsh.warning.DSH_CONFIG_AZURE_ROUTE_KEY.hint')
      case 'DSH_CONFIG_IMAGE_DETAIL':
        return t('dsh.warning.DSH_CONFIG_IMAGE_DETAIL.hint')
      case 'DSH_CONFIG_VERSION_MISMATCH':
        return t('dsh.warning.DSH_CONFIG_VERSION_MISMATCH.hint', {
          schemaVersion: snapshot.schemaVersion,
          installedVersion: snapshot.installedVersion ?? '',
        })
      case 'DSH_CONFIG_HOME_SHADOW':
        return t('dsh.warning.DSH_CONFIG_HOME_SHADOW.hint')
      case 'DSH_CONFIG_LEGACY_SETTINGS':
        return t('dsh.warning.DSH_CONFIG_LEGACY_SETTINGS.hint')
      case 'DSH_CONFIG_NO_PROFILE':
        return t('dsh.warning.DSH_CONFIG_NO_PROFILE.hint')
      default:
        return warning.hint
    }
  }

  return (
    <div className="space-y-4">
      <SettingsSection title={t('dsh.overview.status')}>
        <SettingsRow label={t('dsh.config.profile')}>
          <span className="text-sm">
            {snapshot.profile ?? t('dsh.field.enumEmpty')}
          </span>
        </SettingsRow>
      </SettingsSection>

      <DshVersionSection snapshot={snapshot} />
      <DshPathSection snapshot={snapshot} />

      <SettingsSection
        title={t('dsh.warning.title')}
        description={
          snapshot.warnings.length === 0 ? t('dsh.warning.empty') : undefined
        }
      >
        {snapshot.warnings.length === 0 ? (
          <p className="flex items-center gap-2 text-sm text-muted-foreground">
            <CheckCircle2
              className="size-4 text-green-600 dark:text-green-400"
              aria-hidden="true"
            />
            {t('dsh.warning.empty')}
          </p>
        ) : (
          <ul className="space-y-3">
            {snapshot.warnings.map(warning => {
              const severity = severityOf(warning.severity)
              const Icon =
                severity === 'error'
                  ? AlertCircle
                  : severity === 'info'
                    ? Info
                    : AlertTriangle
              const tone =
                severity === 'error'
                  ? 'text-destructive'
                  : severity === 'info'
                    ? 'text-muted-foreground'
                    : 'text-amber-600 dark:text-amber-400'
              const severityKey =
                severity === 'error'
                  ? 'dsh.warning.severity.error'
                  : severity === 'info'
                    ? 'dsh.warning.severity.info'
                    : 'dsh.warning.severity.warning'

              return (
                <li
                  key={`${warning.code}:${warning.entryId ?? 'profile'}`}
                  data-warning-code={warning.code}
                  className="space-y-1 rounded-md border p-3"
                >
                  <div className="flex flex-wrap items-center gap-2">
                    <Icon className={`size-4 ${tone}`} aria-hidden="true" />
                    <span className="text-sm font-medium">
                      {warningTitle(warning)}
                    </span>
                    <Badge variant="outline" className={`text-xs ${tone}`}>
                      {t(severityKey)}
                    </Badge>
                    <code className="text-xs text-muted-foreground">
                      {warning.code}
                    </code>
                    {warning.entryId ? (
                      <code className="rounded bg-muted px-1.5 py-0.5 text-xs">
                        {warning.entryId}
                      </code>
                    ) : null}
                  </div>
                  <p className="text-xs text-muted-foreground">
                    {warningHint(warning)}
                  </p>
                </li>
              )
            })}
          </ul>
        )}
      </SettingsSection>
    </div>
  )
}
