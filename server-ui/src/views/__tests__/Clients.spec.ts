import {describe, it, expect, vi, beforeEach} from 'vitest'
import {mount, flushPromises} from '@vue/test-utils'
import {createPinia, setActivePinia} from 'pinia'
import {createRouter, createMemoryHistory} from 'vue-router'
import Clients from '../Clients.vue'
import * as api from '@/api'

vi.mock('@/components/AppIcon.vue', () => ({
  default: {template: '<span class="stub-app-icon"/>', props: ['name']},
}))
vi.mock('@/components/EmptyState.vue', () => ({
  default: {template: '<div class="stub-empty-state">{{ title }}</div>', props: ['type', 'title', 'desc']},
}))
vi.mock('@/components/OsBadge.vue', () => ({
  default: {template: '<span class="stub-os-badge"/>', props: ['os', 'arch', 'iconOnly', 'textOnly']},
}))
vi.mock('@/components/PaginationBar.vue', () => ({
  default: {template: '<div class="stub-pagination"/>', props: ['page', 'pageSize', 'total']},
}))
vi.mock('@/components/StatusBadge.vue', () => ({
  default: {template: '<span class="stub-status-badge"/>', props: ['status', 'label']},
}))

vi.mock('@/composables/useLocale', () => ({
  useLocale: () => ({t: (key: string) => key}),
}))

vi.mock('@/composables/usePresence', () => ({
  usePresence: () => ({
    isOnline: (status: unknown) => status === 'online',
    statusLabel: (status: unknown) => String(status ?? ''),
    formatSeen: (seconds: unknown, online: boolean) => `${online ? 'online' : 'offline'}:${seconds ?? 0}`,
  }),
}))

const mockShowToast = vi.fn()
vi.mock('@/composables/useToast', () => ({
  useToast: () => ({show: mockShowToast}),
}))

vi.mock('@/api', () => ({kickClient: vi.fn()}))

const mockStore = {
  clients: [] as unknown[],
  refresh: vi.fn(),
}
vi.mock('@/stores/dashboard', () => ({
  useDashboardStore: () => mockStore,
}))

function makeClient(overrides: Record<string, unknown> = {}) {
  return {
    sessionId: 'client-alpha',
    status: 'online',
    os: 'linux',
    arch: 'amd64',
    hostname: 'host-a',
    user: 'alice',
    version: '1.2.3',
    tunnelCount: 2,
    clientIP: '10.0.0.2',
    connectedSecs: 60,
    ...overrides,
  }
}

function makeRouter() {
  return createRouter({
    history: createMemoryHistory(),
    routes: [
      {path: '/', component: {template: '<div/>'}},
      {path: '/clients', component: Clients},
      {path: '/clients/:sessionId', name: 'client-detail', component: {template: '<div/>'}},
    ],
  })
}

async function mountClients() {
  const router = makeRouter()
  await router.push('/clients')
  const wrapper = mount(Clients, {global: {plugins: [createPinia(), router]}})
  await flushPromises()
  return {wrapper, router}
}

beforeEach(() => {
  setActivePinia(createPinia())
  vi.clearAllMocks()
  mockStore.clients = []
  mockStore.refresh.mockResolvedValue(undefined)
  vi.mocked(api.kickClient).mockResolvedValue(undefined)
})

describe('Clients – empty state', () => {
  it('shows clients.empty when no clients exist', async () => {
    const {wrapper} = await mountClients()
    expect(wrapper.text()).toContain('clients.empty')
  })

  it('renders no client cards when list is empty', async () => {
    const {wrapper} = await mountClients()
    expect(wrapper.findAll('.client-card')).toHaveLength(0)
  })
})

