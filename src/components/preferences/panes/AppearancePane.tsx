import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { locale } from '@tauri-apps/plugin-os'
import { getSystemFonts } from 'tauri-plugin-system-fonts-api'
import { toast } from 'sonner'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Input } from '@/components/ui/input'
import { useTheme } from '@/hooks/use-theme'
import { SettingsField, SettingsSection } from '../shared/SettingsComponents'
import { usePreferences, useSavePreferences } from '@/services/preferences'
import { useIsWindows } from '@/hooks/use-platform'
import { availableLanguages } from '@/i18n'
import { logger } from '@/lib/logger'

// Language display names (native names)
const languageNames: Record<string, string> = {
  en: 'English',
  zh: '中文',
}

export function AppearancePane() {
  const { t, i18n } = useTranslation()
  const { theme, setTheme } = useTheme()
  const { data: preferences } = usePreferences()
  const savePreferences = useSavePreferences()
  const [systemFonts, setSystemFonts] = useState<string[]>([])
  const [fontsLoading, setFontsLoading] = useState(true)
  const isWindows = useIsWindows()
  const [shellCommand, setShellCommand] = useState('')

  // Sync shell command with preferences
  useEffect(() => {
    if (preferences?.terminal_shell_command !== undefined) {
      setShellCommand(preferences.terminal_shell_command ?? '')
    }
  }, [preferences?.terminal_shell_command])

  // Load system fonts on mount
  useEffect(() => {
    const loadFonts = async () => {
      try {
        const fonts = await getSystemFonts()
        // Extract unique font names, sort alphabetically
        // Prefer monospaced fonts for terminal
        const fontNames = [...new Set(fonts.map(f => f.name))].sort((a, b) =>
          a.localeCompare(b)
        )
        setSystemFonts(fontNames)
      } catch (error) {
        logger.error('Failed to load system fonts', { error })
      } finally {
        setFontsLoading(false)
      }
    }
    loadFonts()
  }, [])

  const handleThemeChange = (value: 'light' | 'dark' | 'system') => {
    // Update the theme provider immediately for instant UI feedback
    setTheme(value)

    // Persist the theme preference to disk, preserving other preferences
    if (preferences) {
      savePreferences.mutate({ ...preferences, theme: value })
    }
  }

  const handleLanguageChange = async (value: string) => {
    const language = value === 'system' ? null : value

    try {
      // Change the language immediately for instant UI feedback
      if (language) {
        await i18n.changeLanguage(language)
      } else {
        // System language selected - detect and apply system locale
        const systemLocale = await locale()
        const langCode = systemLocale?.split('-')[0]?.toLowerCase() ?? 'en'
        const targetLang = availableLanguages.includes(langCode)
          ? langCode
          : 'en'
        await i18n.changeLanguage(targetLang)
      }
    } catch (error) {
      logger.error('Failed to change language', { error })
      toast.error(t('toast.error.generic'))
      return
    }

    // Persist the language preference to disk
    if (preferences) {
      savePreferences.mutate({ ...preferences, language })
    }
  }

  // Determine the current language value for the select
  const currentLanguageValue = preferences?.language ?? 'system'

  // Determine the current terminal font value
  const currentTerminalFont = preferences?.terminal_font_family ?? '__default__'

  const handleTerminalFontChange = (value: string) => {
    const fontFamily = value === '__default__' ? null : value

    // Persist the font preference to disk
    if (preferences) {
      savePreferences.mutate({
        ...preferences,
        terminal_font_family: fontFamily,
      })
    }
  }

  const handleShellCommandBlur = () => {
    const command = shellCommand.trim() || null

    // Only save if value changed
    if (command !== (preferences?.terminal_shell_command ?? null)) {
      if (preferences) {
        savePreferences.mutate({
          ...preferences,
          terminal_shell_command: command,
        })
      }
    }
  }

  return (
    <div className="space-y-6">
      <SettingsSection title={t('preferences.appearance.language')}>
        <SettingsField
          label={t('preferences.appearance.language')}
          description={t('preferences.appearance.languageDescription')}
        >
          <Select
            value={currentLanguageValue}
            onValueChange={handleLanguageChange}
            disabled={savePreferences.isPending}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="system">
                {t('preferences.appearance.language.system')}
              </SelectItem>
              {availableLanguages.map(lang => (
                <SelectItem key={lang} value={lang}>
                  {languageNames[lang] ?? lang}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </SettingsField>
      </SettingsSection>

      <SettingsSection title={t('preferences.appearance.theme')}>
        <SettingsField
          label={t('preferences.appearance.colorTheme')}
          description={t('preferences.appearance.colorThemeDescription')}
        >
          <Select
            value={theme}
            onValueChange={handleThemeChange}
            disabled={savePreferences.isPending}
          >
            <SelectTrigger>
              <SelectValue
                placeholder={t('preferences.appearance.selectTheme')}
              />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="light">
                {t('preferences.appearance.theme.light')}
              </SelectItem>
              <SelectItem value="dark">
                {t('preferences.appearance.theme.dark')}
              </SelectItem>
              <SelectItem value="system">
                {t('preferences.appearance.theme.system')}
              </SelectItem>
            </SelectContent>
          </Select>
        </SettingsField>
      </SettingsSection>

      <SettingsSection title={t('preferences.appearance.terminalFont')}>
        <SettingsField
          label={t('preferences.appearance.terminalFont')}
          description={t('preferences.appearance.terminalFontDescription')}
        >
          <Select
            value={currentTerminalFont}
            onValueChange={handleTerminalFontChange}
            disabled={savePreferences.isPending || fontsLoading}
          >
            <SelectTrigger>
              <SelectValue
                placeholder={t(
                  'preferences.appearance.terminalFontPlaceholder'
                )}
              />
            </SelectTrigger>
            <SelectContent className="max-h-60">
              <SelectItem value="__default__">
                {t('preferences.appearance.terminalFont.default')}
              </SelectItem>
              {systemFonts.map(font => (
                <SelectItem key={font} value={font}>
                  {font}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </SettingsField>
      </SettingsSection>

      {isWindows && (
        <SettingsSection title={t('preferences.appearance.terminalShell')}>
          <SettingsField
            label={t('preferences.appearance.terminalShell')}
            description={t('preferences.appearance.terminalShellDescription')}
          >
            <Input
              value={shellCommand}
              onChange={e => setShellCommand(e.target.value)}
              onBlur={handleShellCommandBlur}
              placeholder={t('preferences.appearance.terminalShellPlaceholder')}
              disabled={savePreferences.isPending}
            />
          </SettingsField>
        </SettingsSection>
      )}
    </div>
  )
}
