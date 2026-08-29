import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { ref } from 'vue'
import AppHeader from '../AppHeader.vue'

// ── Static asset stubs ────────────────────────────────────────────────────────
vi.mock('@/assets/images/logo.png', () => ({ default: 'logo.png' }))
vi.mock('@/assets/icon/github.svg?raw', () => ({ default: '<svg data-testid="github-svg"></svg>' }))

// ── Child component stubs ─────────────────────────────────────────────────────
vi.mock('@/components/ThemeToggle.vue', () => ({
  default: { name: 'ThemeToggle', template: '<div data-testid="theme-toggle"/>' },
}))
vi.mock('@/components/LocaleSwitcher.vue', () => ({
  default: { name: 'LocaleSwitcher', template: '<div data-testid="locale-switcher"/>' },
}))

// ── Composable stubs ──────────────────────────────────────────────────────────
const mockToggleCollapsed = vi.fn()
const sidebarState = {
  isMobile: ref(false),
  mobileOpen: ref(false),
  toggleCollapsed: mockToggleCollapsed,
}

vi.mock('@/composables/useLocale', () => ({
  useLocale: () => ({ t: (key: string) => key }),
}))
vi.mock('@/composables/useSidebar', () => ({
  useSidebar: () => sidebarState,
}))

// ── Router stub ───────────────────────────────────────────────────────────────
const mockPush = vi.fn()
vi.mock('vue-router', () => ({
  RouterLink: { name: 'RouterLink', props: ['to'], template: '<a :href="to"><slot/></a>' },
  useRouter: () => ({ push: mockPush }),
}))

// ── Auth store mock ───────────────────────────────────────────────────────────
const authState = { authenticated: false, username: null as string | null }
const mockLogout = vi.fn()

vi.mock('@/stores/auth', () => ({
  useAuthStore: () => ({
    get authenticated() { return authState.authenticated },
    get username() { return authState.username },
    logout: mockLogout,
  }),
}))

// ── Factory ───────────────────────────────────────────────────────────────────
function factory() {
  return mount(AppHeader, {
    global: { plugins: [createPinia()] },
  })
}

// ─────────────────────────────────────────────────────────────────────────────
describe('AppHeader', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    sidebarState.isMobile.value = false
    sidebarState.mobileOpen.value = false
    authState.authenticated = false
    authState.username = null
    mockLogout.mockResolvedValue(undefined)
  })

  // ── Structure ──────────────────────────────────────────────────────────────
  it('renders a <header> element with class "top"', () => {
    const wrapper = factory()
    expect(wrapper.find('header.top').exists()).toBe(true)
  })

  it('renders the brand logo image', () => {
    const wrapper = factory()
    const img = wrapper.find('img')
    expect(img.exists()).toBe(true)
    expect(img.attributes('alt')).toBe('Orbien')
    expect(img.attributes('src')).toBe('logo.png')
  })

  it('renders brand title text "Orbien"', () => {
    const wrapper = factory()
    expect(wrapper.find('.brand-orb').text()).toBe('Orb')
    expect(wrapper.find('.brand-rest').text()).toBe('ien')
  })

  it('renders ThemeToggle and LocaleSwitcher', () => {
    const wrapper = factory()
    expect(wrapper.find('[data-testid="theme-toggle"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="locale-switcher"]').exists()).toBe(true)
  })

  it('renders GitHub link with correct href and attributes', () => {
    const wrapper = factory()
    const link = wrapper.find('a.github-link')
    expect(link.exists()).toBe(true)
    expect(link.attributes('href')).toBe('https://github.com/orbien-org/orbien')
    expect(link.attributes('target')).toBe('_blank')
    expect(link.attributes('rel')).toBe('noopener noreferrer')
  })

  // ── Mobile menu button ─────────────────────────────────────────────────────
  it('does NOT render mobile menu button when isMobile is false', () => {
    const wrapper = factory()
    expect(wrapper.find('button.menu-btn').exists()).toBe(false)
  })

  it('renders mobile menu button when isMobile is true', () => {
    sidebarState.isMobile.value = true
    const wrapper = factory()
    expect(wrapper.find('button.menu-btn').exists()).toBe(true)
  })

  it('calls toggleCollapsed when mobile menu button is clicked', async () => {
    sidebarState.isMobile.value = true
    const wrapper = factory()
    await wrapper.find('button.menu-btn').trigger('click')
    expect(mockToggleCollapsed).toHaveBeenCalledOnce()
  })

  it('sets aria-expanded="true" on menu button when mobileOpen is true', () => {
    sidebarState.isMobile.value = true
    sidebarState.mobileOpen.value = true
    const wrapper = factory()
    expect(wrapper.find('button.menu-btn').attributes('aria-expanded')).toBe('true')
  })

  it('sets aria-expanded="false" on menu button when mobileOpen is false', () => {
    sidebarState.isMobile.value = true
    sidebarState.mobileOpen.value = false
    const wrapper = factory()
    expect(wrapper.find('button.menu-btn').attributes('aria-expanded')).toBe('false')
  })

  // ── Auth: unauthenticated ──────────────────────────────────────────────────
  it('does NOT render user-badge when not authenticated', () => {
    authState.authenticated = false
    const wrapper = factory()
    expect(wrapper.find('.user-badge').exists()).toBe(false)
  })

  // ── Auth: authenticated ────────────────────────────────────────────────────
  it('renders user-badge when authenticated', () => {
    authState.authenticated = true
    authState.username = 'admin'
    const wrapper = factory()
    expect(wrapper.find('.user-badge').exists()).toBe(true)
  })

  it('renders username text when authenticated', () => {
    authState.authenticated = true
    authState.username = 'alice'
    const wrapper = factory()
    expect(wrapper.find('.user-name').text()).toBe('alice')
  })

  it('does NOT render username span when username is null', () => {
    authState.authenticated = true
    authState.username = null
    const wrapper = factory()
    expect(wrapper.find('.user-name').exists()).toBe(false)
  })

  it('renders logout button when authenticated', () => {
    authState.authenticated = true
    authState.username = 'admin'
    const wrapper = factory()
    expect(wrapper.find('button.logout-btn').exists()).toBe(true)
  })

  it('calls auth.logout and pushes to /login on logout click', async () => {
    authState.authenticated = true
    authState.username = 'admin'
    const wrapper = factory()
    await wrapper.find('button.logout-btn').trigger('click')
    await wrapper.vm.$nextTick()
    expect(mockLogout).toHaveBeenCalledOnce()
    expect(mockPush).toHaveBeenCalledWith('/login')
  })
})
