import {beforeEach, describe, expect, it, vi} from 'vitest'
import {mount} from '@vue/test-utils'
import {ref} from 'vue'
import AppSidebar from '../AppSidebar.vue'

// ── SVG asset stubs ───────────────────────────────────────────────────────────────────
vi.mock('@/assets/icon/computer.svg?raw', () => ({default: '<svg data-testid="computer-svg"></svg>'}))
vi.mock('@/assets/icon/share.svg?raw', () => ({default: '<svg data-testid="share-svg"></svg>'}))
vi.mock('@/assets/icon/user.svg?raw', () => ({default: '<svg data-testid="user-svg"></svg>'}))
vi.mock('@/assets/icon/arrow-left.svg?raw', () => ({default: '<svg data-testid="arrow-left-svg"></svg>'}))
vi.mock('@/assets/icon/arrow-right.svg?raw', () => ({default: '<svg data-testid="arrow-right-svg"></svg>'}))

// ── Composable stubs ────────────────────────────────────────────────────────────────
const mockToggleCollapsed = vi.fn()
const mockCloseMobile = vi.fn()
const sidebarState = {
  collapsed: ref(false),
  mobileOpen: ref(false),
  isMobile: ref(false),
  desktopCollapsed: ref(false),
  toggleCollapsed: mockToggleCollapsed,
  closeMobile: mockCloseMobile,
}

vi.mock('@/composables/useLocale', () => ({
  useLocale: () => ({t: (key: string) => key}),
}))
vi.mock('@/composables/useSidebar', () => ({
  useSidebar: () => sidebarState,
}))

// ── Router stub ─────────────────────────────────────────────────────────────────────
const routeState = {path: '/'}
vi.mock('vue-router', () => ({
  RouterLink: {
    name: 'RouterLink',
    props: ['to'],
    template: '<a :href="to" class="router-link"><slot/></a>',
  },
  useRoute: () => routeState,
}))

// ── Factory ─────────────────────────────────────────────────────────────────────────
function factory() {
  return mount(AppSidebar)
}

