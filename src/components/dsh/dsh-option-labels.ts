import { useTranslation } from 'react-i18next'

/**
 * Translate a curated enum value from the DSH config schema.
 * Unknown values fall back to the raw identifier so new schema values still
 * render sensibly.
 */
export function useOptionLabel(): (value: string) => string {
  const { t } = useTranslation()

  return value => {
    switch (value) {
      case 'zh':
        return t('dsh.option.zh')
      case 'en':
        return t('dsh.option.en')
      case 'light':
        return t('dsh.option.light')
      case 'dark':
        return t('dsh.option.dark')
      case 'system':
        return t('dsh.option.system')
      case 'compact':
        return t('dsh.option.compact')
      case 'standard':
        return t('dsh.option.standard')
      case 'detailed':
        return t('dsh.option.detailed')
      case 'verbose':
        return t('dsh.option.verbose')
      case 'completion':
        return t('dsh.option.completion')
      case 'next-input':
        return t('dsh.option.next-input')
      case 'sidebar':
        return t('dsh.option.sidebar')
      case 'new-tab':
        return t('dsh.option.new-tab')
      case 'queue':
        return t('dsh.option.queue')
      case 'steer':
        return t('dsh.option.steer')
      case 'off':
        return t('dsh.option.off')
      case 'low':
        return t('dsh.option.low')
      case 'medium':
        return t('dsh.option.medium')
      case 'high':
        return t('dsh.option.high')
      case 'max':
        return t('dsh.option.max')
      case 'read-only':
        return t('dsh.option.read-only')
      case 'workspace-write':
        return t('dsh.option.workspace-write')
      case 'danger-full-access':
        return t('dsh.option.danger-full-access')
      case 'ask':
        return t('dsh.option.ask')
      case 'never':
        return t('dsh.option.never')
      case 'ptc':
        return t('dsh.option.ptc')
      case 'minimal':
        return t('dsh.option.minimal')
      case 'cordis':
        return t('dsh.option.cordis')
      case 'deepseek-official':
        return t('dsh.option.deepseek-official')
      case 'http':
        return t('dsh.option.http')
      case 'native':
        return t('dsh.option.native')
      default:
        return value
    }
  }
}
