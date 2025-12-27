/**
 * Shared color mappings for platform/provider badges
 * Used by both KeyList (platform) and ModelCard (provider)
 */

export const platformColors: Record<string, string> = {
  anthropic:
    'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200',
  openai: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
  gemini:
    'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200',
}

// Provider colors map to platform colors where applicable
export const providerColors: Record<string, string> = {
  anthropic:
    'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200',
  openai: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
  'generic-chat-completion-api':
    'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
}

export const providerLabels: Record<string, string> = {
  anthropic: 'Anthropic',
  openai: 'OpenAI',
  'generic-chat-completion-api': 'Generic',
}
