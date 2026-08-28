import {describe, it, expect, beforeEach, afterEach, vi} from 'vitest'
import {mount} from '@vue/test-utils'
import {defineComponent, nextTick} from 'vue'
import {createI18n} from 'vue-i18n'

const messages = {
  'en-US': {
    actions: {
      themeToLight: 'Switch to light theme',
      themeToDark: 'Switch to dark theme',
    },
  },
}

function setupDom(theme?: 'light' | 'dark') {
  if (theme) {
    document.documentElement.dataset.theme = theme
  } else {
    delete document.documentElement.dataset.theme
  }
}

function setupMatchMedia(prefersDark: boolean) {
  Object.defineProperty(window, 'matchMedia', {
    writable: true,
    value: vi.fn().mockImplementation(() => ({
      matches: prefersDark,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    })),
  })
}

async function mountTheme(domTheme?: 'light' | 'dark', prefersDark = false) {
  setupDom(domTheme)
  setupMatchMedia(prefersDark)
  vi.resetModules()
  const i18n = createI18n({legacy: false, locale: 'en-US', messages})
  const {useTheme} = await import('../useTheme')
  let result: ReturnType<typeof useTheme>
  mount(defineComponent({
    setup() { result = useTheme(); return {} },
    template: '<div/>',
  }), {global: {plugins: [i18n]}})
  await nextTick()
  return result!
}

beforeEach(() => {
  localStorage.clear()
  delete document.documentElement.dataset.theme
})
afterEach(() => { vi.restoreAllMocks() })

describe('useTheme – initialization', () => {
  it('picks up dark from data-theme attribute', async () => {
    const {mode, isDark} = await mountTheme('dark')
    expect(mode.value).toBe('dark')
    expect(isDark.value).toBe(true)
  })

  it('picks up light from data-theme attribute', async () => {
    const {mode, isDark} = await mountTheme('light')
    expect(mode.value).toBe('light')
    expect(isDark.value).toBe(false)
  })

  it('falls back to system dark when no data-theme and no localStorage', async () => {
    const {mode} = await mountTheme(undefined, true)
    expect(mode.value).toBe('dark')
  })

  it('falls back to system light when no data-theme and no localStorage', async () => {
    const {mode} = await mountTheme(undefined, false)
    expect(mode.value).toBe('light')
  })
})

describe('useTheme – label', () => {
  it('label is switch-to-light when dark', async () => {
    const {label} = await mountTheme('dark')
    expect(label.value).toBe('Switch to light theme')
  })

  it('label is switch-to-dark when light', async () => {
    const {label} = await mountTheme('light')
    expect(label.value).toBe('Switch to dark theme')
  })
})

describe('useTheme – toggle', () => {
  it('toggle flips dark to light', async () => {
    const {mode, isDark, toggle} = await mountTheme('dark')
    toggle()
    await nextTick()
    expect(mode.value).toBe('light')
    expect(isDark.value).toBe(false)
  })

  it('toggle flips light to dark', async () => {
    const {mode, isDark, toggle} = await mountTheme('light')
    toggle()
    await nextTick()
    expect(mode.value).toBe('dark')
    expect(isDark.value).toBe(true)
  })

  it('toggle persists to localStorage', async () => {
    const {toggle} = await mountTheme('light')
    toggle()
    expect(localStorage.getItem('orbien-server-ui-theme')).toBe('dark')
  })
})