describe('Clients – status filters', () => {
  it('renders all, online, and offline filter chips', async () => {
    const {wrapper} = await mountClients()
    expect(wrapper.findAll('.filter-chip')).toHaveLength(3)
  })

  it('makes all filter active by default', async () => {
    const {wrapper} = await mountClients()
    expect(wrapper.find('.filter-chip.active').text()).toContain('clients.filterAll')
  })

  it('shows all, online, and offline counts', async () => {
    mockStore.clients = [
      makeClient({sessionId: 'a', status: 'online'}),
      makeClient({sessionId: 'b', status: 'offline'}),
      makeClient({sessionId: 'c', status: 'online'}),
    ]
    const {wrapper} = await mountClients()
    const chips = wrapper.findAll('.filter-chip')
    expect(chips[0].text()).toContain('3')
    expect(chips[1].text()).toContain('2')
    expect(chips[2].text()).toContain('1')
  })

  it('filters visible cards to online clients', async () => {
    mockStore.clients = [
      makeClient({sessionId: 'online-client', status: 'online'}),
      makeClient({sessionId: 'offline-client', status: 'offline'}),
    ]
    const {wrapper} = await mountClients()
    await wrapper.findAll('.filter-chip')[1].trigger('click')
    await flushPromises()
    expect(wrapper.findAll('.client-card')).toHaveLength(1)
    expect(wrapper.text()).toContain('online-client')
    expect(wrapper.text()).not.toContain('offline-client')
  })

  it('filters visible cards to offline clients', async () => {
    mockStore.clients = [
      makeClient({sessionId: 'online-client', status: 'online'}),
      makeClient({sessionId: 'offline-client', status: 'offline'}),
    ]
    const {wrapper} = await mountClients()
    await wrapper.findAll('.filter-chip')[2].trigger('click')
    await flushPromises()
    expect(wrapper.findAll('.client-card')).toHaveLength(1)
    expect(wrapper.text()).toContain('offline-client')
  })

  it('shows clients.filterEmpty when selected filter has no results', async () => {
    mockStore.clients = [makeClient({status: 'online'})]
    const {wrapper} = await mountClients()
    await wrapper.findAll('.filter-chip')[2].trigger('click')
    await flushPromises()
    expect(wrapper.text()).toContain('clients.filterEmpty')
  })

  it('resets page to one when status filter changes', async () => {
    mockStore.clients = Array.from({length: 12}, (_, i) => makeClient({sessionId: `c${i}`, status: 'online'}))
    const {wrapper} = await mountClients()
    const vm = wrapper.vm as unknown as {page: number}
    vm.page = 2
    await flushPromises()
    await wrapper.findAll('.filter-chip')[2].trigger('click')
    await flushPromises()
    expect(vm.page).toBe(1)
  })
})

describe('Clients – client cards', () => {
  it('renders session ID and optional metadata', async () => {
    mockStore.clients = [makeClient()]
    const {wrapper} = await mountClients()
    expect(wrapper.text()).toContain('client-alpha')
    expect(wrapper.text()).toContain('host-a')
    expect(wrapper.text()).toContain('alice')
    expect(wrapper.text()).toContain('v1.2.3')
  })

  it('renders IP address and tunnel count', async () => {
    mockStore.clients = [makeClient({clientIP: '192.168.1.10', tunnelCount: 7})]
    const {wrapper} = await mountClients()
    expect(wrapper.text()).toContain('192.168.1.10')
    expect(wrapper.text()).toContain('7')
  })

  it('uses em dash when client IP is unavailable', async () => {
    mockStore.clients = [makeClient({clientIP: ''})]
    const {wrapper} = await mountClients()
    expect(wrapper.text()).toContain('—')
  })

  it('applies offline class to offline client cards', async () => {
    mockStore.clients = [makeClient({status: 'offline'})]
    const {wrapper} = await mountClients()
    expect(wrapper.find('.client-card').classes()).toContain('offline')
  })

  it('shows kick button only for online clients', async () => {
    mockStore.clients = [
      makeClient({sessionId: 'online', status: 'online'}),
      makeClient({sessionId: 'offline', status: 'offline'}),
    ]
    const {wrapper} = await mountClients()
    expect(wrapper.findAll('.kick-btn')).toHaveLength(1)
  })

  it('navigates to client-detail when a card is clicked', async () => {
    mockStore.clients = [makeClient()]
    const {wrapper, router} = await mountClients()
    await wrapper.find('.client-card').trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.name).toBe('client-detail')
    expect(router.currentRoute.value.params.sessionId).toBe('client-alpha')
  })

  it('navigates to client-detail on Enter key', async () => {
    mockStore.clients = [makeClient()]
    const {wrapper, router} = await mountClients()
    await wrapper.find('.client-card').trigger('keydown', {key: 'Enter'})
    await flushPromises()
    expect(router.currentRoute.value.name).toBe('client-detail')
  })

  it('navigates to client-detail on Space key', async () => {
    mockStore.clients = [makeClient()]
    const {wrapper, router} = await mountClients()
    await wrapper.find('.client-card').trigger('keydown', {key: ' '})
    await flushPromises()
    expect(router.currentRoute.value.name).toBe('client-detail')
  })
})

