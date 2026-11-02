import { createContext } from 'react'

export type Theme = 'dark' | 'light' | 'system'

/** localStorage key holding the user-chosen theme (see ThemeProvider). */
export const THEME_STORAGE_KEY = 'ui-theme'

export interface ThemeProviderState {
  theme: Theme
  setTheme: (theme: Theme) => void
}

const initialState: ThemeProviderState = {
  theme: 'system',
  setTheme: () => null,
}

export const ThemeProviderContext =
  createContext<ThemeProviderState>(initialState)

/** Resolve a theme setting against the current OS colour-scheme preference. */
export function resolveIsDark(theme: Theme): boolean {
  return theme === 'system'
    ? window.matchMedia('(prefers-color-scheme: dark)').matches
    : theme === 'dark'
}

/**
 * Apply the persisted theme to `<html>` before React mounts.
 *
 * ThemeProvider only sets `.dark`/`.light` in an effect, so without this the
 * first rendered frame would fall back to the light palette (wrong-colour
 * flash for dark-theme users). Called from main.tsx before the first render;
 * ThemeProvider takes over from there.
 */
export function applyInitialTheme(storageKey = THEME_STORAGE_KEY): void {
  const stored = localStorage.getItem(storageKey) as Theme | null
  const isDark = resolveIsDark(stored ?? 'system')
  const root = document.documentElement
  root.classList.toggle('dark', isDark)
  root.classList.toggle('light', !isDark)
}
