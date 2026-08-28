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
  it('returns the explicit mode when provided', () => {
    expect(resolveTheme('light')).toBe('light')
    expect(resolveTheme('dark')).toBe('dark')
  })

  it('falls back to stored value when no mode given', () => {
    localStorage.setItem(STORAGE_KEY, 'light')
    expect(resolveTheme()).toBe('light')
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
})

describe('applyTheme', () => {
  it('sets data-theme on documentElement', () => {
    applyTheme('dark')
    expect(document.documentElement.dataset.theme).toBe('dark')
  })

  it('sets colorScheme on documentElement', () => {
    applyTheme('light')
    expect(document.documentElement.style.colorScheme).toBe('light')
  })

  it('persists to localStorage', () => {
    applyTheme('dark')
    expect(localStorage.getItem(STORAGE_KEY)).toBe('dark')
  })
})

describe('toggleTheme', () => {
  it('toggles dark -> light', () => {
    expect(toggleTheme('dark')).toBe('light')
  })

  it('toggles light -> dark', () => {
    expect(toggleTheme('light')).toBe('dark')
  })

  it('applies the new theme as a side effect', () => {
    toggleTheme('light')
    expect(document.documentElement.dataset.theme).toBe('dark')
    expect(localStorage.getItem(STORAGE_KEY)).toBe('dark')
  })
})
