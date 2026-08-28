import {describe, it, expect, vi, beforeEach} from 'vitest'
import {mount} from '@vue/test-utils'
import {defineComponent, nextTick} from 'vue'

const STORAGE_KEY = 'orbien-server-ui-sidebar-collapsed'

function mockMatchMedia(matches: boolean) {
  Object.defineProperty(window, 'matchMedia', {
    writable: true,
    value: vi.fn().mockImplementation(() => ({
      matches,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    })),
  })
}

async function mountSidebar(isMobileWindow = false) {
  mockMatchMedia(isMobileWindow)
  vi.resetModules()
  const {useSidebar} = await import('../useSidebar')
  let result: ReturnType<typeof useSidebar>
  mount(defineComponent({
    async setup() { result = useSidebar(); return {} },
    template: '<div/>',
  }))
  await nextTick()
  return result!
}

beforeEach(() => {
  localStorage.clear()
  document.body.style.overflow = ''
  vi.resetModules()
})

describe('useSidebar – desktop mode', () => {
  it('collapsed defaults to false when nothing in localStorage', async () => {
    const {collapsed} = await mountSidebar(false)
    expect(collapsed.value).toBe(false)
  })

  it('collapsed reads true from localStorage', async () => {
    localStorage.setItem(STORAGE_KEY, '1')
    const {collapsed} = await mountSidebar(false)
    expect(collapsed.value).toBe(true)
  })

  it('isMobile is false on desktop', async () => {
    const {isMobile} = await mountSidebar(false)
    expect(isMobile.value).toBe(false)
  })

  it('desktopCollapsed = collapsed when not mobile', async () => {
    localStorage.setItem(STORAGE_KEY, '1')
    const {collapsed, desktopCollapsed} = await mountSidebar(false)
    expect(desktopCollapsed.value).toBe(collapsed.value)
  })

  it('toggleCollapsed flips collapsed and persists to localStorage', async () => {
    const {collapsed, toggleCollapsed} = await mountSidebar(false)
    expect(collapsed.value).toBe(false)
    toggleCollapsed()
    expect(collapsed.value).toBe(true)
    expect(localStorage.getItem(STORAGE_KEY)).toBe('1')
    toggleCollapsed()
    expect(collapsed.value).toBe(false)
    expect(localStorage.getItem(STORAGE_KEY)).toBe('0')
  })
})

describe('useSidebar – mobile mode', () => {
  it('isMobile is true on mobile', async () => {
    const {isMobile} = await mountSidebar(true)
    expect(isMobile.value).toBe(true)
  })

  it('desktopCollapsed is false on mobile even if collapsed=true', async () => {
    localStorage.setItem(STORAGE_KEY, '1')
    const {desktopCollapsed} = await mountSidebar(true)
    expect(desktopCollapsed.value).toBe(false)
  })

  it('toggleCollapsed toggles mobileOpen on mobile', async () => {
    const {mobileOpen, toggleCollapsed} = await mountSidebar(true)
    expect(mobileOpen.value).toBe(false)
    toggleCollapsed()
    expect(mobileOpen.value).toBe(true)
    toggleCollapsed()
    expect(mobileOpen.value).toBe(false)
  })

  it('closeMobile sets mobileOpen to false', async () => {
    const {mobileOpen, toggleCollapsed, closeMobile} = await mountSidebar(true)
    toggleCollapsed()
    expect(mobileOpen.value).toBe(true)
    closeMobile()
    expect(mobileOpen.value).toBe(false)
  })

  it('body overflow hidden when mobileOpen=true and isMobile=true', async () => {
    const {toggleCollapsed} = await mountSidebar(true)
    toggleCollapsed()
    await nextTick()
    await nextTick()
    expect(document.body.style.overflow).toBe('hidden')
  })

  it('body overflow cleared when mobileOpen=false', async () => {
    const {toggleCollapsed, closeMobile} = await mountSidebar(true)
    toggleCollapsed()
    await nextTick()
    closeMobile()
    await nextTick()
    await nextTick()
    expect(document.body.style.overflow).toBe('')
  })
})