describe('Clients – pagination', () => {
  it('shows only ten cards on the first page by default', async () => {
    mockStore.clients = Array.from({length: 15}, (_, i) => makeClient({sessionId: `client-${i}`}))
    const {wrapper} = await mountClients()
    expect(wrapper.findAll('.client-card')).toHaveLength(10)
  })

  it('clamps page to one after filtering down to a single page', async () => {
    mockStore.clients = Array.from({length: 12}, (_, i) => makeClient({sessionId: `client-${i}`, status: i < 2 ? 'offline' : 'online'}))
    const {wrapper} = await mountClients()
    const vm = wrapper.vm as unknown as {page: number}
    vm.page = 2
    await flushPromises()
    await wrapper.findAll('.filter-chip')[2].trigger('click')
    await flushPromises()
    expect(vm.page).toBe(1)
  })
})

describe('Clients – kick', () => {
  it('calls kickClient with session ID when kick button is clicked', async () => {
    mockStore.clients = [makeClient()]
    const {wrapper} = await mountClients()
    await wrapper.find('.kick-btn').trigger('click')
    await flushPromises()
    expect(api.kickClient).toHaveBeenCalledWith('client-alpha')
  })

  it('refreshes dashboard data after successful kick', async () => {
    mockStore.clients = [makeClient()]
    const {wrapper} = await mountClients()
    await wrapper.find('.kick-btn').trigger('click')
    await flushPromises()
    expect(mockStore.refresh).toHaveBeenCalledOnce()
  })

  it('shows success toast after successful kick', async () => {
    mockStore.clients = [makeClient()]
    const {wrapper} = await mountClients()
    await wrapper.find('.kick-btn').trigger('click')
    await flushPromises()
    expect(mockShowToast).toHaveBeenCalledWith('info', 'clients.kickSuccess')
  })

  it('shows failure toast when kickClient rejects', async () => {
    vi.mocked(api.kickClient).mockRejectedValue(new Error('network error'))
    mockStore.clients = [makeClient()]
    const {wrapper} = await mountClients()
    await wrapper.find('.kick-btn').trigger('click')
    await flushPromises()
    expect(mockShowToast).toHaveBeenCalledWith('error', expect.stringContaining('clients.kickFailed'))
  })

  it('does not navigate to detail when kick button is clicked', async () => {
    mockStore.clients = [makeClient()]
    const {wrapper, router} = await mountClients()
    await wrapper.find('.kick-btn').trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.path).toBe('/clients')
  })
})

