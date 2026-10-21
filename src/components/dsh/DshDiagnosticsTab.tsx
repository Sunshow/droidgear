import { useTranslation } from 'react-i18next'
import { Badge } from '@/components/ui/badge'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { useDshStore } from '@/store/dsh-store'
import { JsExpressionBadge } from './config/JsExpressionBadge'
import { OverrideBadge } from './config/OverrideBadge'
import { SettingsSection } from './config/SettingsSection'
import { DshPathSection, DshVersionSection } from './DshConfigFacts'
import { layerLabelKey, tierLabelKey } from './dsh-config-utils'

/** Diagnostics tab: read-only inventory and file locations. */
export function DshDiagnosticsTab() {
  const { t } = useTranslation()
  const snapshot = useDshStore(state => state.snapshot)
  const isLoading = useDshStore(state => state.isLoadingSnapshot)
  const credentials = useDshStore(state => state.credentials)

  if (!snapshot) {
    return (
      <p className="py-8 text-center text-sm text-muted-foreground">
        {isLoading ? t('dsh.config.loading') : t('dsh.config.noSnapshot')}
      </p>
    )
  }

  const credentialNames = Object.keys(credentials)

  return (
    <div className="space-y-4">
      <DshVersionSection snapshot={snapshot} />
      <DshPathSection snapshot={snapshot} />

      <SettingsSection title={t('dsh.diagnostics.entries')}>
        {snapshot.entries.length === 0 ? (
          <p className="text-sm text-muted-foreground">
            {t('dsh.diagnostics.empty')}
          </p>
        ) : (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{t('dsh.diagnostics.col.id')}</TableHead>
                <TableHead>{t('dsh.diagnostics.col.package')}</TableHead>
                <TableHead>{t('dsh.diagnostics.col.layer')}</TableHead>
                <TableHead>{t('dsh.diagnostics.col.tier')}</TableHead>
                <TableHead>{t('dsh.diagnostics.col.status')}</TableHead>
                <TableHead>{t('dsh.diagnostics.col.flags')}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {snapshot.entries.map(entry => (
                <TableRow key={entry.id} data-entry-id={entry.id}>
                  <TableCell className="font-mono text-xs">
                    {entry.id}
                  </TableCell>
                  <TableCell className="font-mono text-xs text-muted-foreground">
                    {entry.packageName}
                  </TableCell>
                  <TableCell>
                    <Badge variant="outline" className="text-xs">
                      {t(layerLabelKey(entry.layer))}
                    </Badge>
                  </TableCell>
                  <TableCell className="text-xs">
                    {t(tierLabelKey(entry.tier))}
                  </TableCell>
                  <TableCell className="text-xs">
                    {entry.disabled
                      ? t('dsh.config.disabled')
                      : t('common.enabled')}
                  </TableCell>
                  <TableCell>
                    <div className="flex flex-wrap items-center gap-1">
                      {entry.overridden ? <OverrideBadge /> : null}
                      {entry.hasJs ? <JsExpressionBadge /> : null}
                      {entry.shadowedByHome ? (
                        <Badge
                          variant="outline"
                          className="text-xs text-destructive"
                          title={t('dsh.config.homePatchShadow')}
                        >
                          {t('dsh.config.homePatch')}
                        </Badge>
                      ) : null}
                    </div>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
      </SettingsSection>

      <SettingsSection title={t('dsh.diagnostics.credentials')}>
        {credentialNames.length === 0 ? (
          <p className="text-sm text-muted-foreground">
            {t('dsh.diagnostics.noCredentials')}
          </p>
        ) : (
          <div className="flex flex-wrap gap-2">
            {credentialNames.map(name => (
              <Badge
                key={name}
                variant="secondary"
                className="font-mono text-xs"
              >
                {name}
              </Badge>
            ))}
          </div>
        )}
      </SettingsSection>

      <SettingsSection title={t('dsh.diagnostics.sessions')}>
        <p className="text-sm text-muted-foreground">
          {t('dsh.diagnostics.sessionsHint')}
        </p>
      </SettingsSection>
    </div>
  )
}
