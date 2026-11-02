import ReactDOM from 'react-dom/client'
import { QueryClientProvider } from '@tanstack/react-query'
import { ReactQueryDevtools } from '@tanstack/react-query-devtools'
import './i18n'
import App from './App'
import { queryClient } from './lib/query-client'
import { applyInitialTheme } from './lib/theme-context'

// Apply the persisted theme before the first render so the initial frame
// doesn't flash the wrong palette (ThemeProvider takes over after mount).
applyInitialTheme()

// Disable default browser context menu to prevent accidental page reload
document.addEventListener('contextmenu', e => {
  e.preventDefault()
})

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <QueryClientProvider client={queryClient}>
    <App />
    <ReactQueryDevtools initialIsOpen={false} />
  </QueryClientProvider>
)