describe('Clients – search box', () => {
  it('renders a search input', async () => {
    const {wrapper} = await mountClients()
    expect(wrapper.find('.search-input').exists()).toBe(true)
  })

  it('filters clients by hostname when typing', async () => {
    mockStore.clients = [
      makeClient({sessionId: 'a', hostname: 'web-server'}),
      makeClient({sessionId: 'b', hostname: 'db-server'}),
    ]
    const {wrapper} = await mountClients()
    await wrapper.find('.search-input').setValue('web')
    await flushPromises()
    expect(wrapper.findAll('.client-card')).toHaveLength(1)
    expect(wrapper.text()).toContain('web-server')
    expect(wrapper.text()).not.toContain('db-server')
  })

  it('filters clients by sessionId when typing', async () => {
    mockStore.clients = [
      makeClient({sessionId: 'session-abc', hostname: 'host-x'}),
      makeClient({sessionId: 'session-xyz', hostname: 'host-y'}),
    ]
    const {wrapper} = await mountClients()
    await wrapper.find('.search-input').setValue('abc')
    await flushPromises()
    expect(wrapper.findAll('.client-card')).toHaveLength(1)
    expect(wrapper.text()).toContain('session-abc')
  })

  it('filters clients by clientIP when typing', async () => {
    mockStore.clients = [
      makeClient({sessionId: 'a', clientIP: '10.0.0.1'}),
      makeClient({sessionId: 'b', clientIP: '192.168.1.5'}),
    ]
    const {wrapper} = await mountClients()
    await wrapper.find('.search-input').setValue('192')
    await flushPromises()
    expect(wrapper.findAll('.client-card')).toHaveLength(1)
    expect(wrapper.text()).toContain('192.168.1.5')
  })

  it('shows clients.searchEmpty when no clients match the query', async () => {
    mockStore.clients = [makeClient({hostname: 'production-host'})]
    const {wrapper} = await mountClients()
    await wrapper.find('.search-input').setValue('zzz-no-match')
    await flushPromises()
    expect(wrapper.findAll('.client-card')).toHaveLength(0)
    expect(wrapper.text()).toContain('clients.searchEmpty')
  })

  it('restores all cards after clearing the search input', async () => {
    mockStore.clients = [
      makeClient({sessionId: 'a', hostname: 'alpha'}),
      makeClient({sessionId: 'b', hostname: 'beta'}),
    ]
    const {wrapper} = await mountClients()
    await wrapper.find('.search-input').setValue('alpha')
    await flushPromises()
    expect(wrapper.findAll('.client-card')).toHaveLength(1)
    await wrapper.find('.search-clear').trigger('click')
    await flushPromises()
    expect(wrapper.findAll('.client-card')).toHaveLength(2)
  })

  it('resets page to 1 when search query changes', async () => {
    mockStore.clients = Array.from({length: 12}, (_, i) =>
      makeClient({sessionId: `client-${i}`, hostname: `host-${i}`}),
    )
    const {wrapper} = await mountClients()
    const vm = wrapper.vm as unknown as {page: number}
    vm.page = 2
    await flushPromises()
    await wrapper.find('.search-input').setValue('host')
    await flushPromises()
    expect(vm.page).toBe(1)
  })

  it('applies search and status filter simultaneously', async () => {
    mockStore.clients = [
      makeClient({sessionId: 'web-online',  hostname: 'web', status: 'online'}),
      makeClient({sessionId: 'web-offline', hostname: 'web', status: 'offline'}),
      makeClient({sessionId: 'db-online',   hostname: 'db',  status: 'online'}),
    ]
    const {wrapper} = await mountClients()
    await wrapper.findAll('.filter-chip')[1].trigger('click')
    await flushPromises()
    await wrapper.find('.search-input').setValue('web')
    await flushPromises()
    expect(wrapper.findAll('.client-card')).toHaveLength(1)
    expect(wrapper.text()).toContain('web-online')
    expect(wrapper.text()).not.toContain('web-offline')
    expect(wrapper.text()).not.toContain('db-online')
  })
})

describe('Clients – sort', () => {
  it('renders sort chips', async () => {
    const {wrapper} = await mountClients()
    expect(wrapper.find('.sort-wrap').exists()).toBe(true)
    expect(wrapper.findAll('.sort-chip').length).toBeGreaterThanOrEqual(2)
  })

  it('sorts clients by tunnelCount descending', async () => {
    mockStore.clients = [
      makeClient({sessionId: 'low',  tunnelCount: 1}),
      makeClient({sessionId: 'high', tunnelCount: 9}),
    ]
    const {wrapper} = await mountClients()
    const tunnelChip = wrapper.findAll('.sort-chip').find(w => w.text().includes('clients.sortTunnels'))
    await tunnelChip!.trigger('click')
    await flushPromises()
    const cards = wrapper.findAll('.client-card')
    expect(cards[0].text()).toContain('high')
    expect(cards[1].text()).toContain('low')
  })

  it('toggles sort direction on second click', async () => {
    mockStore.clients = [
      makeClient({sessionId: 'low',  tunnelCount: 1}),
      makeClient({sessionId: 'high', tunnelCount: 9}),
    ]
    const {wrapper} = await mountClients()
    const tunnelChip = wrapper.findAll('.sort-chip').find(w => w.text().includes('clients.sortTunnels'))!
    await tunnelChip.trigger('click')
    await flushPromises()
    await tunnelChip.trigger('click')
    await flushPromises()
    const cards = wrapper.findAll('.client-card')
    expect(cards[0].text()).toContain('low')
    expect(cards[1].text()).toContain('high')
  })
})
