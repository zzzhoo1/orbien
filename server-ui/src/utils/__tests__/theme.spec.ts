import {describe, expect, it, vi, beforeEach, afterEach} from 'vitest'
import {resolveTheme, applyTheme, toggleTheme} from '../theme'

const STORAGE_KEY = 'orbien-server-ui-theme'

function setMatchMedia(prefersDark: boolean) {
  Object.defineProperty(window, 'matchMedia', {
    writable: true,
    value: vi.fn().mockImplementation((query: string) => ({
      matches: query === '(prefers-color-scheme: dark)' ? prefersDark : false,
      media: query,
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })),
  })
}

beforeEach(() => {
  localStorage.clear()
  document.documentElement.removeAttribute('data-theme')
  document.documentElement.style.colorScheme = ''
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('resolveTheme', () => {
  it('returns the explicit mode "light" when provided', () => {
    expect(resolveTheme('light')).toBe('light')
  })

  it('returns the explicit mode "dark" when provided', () => {
    expect(resolveTheme('dark')).toBe('dark')
  })

  it('falls back to stored value "light" when no mode given', () => {
    localStorage.setItem(STORAGE_KEY, 'light')
    expect(resolveTheme()).toBe('light')
  })

  it('falls back to stored value "dark" when no mode given', () => {
    localStorage.setItem(STORAGE_KEY, 'dark')
    expect(resolveTheme()).toBe('dark')
  })

  it('ignores invalid localStorage value and falls back to system', () => {
    localStorage.setItem(STORAGE_KEY, 'invalid-theme')
    setMatchMedia(true)
    expect(resolveTheme()).toBe('dark')
  })

  it('falls back to system dark preference when nothing stored', () => {
    setMatchMedia(true)
    expect(resolveTheme()).toBe('dark')
  })

  it('falls back to system light preference when nothing stored', () => {
    setMatchMedia(false)
    expect(resolveTheme()).toBe('light')
  })

  it('explicit mode overrides localStorage', () => {
    localStorage.setItem(STORAGE_KEY, 'dark')
    expect(resolveTheme('light')).toBe('light')
  })

  it('null mode falls through to localStorage', () => {
    localStorage.setItem(STORAGE_KEY, 'dark')
    expect(resolveTheme(null)).toBe('dark')
  })

  it('undefined mode falls through to localStorage', () => {
    localStorage.setItem(STORAGE_KEY, 'light')
    expect(resolveTheme(undefined)).toBe('light')
  })
})

describe('applyTheme', () => {
  it('sets data-theme="dark" on documentElement', () => {
    applyTheme('dark')
    expect(document.documentElement.dataset.theme).toBe('dark')
  })

  it('sets data-theme="light" on documentElement', () => {
    applyTheme('light')
    expect(document.documentElement.dataset.theme).toBe('light')
  })

  it('sets colorScheme="light" on documentElement', () => {
    applyTheme('light')
    expect(document.documentElement.style.colorScheme).toBe('light')
  })

  it('sets colorScheme="dark" on documentElement', () => {
    applyTheme('dark')
    expect(document.documentElement.style.colorScheme).toBe('dark')
  })

  it('persists "dark" to localStorage', () => {
    applyTheme('dark')
    expect(localStorage.getItem(STORAGE_KEY)).toBe('dark')
  })

  it('persists "light" to localStorage', () => {
    applyTheme('light')
    expect(localStorage.getItem(STORAGE_KEY)).toBe('light')
  })

  it('overwrites previous localStorage value', () => {
    localStorage.setItem(STORAGE_KEY, 'dark')
    applyTheme('light')
    expect(localStorage.getItem(STORAGE_KEY)).toBe('light')
  })
})

describe('toggleTheme', () => {
  it('toggles dark -> light', () => {
    expect(toggleTheme('dark')).toBe('light')
  })

  it('toggles light -> dark', () => {
    expect(toggleTheme('light')).toBe('dark')
  })

  it('applies the new theme side effect (light -> dark)', () => {
    toggleTheme('light')
    expect(document.documentElement.dataset.theme).toBe('dark')
    expect(localStorage.getItem(STORAGE_KEY)).toBe('dark')
  })

  it('applies the new theme side effect (dark -> light)', () => {
    toggleTheme('dark')
    expect(document.documentElement.dataset.theme).toBe('light')
    expect(localStorage.getItem(STORAGE_KEY)).toBe('light')
  })

  it('double toggle returns to original mode', () => {
    const after1 = toggleTheme('dark')
    const after2 = toggleTheme(after1)
    expect(after2).toBe('dark')
  })
})
