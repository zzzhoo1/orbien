import {describe, it, expect, vi, beforeEach} from 'vitest'
import {mount, flushPromises} from '@vue/test-utils'
import {createPinia, setActivePinia} from 'pinia'
import {createRouter, createMemoryHistory} from 'vue-router'
import {ref} from 'vue'
import AppHeader from '../AppHeader.vue'

vi.mock('@/assets/images/logo.png', () => ({default: 'logo.png'}))
vi.mock('@/assets/icon/github.svg?raw', () => ({default: '<svg class="github"/>'}))

vi.mock('@/components/ThemeToggle.vue', () => ({
  default: {template: '<div class="stub-theme-toggle"/>'},
}))
vi.mock('@/components/LocaleSwitcher.vue', () => ({
  default: {template: '<div class="stub-locale-switcher"/>'},
}))

vi.mock('@/composables/useLocale', () => ({
  useLocale: () => ({t: (key: string) => key}),
}))

const isMobileRef = ref(false)
const mobileOpenRef = ref(false)
const toggleCollapsed = vi.fn()
vi.mock('@/composables/useSidebar', () => ({
  useSidebar: () => ({
    isMobile: isMobileRef,
    mobileOpen: mobileOpenRef,
    toggleCollapsed,
  }),
}))

const mockAuth = {
  authenticated: false,
  username: '',
  logout: vi.fn(),
}
vi.mock('@/stores/auth', () => ({
  useAuthStore: () => mockAuth,
}))

function makeRouter() {
  return createRouter({
    history: createMemoryHistory(),
    routes: [
      {path: '/', component: {template: '<div/>'}},
      {path: '/login', name: 'login', component: {template: '<div/>'}},
    ],
  })
}

async function mountHeader() {
  const router = makeRouter()
  await router.push('/')
  const wrapper = mount(AppHeader, {global: {plugins: [createPinia(), router]}})
  await flushPromises()
  return {wrapper, router}
}

beforeEach(() => {
  setActivePinia(createPinia())
  vi.clearAllMocks()
  isMobileRef.value = false
  mobileOpenRef.value = false
  mockAuth.authenticated = false
  mockAuth.username = ''
  mockAuth.logout.mockResolvedValue(undefined)
})

describe('AppHeader – brand', () => {
  it('renders the logo image', async () => {
    const {wrapper} = await mountHeader()
    expect(wrapper.find('img.logo-img').exists()).toBe(true)
  })

  it('renders brand text Orbien', async () => {
    const {wrapper} = await mountHeader()
    expect(wrapper.text()).toContain('Orb')
    expect(wrapper.text()).toContain('ien')
  })

  it('brand block links to /', async () => {
    const {wrapper} = await mountHeader()
    expect(wrapper.find('a.brand-block').attributes('href')).toBe('/')
  })
})

describe('AppHeader – mobile menu button', () => {
  it('hides menu button on desktop', async () => {
    isMobileRef.value = false
    const {wrapper} = await mountHeader()
    expect(wrapper.find('.menu-btn').exists()).toBe(false)
  })

  it('shows menu button on mobile', async () => {
    isMobileRef.value = true
    const {wrapper} = await mountHeader()
    expect(wrapper.find('.menu-btn').exists()).toBe(true)
  })

  it('calls toggleCollapsed when menu button is clicked', async () => {
    isMobileRef.value = true
    const {wrapper} = await mountHeader()
    await wrapper.find('.menu-btn').trigger('click')
    expect(toggleCollapsed).toHaveBeenCalledOnce()
  })

  it('sets aria-expanded=true when mobileOpen is true', async () => {
    isMobileRef.value = true
    mobileOpenRef.value = true
    const {wrapper} = await mountHeader()
    expect(wrapper.find('.menu-btn').attributes('aria-expanded')).toBe('true')
  })

  it('sets aria-expanded=false when mobileOpen is false', async () => {
    isMobileRef.value = true
    mobileOpenRef.value = false
    const {wrapper} = await mountHeader()
    expect(wrapper.find('.menu-btn').attributes('aria-expanded')).toBe('false')
  })

  it('shows close icon (X) when mobileOpen is true', async () => {
    isMobileRef.value = true
    mobileOpenRef.value = true
    const {wrapper} = await mountHeader()
    const svgHtml = wrapper.find('.menu-btn svg').html()
    expect(svgHtml).toContain('M6 6l12 12')
  })

  it('shows hamburger icon when mobileOpen is false', async () => {
    isMobileRef.value = true
    mobileOpenRef.value = false
    const {wrapper} = await mountHeader()
    const svgHtml = wrapper.find('.menu-btn svg').html()
    expect(svgHtml).toContain('M4 7h16')
  })
})

describe('AppHeader – actions', () => {
  it('renders GitHub link with correct href', async () => {
    const {wrapper} = await mountHeader()
    const link = wrapper.find('a.github-link')
    expect(link.attributes('href')).toBe('https://github.com/orbien-org/orbien')
    expect(link.attributes('target')).toBe('_blank')
  })

  it('renders ThemeToggle and LocaleSwitcher stubs', async () => {
    const {wrapper} = await mountHeader()
    expect(wrapper.find('.stub-theme-toggle').exists()).toBe(true)
    expect(wrapper.find('.stub-locale-switcher').exists()).toBe(true)
  })

  it('hides user badge when not authenticated', async () => {
    const {wrapper} = await mountHeader()
    expect(wrapper.find('.user-badge').exists()).toBe(false)
  })

  it('shows user badge when authenticated', async () => {
    mockAuth.authenticated = true
    const {wrapper} = await mountHeader()
    expect(wrapper.find('.user-badge').exists()).toBe(true)
  })

  it('shows username when authenticated and username set', async () => {
    mockAuth.authenticated = true
    mockAuth.username = 'alice'
    const {wrapper} = await mountHeader()
    expect(wrapper.find('.user-name').text()).toBe('alice')
  })

  it('calls logout and redirects to /login on logout button click', async () => {
    mockAuth.authenticated = true
    const {wrapper, router} = await mountHeader()
    await wrapper.find('.logout-btn').trigger('click')
    await flushPromises()
    expect(mockAuth.logout).toHaveBeenCalledOnce()
    expect(router.currentRoute.value.path).toBe('/login')
  })
})
