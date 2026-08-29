import {describe, it, expect, vi, beforeEach, afterEach} from 'vitest'
import {mount, flushPromises} from '@vue/test-utils'
import {createPinia, setActivePinia} from 'pinia'
import {createRouter, createMemoryHistory} from 'vue-router'
import {ref, computed, reactive} from 'vue'
import DefaultLayout from '../DefaultLayout.vue'
import {ApiError} from '@/api/errors'

vi.mock('@/layouts/AppHeader.vue', () => ({
  default: {template: '<div class="stub-header"/>'},
}))
vi.mock('@/layouts/AppSidebar.vue', () => ({
  default: {template: '<div class="stub-sidebar"/>'},
}))
vi.mock('@/components/InlineAlert.vue', () => ({
  default: {
    template: '<div class="stub-inline-alert" :title="title"><slot/></div>',
    props: ['variant', 'title', 'closable'],
    emits: ['close'],
  },
}))

vi.mock('@/composables/useLocale', () => ({
  useLocale: () => ({
    t: (key: string, params?: Record<string, unknown>) => {
      if (params && Object.keys(params).length) return `${key}:${JSON.stringify(params)}`
      return key
    },
  }),
}))

const desktopCollapsedRef = ref(false)
const isMobileRef = ref(false)
vi.mock('@/composables/useSidebar', () => ({
  useSidebar: () => ({
    desktopCollapsed: desktopCollapsedRef,
    isMobile: isMobileRef,
  }),
}))

const messageRef = ref<{type: string; text: string} | null>(null)
vi.mock('@/composables/useToast', () => ({
  useToast: () => ({message: computed(() => messageRef.value)}),
}))

const mockStoreState = reactive({
  error: null as ApiError | null,
  refresh: vi.fn(),
})
vi.mock('@/stores/dashboard', () => ({
  useDashboardStore: () => mockStoreState,
}))

function makeRouter() {
  return createRouter({
    history: createMemoryHistory(),
    routes: [
      {path: '/', component: {template: '<div class="route-content"/>'}},
      {path: '/page', component: {template: '<div/>'}},
    ],
  })
}

async function mountLayout() {
  const router = makeRouter()
  await router.push('/')
  const wrapper = mount(DefaultLayout, {global: {plugins: [createPinia(), router]}})
  await flushPromises()
  return {wrapper, router}
}

beforeEach(() => {
  setActivePinia(createPinia())
  vi.clearAllMocks()
  vi.useFakeTimers()
  mockStoreState.error = null
  mockStoreState.refresh = vi.fn().mockResolvedValue(undefined)
  messageRef.value = null
  desktopCollapsedRef.value = false
  isMobileRef.value = false
})

afterEach(() => {
  vi.useRealTimers()
})

describe('DefaultLayout – shell structure', () => {
  it('renders header and sidebar stubs', async () => {
    const {wrapper} = await mountLayout()
    expect(wrapper.find('.stub-header').exists()).toBe(true)
    expect(wrapper.find('.stub-sidebar').exists()).toBe(true)
  })

  it('renders RouterView inside .content', async () => {
    const {wrapper} = await mountLayout()
    expect(wrapper.find('main.content').exists()).toBe(true)
    expect(wrapper.find('.route-content').exists()).toBe(true)
  })

  it('applies sidebar-collapsed class when desktopCollapsed is true', async () => {
    desktopCollapsedRef.value = true
    const {wrapper} = await mountLayout()
    expect(wrapper.find('.shell').classes()).toContain('sidebar-collapsed')
  })

  it('does NOT apply sidebar-collapsed class when desktopCollapsed is false', async () => {
    desktopCollapsedRef.value = false
    const {wrapper} = await mountLayout()
    expect(wrapper.find('.shell').classes()).not.toContain('sidebar-collapsed')
  })

  it('applies sidebar-mobile class when isMobile is true', async () => {
    isMobileRef.value = true
    const {wrapper} = await mountLayout()
    expect(wrapper.find('.shell').classes()).toContain('sidebar-mobile')
  })

  it('does NOT apply sidebar-mobile class when isMobile is false', async () => {
    isMobileRef.value = false
    const {wrapper} = await mountLayout()
    expect(wrapper.find('.shell').classes()).not.toContain('sidebar-mobile')
  })

  it('can apply both sidebar-collapsed and sidebar-mobile simultaneously', async () => {
    desktopCollapsedRef.value = true
    isMobileRef.value = true
    const {wrapper} = await mountLayout()
    const classes = wrapper.find('.shell').classes()
    expect(classes).toContain('sidebar-collapsed')
    expect(classes).toContain('sidebar-mobile')
  })
})

