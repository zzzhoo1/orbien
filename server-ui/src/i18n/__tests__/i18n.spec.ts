import {describe, it, expect, vi, beforeEach, afterEach} from 'vitest'

// ── locales.ts ─────────────────────────────────────────────────────────────────────────────────
describe('locales – isAppLocale', () => {
  let isAppLocale: (v: string) => boolean

  beforeEach(async () => {
    vi.resetModules()
    ;({isAppLocale} = await import('../locales'))
  })

  it('returns true for "zh-CN"', () => {
    expect(isAppLocale('zh-CN')).toBe(true)
  })

  it('returns true for "en-US"', () => {
    expect(isAppLocale('en-US')).toBe(true)
  })

  it('returns false for unsupported locale', () => {
    expect(isAppLocale('fr-FR')).toBe(false)
  })

  it('returns false for empty string', () => {
    expect(isAppLocale('')).toBe(false)
  })

  it('returns false for partial match like "zh"', () => {
    expect(isAppLocale('zh')).toBe(false)
  })

  it('returns false for case-variant like "en-us"', () => {
    expect(isAppLocale('en-us')).toBe(false)
  })
})

describe('locales – LOCALE_META', () => {
  it('has correct htmlLang for zh-CN', async () => {
    const {LOCALE_META} = await import('../locales')
    expect(LOCALE_META['zh-CN'].htmlLang).toBe('zh-CN')
  })

  it('has correct htmlLang for en-US', async () => {
    const {LOCALE_META} = await import('../locales')
    expect(LOCALE_META['en-US'].htmlLang).toBe('en')
  })

  it('has nativeLabel for zh-CN', async () => {
    const {LOCALE_META} = await import('../locales')
    expect(LOCALE_META['zh-CN'].nativeLabel).toBe('\u4e2d\u6587')
  })

  it('has nativeLabel for en-US', async () => {
    const {LOCALE_META} = await import('../locales')
    expect(LOCALE_META['en-US'].nativeLabel).toBe('English')
  })
})

// ── index.ts: resolveLocale ───────────────────────────────────────────────────────────────────────────────
describe('resolveLocale', () => {
  let resolveLocale: (preferred?: string | null) => string
  const STORAGE_KEY = 'orbien-server-ui-locale'

  beforeEach(async () => {
    vi.resetModules()
    localStorage.clear()
    Object.defineProperty(navigator, 'languages', {value: [], configurable: true})
    Object.defineProperty(navigator, 'language', {value: 'en-US', configurable: true})
    ;({resolveLocale} = await import('../index'))
  })

  afterEach(() => {
    localStorage.clear()
  })

  it('returns preferred locale when valid', async () => {
    expect(resolveLocale('zh-CN')).toBe('zh-CN')
  })

  it('returns preferred en-US when valid', async () => {
    expect(resolveLocale('en-US')).toBe('en-US')
  })

  it('ignores invalid preferred and falls back to localStorage', async () => {
    localStorage.setItem(STORAGE_KEY, 'zh-CN')
    expect(resolveLocale('fr-FR')).toBe('zh-CN')
  })

  it('ignores null preferred and uses localStorage', async () => {
    localStorage.setItem(STORAGE_KEY, 'en-US')
    expect(resolveLocale(null)).toBe('en-US')
  })

  it('ignores invalid localStorage value and falls back to browser', async () => {
    localStorage.setItem(STORAGE_KEY, 'xx-XX')
    Object.defineProperty(navigator, 'languages', {value: ['en-US'], configurable: true})
    vi.resetModules()
    ;({resolveLocale} = await import('../index'))
    expect(resolveLocale()).toBe('en-US')
  })

  it('detects zh-CN from navigator.languages starting with zh', async () => {
    localStorage.clear()
    Object.defineProperty(navigator, 'languages', {value: ['zh-TW'], configurable: true})
    vi.resetModules()
    ;({resolveLocale} = await import('../index'))
    expect(resolveLocale()).toBe('zh-CN')
  })

  it('detects en-US from navigator.languages starting with en', async () => {
    localStorage.clear()
    Object.defineProperty(navigator, 'languages', {value: ['en-GB'], configurable: true})
    vi.resetModules()
    ;({resolveLocale} = await import('../index'))
    expect(resolveLocale()).toBe('en-US')
  })

  it('falls back to DEFAULT_LOCALE (zh-CN) for unknown browser language', async () => {
    localStorage.clear()
    Object.defineProperty(navigator, 'languages', {value: ['fr-FR'], configurable: true})
    vi.resetModules()
    ;({resolveLocale} = await import('../index'))
    expect(resolveLocale()).toBe('zh-CN')
  })

  it('uses navigator.language when navigator.languages is empty', async () => {
    localStorage.clear()
    Object.defineProperty(navigator, 'languages', {value: [], configurable: true})
    Object.defineProperty(navigator, 'language', {value: 'zh-CN', configurable: true})
    vi.resetModules()
    ;({resolveLocale} = await import('../index'))
    expect(resolveLocale()).toBe('zh-CN')
  })
})

