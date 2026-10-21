import { useTranslation } from 'react-i18next'
import { Badge } from '@/components/ui/badge'
import type { DshConfigSnapshot } from '@/lib/bindings'
import { SettingsRow } from './config/SettingsRow'
import { SettingsSection } from './config/SettingsSection'

interface DshConfigFactsProps {
  snapshot: DshConfigSnapshot
}

function PathValue({ path }: { path: string }) {
  return (
    <code
      className="block max-w-sm truncate rounded bg-muted px-1.5 py-0.5 text-xs select-all"
      title={path}
    >
      {path}
    </code>
  )
}

function ExistsBadge({ exists }: { exists: boolean }) {
  const { t } = useTranslation()

  return (
    <Badge variant={exists ? 'secondary' : 'outline'} className="text-xs">
      {exists ? t('common.exists') : t('common.missing')}
    </Badge>
  )
}

/** Version rows shared by the overview and diagnostics tabs. */
export function DshVersionSection({ snapshot }: DshConfigFactsProps) {
  const { t } = useTranslation()
  const mismatch =
    snapshot.installedVersion !== null &&
    snapshot.installedVersion !== snapshot.schemaVersion

  return (
    <SettingsSection title={t('dsh.overview.versions')}>
      <SettingsRow label={t('dsh.config.installedVersion')}>
        <span className="text-sm">
          {snapshot.installedVersion ?? t('common.missing')}
        </span>
        {mismatch ? (
          <Badge
            variant="outline"
            className="ms-2 text-xs text-amber-600 dark:text-amber-400"
          >
            {t('dsh.warning.DSH_CONFIG_VERSION_MISMATCH.title')}
          </Badge>
        ) : null}
      </SettingsRow>
      <SettingsRow label={t('dsh.config.schemaVersion')}>
        <span className="text-sm">{snapshot.schemaVersion}</span>
      </SettingsRow>
    </SettingsSection>
  )
}

/** Config file path rows shared by the overview and diagnostics tabs. */
export function DshPathSection({ snapshot }: DshConfigFactsProps) {
  const { t } = useTranslation()

  return (
    <SettingsSection title={t('dsh.overview.paths')}>
      <SettingsRow label={t('dsh.config.profilePatch')}>
        <PathValue path={snapshot.profilePatchPath} />
      </SettingsRow>
      <SettingsRow label={t('dsh.config.homePatch')}>
        <div className="flex min-w-0 items-center gap-2">
          <PathValue path={snapshot.homePatchPath} />
          <ExistsBadge exists={snapshot.homePatchExists} />
        </div>
      </SettingsRow>
      <SettingsRow label={t('dsh.config.legacyPath')}>
        <div className="flex min-w-0 items-center gap-2">
          <PathValue path={snapshot.legacySettingsPath} />
          <ExistsBadge exists={snapshot.legacySettingsExists} />
        </div>
      </SettingsRow>
      <SettingsRow label={t('dsh.config.hash')}>
        <code className="rounded bg-muted px-1.5 py-0.5 text-xs select-all">
          {snapshot.patchHash}
        </code>
      </SettingsRow>
    </SettingsSection>
  )
}