describe('DefaultLayout – error alert', () => {
  it('hides alert when store.error is null', async () => {
    const {wrapper} = await mountLayout()
    expect(wrapper.find('.stub-inline-alert').exists()).toBe(false)
  })

  it('shows alert with http error text', async () => {
    mockStoreState.error = new ApiError('http', {status: 503})
    const {wrapper} = await mountLayout()
    expect(wrapper.find('.stub-inline-alert').exists()).toBe(true)
    expect(wrapper.find('.stub-inline-alert').attributes('title')).toContain('errors.http')
  })

  it('shows alert with api error msg string directly', async () => {
    mockStoreState.error = new ApiError('api', {msg: 'custom error msg'})
    const {wrapper} = await mountLayout()
    expect(wrapper.find('.stub-inline-alert').attributes('title')).toBe('custom error msg')
  })

  it('falls back to generic code for api error with empty msg', async () => {
    mockStoreState.error = new ApiError('api', {msg: ''})
    const {wrapper} = await mountLayout()
    expect(wrapper.find('.stub-inline-alert').attributes('title')).toBe('errors.api')
  })

  it('falls back to generic code for api error with no msg param', async () => {
    mockStoreState.error = new ApiError('api')
    const {wrapper} = await mountLayout()
    expect(wrapper.find('.stub-inline-alert').attributes('title')).toBe('errors.api')
  })

  it('shows alert with generic code for non-http/api errors', async () => {
    mockStoreState.error = new ApiError('unauthorized')
    const {wrapper} = await mountLayout()
    expect(wrapper.find('.stub-inline-alert').attributes('title')).toBe('errors.unauthorized')
  })

  it('dismisses alert when close event is emitted', async () => {
    mockStoreState.error = new ApiError('unauthorized')
    const {wrapper} = await mountLayout()
    expect(wrapper.find('.stub-inline-alert').exists()).toBe(true)
    await wrapper.findComponent({name: 'InlineAlert'}).vm.$emit('close')
    await flushPromises()
    expect(wrapper.find('.stub-inline-alert').exists()).toBe(false)
  })

  it('re-shows alert when a new error arrives after dismiss', async () => {
    mockStoreState.error = new ApiError('unauthorized')
    const {wrapper} = await mountLayout()
    await wrapper.findComponent({name: 'InlineAlert'}).vm.$emit('close')
    await flushPromises()
    expect(wrapper.find('.stub-inline-alert').exists()).toBe(false)
    mockStoreState.error = new ApiError('http', {status: 500})
    await flushPromises()
    expect(wrapper.find('.stub-inline-alert').exists()).toBe(true)
  })
})

describe('DefaultLayout – toast', () => {
  it('hides toast when message is null', async () => {
    const {wrapper} = await mountLayout()
    expect(wrapper.find('.global-toast').exists()).toBe(false)
  })

  it('shows toast when message is set', async () => {
    messageRef.value = {type: 'info', text: 'Saved!'}
    const {wrapper} = await mountLayout()
    expect(wrapper.find('.global-toast').exists()).toBe(true)
    expect(wrapper.find('.global-toast').text()).toBe('Saved!')
  })

  it('applies error class to error toast', async () => {
    messageRef.value = {type: 'error', text: 'Failed'}
    const {wrapper} = await mountLayout()
    expect(wrapper.find('.global-toast').classes()).toContain('error')
  })

  it('does not apply error class to info toast', async () => {
    messageRef.value = {type: 'info', text: 'Hello'}
    const {wrapper} = await mountLayout()
    expect(wrapper.find('.global-toast').classes()).not.toContain('error')
  })

  it('toast has role=status and aria-live=polite', async () => {
    messageRef.value = {type: 'info', text: 'Hi'}
    const {wrapper} = await mountLayout()
    const toast = wrapper.find('.global-toast')
    expect(toast.attributes('role')).toBe('status')
    expect(toast.attributes('aria-live')).toBe('polite')
  })

  it('hides toast after messageRef becomes null reactively', async () => {
    messageRef.value = {type: 'info', text: 'Hi'}
    const {wrapper} = await mountLayout()
    expect(wrapper.find('.global-toast').exists()).toBe(true)
    messageRef.value = null
    await flushPromises()
    expect(wrapper.find('.global-toast').exists()).toBe(false)
  })
})

describe('DefaultLayout – refresh timer', () => {
  it('calls store.refresh on mount', async () => {
    await mountLayout()
    expect(mockStoreState.refresh).toHaveBeenCalledOnce()
  })

  it('calls store.refresh again after 5 seconds', async () => {
    await mountLayout()
    mockStoreState.refresh.mockClear()
    vi.advanceTimersByTime(5000)
    await flushPromises()
    expect(mockStoreState.refresh).toHaveBeenCalledOnce()
  })

  it('calls store.refresh three more times after 15 seconds', async () => {
    await mountLayout()
    mockStoreState.refresh.mockClear()
    vi.advanceTimersByTime(15000)
    await flushPromises()
    expect(mockStoreState.refresh).toHaveBeenCalledTimes(3)
  })

  it('clears the interval on unmount', async () => {
    const clearSpy = vi.spyOn(window, 'clearInterval')
    const {wrapper} = await mountLayout()
    wrapper.unmount()
    expect(clearSpy).toHaveBeenCalled()
  })

  it('does NOT call refresh after unmount', async () => {
    const {wrapper} = await mountLayout()
    wrapper.unmount()
    mockStoreState.refresh.mockClear()
    vi.advanceTimersByTime(10000)
    await flushPromises()
    expect(mockStoreState.refresh).not.toHaveBeenCalled()
  })
})