// ── index.ts: applyDocumentLocale ──────────────────────────────────────────────────────────────────────────
describe('applyDocumentLocale', () => {
  const STORAGE_KEY = 'orbien-server-ui-locale'

  beforeEach(() => {
    localStorage.clear()
    document.documentElement.lang = ''
  })

  afterEach(() => {
    localStorage.clear()
  })

  it('sets document.documentElement.lang to zh-CN', async () => {
    vi.resetModules()
    const {applyDocumentLocale} = await import('../index')
    applyDocumentLocale('zh-CN')
    expect(document.documentElement.lang).toBe('zh-CN')
  })

  it('sets document.documentElement.lang to en for en-US', async () => {
    vi.resetModules()
    const {applyDocumentLocale} = await import('../index')
    applyDocumentLocale('en-US')
    expect(document.documentElement.lang).toBe('en')
  })

  it('persists locale to localStorage', async () => {
    vi.resetModules()
    const {applyDocumentLocale} = await import('../index')
    applyDocumentLocale('en-US')
    expect(localStorage.getItem(STORAGE_KEY)).toBe('en-US')
  })

  it('overwrites previous localStorage value', async () => {
    localStorage.setItem(STORAGE_KEY, 'zh-CN')
    vi.resetModules()
    const {applyDocumentLocale} = await import('../index')
    applyDocumentLocale('en-US')
    expect(localStorage.getItem(STORAGE_KEY)).toBe('en-US')
  })
})

// ── index.ts: setLocale ────────────────────────────────────────────────────────────────────────────────────────
describe('setLocale', () => {
  const STORAGE_KEY = 'orbien-server-ui-locale'

  beforeEach(() => {
    localStorage.clear()
    document.documentElement.lang = ''
  })

  afterEach(() => {
    localStorage.clear()
  })

  it('updates i18n.global.locale to en-US', async () => {
    vi.resetModules()
    const {setLocale, i18n} = await import('../index')
    setLocale('en-US')
    expect(i18n.global.locale.value).toBe('en-US')
  })

  it('updates i18n.global.locale to zh-CN', async () => {
    vi.resetModules()
    const {setLocale, i18n} = await import('../index')
    setLocale('zh-CN')
    expect(i18n.global.locale.value).toBe('zh-CN')
  })

  it('persists to localStorage via applyDocumentLocale', async () => {
    vi.resetModules()
    const {setLocale} = await import('../index')
    setLocale('en-US')
    expect(localStorage.getItem(STORAGE_KEY)).toBe('en-US')
  })

  it('updates document lang via applyDocumentLocale', async () => {
    vi.resetModules()
    const {setLocale} = await import('../index')
    setLocale('zh-CN')
    expect(document.documentElement.lang).toBe('zh-CN')
  })

  it('can switch locale back and forth', async () => {
    vi.resetModules()
    const {setLocale, i18n} = await import('../index')
    setLocale('en-US')
    expect(i18n.global.locale.value).toBe('en-US')
    setLocale('zh-CN')
    expect(i18n.global.locale.value).toBe('zh-CN')
  })
})

// ── constants: NAV_ITEMS ───────────────────────────────────────────────────────────────────────────────────
describe('NAV_ITEMS', () => {
  it('has exactly 5 items', async () => {
    const {NAV_ITEMS} = await import('@/constants/menus')
    expect(NAV_ITEMS).toHaveLength(5)
  })

  it('first item is dashboard at /', async () => {
    const {NAV_ITEMS} = await import('@/constants/menus')
    expect(NAV_ITEMS[0]).toMatchObject({key: 'dashboard', path: '/', icon: 'dashboard'})
  })

  it('second item is clients at /clients', async () => {
    const {NAV_ITEMS} = await import('@/constants/menus')
    expect(NAV_ITEMS[1]).toMatchObject({key: 'clients', path: '/clients', icon: 'clients'})
  })

  it('third item is tunnels at /tunnels', async () => {
    const {NAV_ITEMS} = await import('@/constants/menus')
    expect(NAV_ITEMS[2]).toMatchObject({key: 'tunnels', path: '/tunnels', icon: 'tunnels'})
  })

  it('fourth item is tokens at /tokens', async () => {
    const {NAV_ITEMS} = await import('@/constants/menus')
    expect(NAV_ITEMS[3]).toMatchObject({key: 'tokens', path: '/tokens', icon: 'tokens'})
  })

  it('fifth item is settings at /settings', async () => {
    const {NAV_ITEMS} = await import('@/constants/menus')
    expect(NAV_ITEMS[4]).toMatchObject({key: 'settings', path: '/settings', icon: 'settings'})
  })

  it('all items have labelKey prefixed with nav.', async () => {
    const {NAV_ITEMS} = await import('@/constants/menus')
    for (const item of NAV_ITEMS) {
      expect(item.labelKey).toBe(`nav.${item.key}`)
    }
  })
})
