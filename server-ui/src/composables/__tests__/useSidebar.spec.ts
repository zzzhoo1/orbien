import {describe, it, expect, vi, beforeEach, afterEach} from 'vitest'
import {nextTick} from 'vue'

/**
 * useSidebar is a module-singleton composable (refs live at module scope).
 * We bypass vue-test-utils mount entirely to avoid the "async setup / no
 * Suspense" Vue warning – the composable itself has no template dependency,
 * so calling it directly is both simpler and warning-free.
 */

const STORAGE_KEY = 'orbien-server-ui-sidebar-collapsed'

function mockMatchMedia(matches: boolean) {
  Object.defineProperty(window, 'matchMedia', {
    writable: true,
    configurable: true,
    value: vi.fn().mockImplementation(() => ({
      matches,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    })),
  })
}

async function getSidebar(isMobileWindow = false) {
  mockMatchMedia(isMobileWindow)
  vi.resetModules()
  const {useSidebar} = await import('../useSidebar')
  return useSidebar()
}

beforeEach(() => {
  localStorage.clear()
  document.body.style.overflow = ''
  vi.resetModules()
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('useSidebar – desktop mode', () => {
  it('collapsed defaults to false when nothing in localStorage', async () => {
    const {collapsed} = await getSidebar(false)
    expect(collapsed.value).toBe(false)
  })

  it('collapsed reads true from localStorage', async () => {
    localStorage.setItem(STORAGE_KEY, '1')
    const {collapsed} = await getSidebar(false)
    expect(collapsed.value).toBe(true)
  })

  it('collapsed reads false when localStorage has "0"', async () => {
    localStorage.setItem(STORAGE_KEY, '0')
    const {collapsed} = await getSidebar(false)
    expect(collapsed.value).toBe(false)
  })

  it('isMobile is false on desktop', async () => {
    const {isMobile} = await getSidebar(false)
    expect(isMobile.value).toBe(false)
  })

  it('desktopCollapsed = collapsed when not mobile', async () => {
    localStorage.setItem(STORAGE_KEY, '1')
    const {collapsed, desktopCollapsed} = await getSidebar(false)
    expect(desktopCollapsed.value).toBe(collapsed.value)
  })

  it('desktopCollapsed is false when collapsed=false on desktop', async () => {
    const {desktopCollapsed} = await getSidebar(false)
    expect(desktopCollapsed.value).toBe(false)
  })

  it('toggleCollapsed flips collapsed and persists to localStorage', async () => {
    const {collapsed, toggleCollapsed} = await getSidebar(false)
    expect(collapsed.value).toBe(false)
    toggleCollapsed()
    expect(collapsed.value).toBe(true)
    expect(localStorage.getItem(STORAGE_KEY)).toBe('1')
    toggleCollapsed()
    expect(collapsed.value).toBe(false)
    expect(localStorage.getItem(STORAGE_KEY)).toBe('0')
  })

  it('mobileOpen stays false on desktop when toggling', async () => {
    const {mobileOpen, toggleCollapsed} = await getSidebar(false)
    toggleCollapsed()
    expect(mobileOpen.value).toBe(false)
  })
})

describe('useSidebar – mobile mode', () => {
  it('isMobile is true on mobile', async () => {
    const {isMobile} = await getSidebar(true)
    expect(isMobile.value).toBe(true)
  })

  it('desktopCollapsed is false on mobile even if collapsed=true', async () => {
    localStorage.setItem(STORAGE_KEY, '1')
    const {desktopCollapsed} = await getSidebar(true)
    expect(desktopCollapsed.value).toBe(false)
  })

  it('toggleCollapsed toggles mobileOpen on mobile', async () => {
    const {mobileOpen, toggleCollapsed} = await getSidebar(true)
    expect(mobileOpen.value).toBe(false)
    toggleCollapsed()
    expect(mobileOpen.value).toBe(true)
    toggleCollapsed()
    expect(mobileOpen.value).toBe(false)
  })

  it('toggleCollapsed does NOT change collapsed on mobile', async () => {
    const {collapsed, toggleCollapsed} = await getSidebar(true)
    const before = collapsed.value
    toggleCollapsed()
    expect(collapsed.value).toBe(before)
  })

  it('closeMobile sets mobileOpen to false', async () => {
    const {mobileOpen, toggleCollapsed, closeMobile} = await getSidebar(true)
    toggleCollapsed()
    expect(mobileOpen.value).toBe(true)
    closeMobile()
    expect(mobileOpen.value).toBe(false)
  })

  it('body overflow hidden when mobileOpen=true and isMobile=true', async () => {
    const {toggleCollapsed} = await getSidebar(true)
    toggleCollapsed()
    await nextTick()
    await nextTick()
    expect(document.body.style.overflow).toBe('hidden')
  })

  it('body overflow cleared when mobileOpen=false', async () => {
    const {toggleCollapsed, closeMobile} = await getSidebar(true)
    toggleCollapsed()
    await nextTick()
    closeMobile()
    await nextTick()
    await nextTick()
    expect(document.body.style.overflow).toBe('')
  })
})