// ── Suite ────────────────────────────────────────────────────────────────────────
describe('AppSidebar', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    sidebarState.collapsed.value = false
    sidebarState.mobileOpen.value = false
    sidebarState.isMobile.value = false
    sidebarState.desktopCollapsed.value = false
    routeState.path = '/'
  })

  // ── Structure ────────────────────────────────────────────────────────────

  it('renders an <aside> element', () => {
    const wrapper = factory()
    expect(wrapper.find('aside.sidebar').exists()).toBe(true)
  })

  it('sets aria-label on aside from t("nav.menu")', () => {
    const wrapper = factory()
    expect(wrapper.find('aside').attributes('aria-label')).toBe('nav.menu')
  })

  it('renders NAV_ITEMS as side-link anchors', () => {
    const wrapper = factory()
    const links = wrapper.findAll('.side-link')
    expect(links.length).toBeGreaterThan(0)
  })

  it('renders side-label spans for each nav item when not collapsed', () => {
    sidebarState.desktopCollapsed.value = false
    const wrapper = factory()
    const labels = wrapper.findAll('.side-label')
    expect(labels.length).toBeGreaterThan(0)
  })

  // ── Collapse state ───────────────────────────────────────────────────────

  it('adds is-collapsed class when desktopCollapsed is true', () => {
    sidebarState.desktopCollapsed.value = true
    const wrapper = factory()
    expect(wrapper.find('aside').classes()).toContain('is-collapsed')
  })

  it('does not add is-collapsed class when desktopCollapsed is false', () => {
    sidebarState.desktopCollapsed.value = false
    const wrapper = factory()
    expect(wrapper.find('aside').classes()).not.toContain('is-collapsed')
  })

  // ── Collapse button (desktop) ──────────────────────────────────────────────

  it('renders collapse button on desktop', () => {
    sidebarState.isMobile.value = false
    const wrapper = factory()
    expect(wrapper.find('button.sidebar-collapse').exists()).toBe(true)
  })

  it('does NOT render collapse button on mobile', () => {
    sidebarState.isMobile.value = true
    const wrapper = factory()
    expect(wrapper.find('button.sidebar-collapse').exists()).toBe(false)
  })

  it('collapse button shows arrow-right icon when collapsed', () => {
    sidebarState.collapsed.value = true
    const wrapper = factory()
    const btn = wrapper.find('button.sidebar-collapse')
    expect(btn.html()).toContain('arrow-right-svg')
  })

  it('collapse button shows arrow-left icon when expanded', () => {
    sidebarState.collapsed.value = false
    const wrapper = factory()
    const btn = wrapper.find('button.sidebar-collapse')
    expect(btn.html()).toContain('arrow-left-svg')
  })

  it('calls toggleCollapsed when collapse button is clicked', async () => {
    sidebarState.isMobile.value = false
    const wrapper = factory()
    await wrapper.find('button.sidebar-collapse').trigger('click')
    expect(mockToggleCollapsed).toHaveBeenCalledOnce()
  })

  it('collapse button aria-label reflects collapsed state', () => {
    sidebarState.collapsed.value = true
    const wrapper = factory()
    const btn = wrapper.find('button.sidebar-collapse')
    expect(btn.attributes('aria-label')).toBe('actions.expandSidebar')
  })

  it('collapse button aria-label reflects expanded state', () => {
    sidebarState.collapsed.value = false
    const wrapper = factory()
    const btn = wrapper.find('button.sidebar-collapse')
    expect(btn.attributes('aria-label')).toBe('actions.collapseSidebar')
  })

  // ── Mobile behaviour ───────────────────────────────────────────────────────

  it('does NOT render backdrop when isMobile is false', () => {
    sidebarState.isMobile.value = false
    const wrapper = factory()
    expect(wrapper.find('.sidebar-backdrop').exists()).toBe(false)
  })

  it('does NOT render backdrop when isMobile but mobileOpen is false', () => {
    sidebarState.isMobile.value = true
    sidebarState.mobileOpen.value = false
    const wrapper = factory()
    expect(wrapper.find('.sidebar-backdrop').exists()).toBe(false)
  })

  it('renders backdrop when isMobile and mobileOpen are both true', () => {
    sidebarState.isMobile.value = true
    sidebarState.mobileOpen.value = true
    const wrapper = factory()
    expect(wrapper.find('.sidebar-backdrop').exists()).toBe(true)
  })

  it('adds is-mobile-open class when isMobile and mobileOpen are true', () => {
    sidebarState.isMobile.value = true
    sidebarState.mobileOpen.value = true
    const wrapper = factory()
    expect(wrapper.find('aside').classes()).toContain('is-mobile-open')
  })

  it('adds is-mobile class when isMobile is true', () => {
    sidebarState.isMobile.value = true
    const wrapper = factory()
    expect(wrapper.find('aside').classes()).toContain('is-mobile')
  })

  it('calls closeMobile when backdrop is clicked', async () => {
    sidebarState.isMobile.value = true
    sidebarState.mobileOpen.value = true
    const wrapper = factory()
    await wrapper.find('.sidebar-backdrop').trigger('click')
    expect(mockCloseMobile).toHaveBeenCalledOnce()
  })

  it('calls closeMobile on nav link click when isMobile is true', async () => {
    sidebarState.isMobile.value = true
    sidebarState.mobileOpen.value = true
    const wrapper = factory()
    const firstLink = wrapper.find('.side-link')
    await firstLink.trigger('click')
    expect(mockCloseMobile).toHaveBeenCalledOnce()
  })

  it('does NOT call closeMobile on nav link click when isMobile is false', async () => {
    sidebarState.isMobile.value = false
    const wrapper = factory()
    const firstLink = wrapper.find('.side-link')
    await firstLink.trigger('click')
    expect(mockCloseMobile).not.toHaveBeenCalled()
  })

  // ── Active link ───────────────────────────────────────────────────────────────

  it('marks the root path link as active when route.path is "/"', () => {
    routeState.path = '/'
    const wrapper = factory()
    // The first nav item ("/") should have class "active"
    const links = wrapper.findAll('.side-link')
    const rootLink = links.find((l) => l.attributes('href') === '/')
    expect(rootLink?.classes()).toContain('active')
  })

  it('marks /tunnels link as active when route.path starts with "/tunnels"', () => {
    routeState.path = '/tunnels/my-tunnel'
    const wrapper = factory()
    const links = wrapper.findAll('.side-link')
    const tunnelLink = links.find((l) => l.attributes('href') === '/tunnels')
    expect(tunnelLink?.classes()).toContain('active')
  })
})
