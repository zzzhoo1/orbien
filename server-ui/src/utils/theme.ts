export type ThemeMode = 'light' | 'dark'

const STORAGE_KEY = 'orbien-server-ui-theme'

function systemPrefersDark(): boolean {
    return window.matchMedia?.('(prefers-color-scheme: dark)').matches ?? true
}

function readStored(): ThemeMode | null {
    const v = localStorage.getItem(STORAGE_KEY)
    return v === 'light' || v === 'dark' ? v : null
}

export function resolveTheme(mode?: ThemeMode | null): ThemeMode {
    return mode ?? readStored() ?? (systemPrefersDark() ? 'dark' : 'light')
}

export function applyTheme(mode: ThemeMode) {
    const root = document.documentElement
    root.dataset.theme = mode
    root.style.colorScheme = mode
    localStorage.setItem(STORAGE_KEY, mode)
}

export function toggleTheme(current: ThemeMode): ThemeMode {
    const next: ThemeMode = current === 'dark' ? 'light' : 'dark'
    applyTheme(next)
    return next
}
