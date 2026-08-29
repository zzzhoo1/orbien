import {describe, it, expect} from 'vitest'
import {mount} from '@vue/test-utils'
import {createI18n} from 'vue-i18n'
import PaginationBar from '../PaginationBar.vue'

const i18n = createI18n({
  legacy: false,
  locale: 'en-US',
  messages: {
    'en-US': {
      common: {
        total: 'Total {n}',
        perPage: '{n}/page',
        pagination: 'Pagination',
        prevPage: 'Previous page',
        nextPage: 'Next page',
        notConfigured: '', enabled: '', disabled: '', back: '',
      },
      actions: {themeToLight: '', themeToDark: '', locale: '', github: '', collapseSidebar: '', expandSidebar: '', openMenu: '', closeMenu: '', logout: ''},
      nav: {menu: '', monitor: '', tunnels: '', clients: ''},
      login: {title: '', subtitle: '', tabPassword: '', tabFingerprint: '', username: '', usernamePh: '', password: '', passwordPh: '', submit: '', loading: '', registerFingerprint: '', registering: '', scanFingerprint: '', scanning: '', webAuthnHint: '', errorEmpty: '', errorEmptyUser: '', errorFailed: '', errorWebAuthn: '', errorRegister: ''},
      overview: {totalClients: '', onlineClients: '', tunnels: '', connections: '', emptyConfig: '', emptyProxies: ''},
      monitor: {listen: '', tunnelTypes: '', tunnelDist: '', serverConfig: '', chartTotal: '', quicPort: '', kcpPort: '', tcpMux: '', tlsForce: '', httpGwPort: '', httpsGwPort: '', rootDomain: '', maxConnPool: '', heartbeatTimeout: '', version: '', tipPortZero: '', hintClients: '', hintProxies: '', hintConns: '', hintBind: '', hintMuxOn: '', hintMuxOff: '', tokenConns: '', tokenConnsDesc: '', token: '', activeConns: '', allowedTunnels: '', allowedProtocols: '', allowedRemotePorts: '', noRestriction: '', emptyTokens: ''},
      clients: {hostname: '', ip: '', osFamily: {windows: '', macos: '', linux: '', android: '', freebsd: '', other: ''}, tunnels: '', connected: '', disconnected: '', connections: '', empty: '', filter: '', filterAll: '', filterEmpty: '', search: '', uptimeSecs: '', uptimeMins: '', uptimeHours: '', uptimeDays: '', agoSecs: '', agoMins: '', agoHours: '', agoDays: '', kick: '', kickConfirm: '', kickSuccess: '', kickFailed: '', back: '', detail: '', notFound: '', notFoundDesc: '', searchTunnels: '', tunnelsEmpty: '', tunnelsSearchEmpty: ''},
      tunnels: {port: '', domain: '', localAddr: '', client: '', empty: '', traffic: '', activeConnections: '', filter: '', filterAll: '', filterEmpty: '', back: '', lastStarted: '', openClient: '', delete: '', deleteSuccess: '', deleteFailed: ''},
      traffic: {in: '', out: '', total: '', today: '', network: '', history: '', historyAll: '', range: '', range24h: '', range7d: '', chartType: '', chartLine: '', chartBar: '', loading: '', failed: '', empty: ''},
      status: {online: '', offline: ''},
      errors: {unauthorized: '', http: '', api: '', unknown: ''},
    },
  },
})

function w(props: Record<string, unknown>) {
  return mount(PaginationBar, {props, global: {plugins: [i18n]}})
}

describe('PaginationBar', () => {
  describe('visibility', () => {
    it('renders nothing when total=0', () => {
      const wrapper = w({total: 0, page: 1, pageSize: 10})
      expect(wrapper.find('.pagination-bar').exists()).toBe(false)
    })

    it('renders when total > 0', () => {
      const wrapper = w({total: 5, page: 1, pageSize: 10})
      expect(wrapper.find('.pagination-bar').exists()).toBe(true)
    })

    it('renders when total equals pageSize exactly (one full page)', () => {
      const wrapper = w({total: 10, page: 1, pageSize: 10})
      expect(wrapper.find('.pagination-bar').exists()).toBe(true)
    })

    it('renders when total=1', () => {
      const wrapper = w({total: 1, page: 1, pageSize: 10})
      expect(wrapper.find('.pagination-bar').exists()).toBe(true)
    })
  })

  describe('total label', () => {
    it('shows total count 42', () => {
      const wrapper = w({total: 42, page: 1, pageSize: 10})
      expect(wrapper.find('.total').text()).toBe('Total 42')
    })

    it('shows total count 1', () => {
      const wrapper = w({total: 1, page: 1, pageSize: 10})
      expect(wrapper.find('.total').text()).toBe('Total 1')
    })

    it('shows total count 1000', () => {
      const wrapper = w({total: 1000, page: 1, pageSize: 10})
      expect(wrapper.find('.total').text()).toBe('Total 1000')
    })
  })

  describe('page buttons', () => {
    it('renders correct number of page buttons for small total', () => {
      const wrapper = w({total: 30, page: 1, pageSize: 10})
      const buttons = wrapper.findAll('button.page-btn')
      expect(buttons).toHaveLength(3)
    })

    it('renders 1 page button when total <= pageSize', () => {
      const wrapper = w({total: 5, page: 1, pageSize: 10})
      const buttons = wrapper.findAll('button.page-btn')
      expect(buttons).toHaveLength(1)
    })

    it('current page button has .active class', () => {
      const wrapper = w({total: 50, page: 3, pageSize: 10})
      const active = wrapper.findAll('button.page-btn').filter(b => b.classes('active'))
      expect(active).toHaveLength(1)
      expect(active[0].text()).toBe('3')
    })

    it('page 1 is active when page=1', () => {
      const wrapper = w({total: 50, page: 1, pageSize: 10})
      const active = wrapper.findAll('button.page-btn').filter(b => b.classes('active'))
      expect(active[0].text()).toBe('1')
    })

    it('emits update:page when page button clicked', async () => {
      const wrapper = w({total: 50, page: 1, pageSize: 10})
      await wrapper.findAll('button.page-btn')[1].trigger('click')
      expect(wrapper.emitted('update:page')).toBeTruthy()
      expect(wrapper.emitted('update:page')![0]).toEqual([2])
    })

    it('does not emit update:page when clicking already-active page', async () => {
      const wrapper = w({total: 50, page: 1, pageSize: 10})
      await wrapper.findAll('button.page-btn')[0].trigger('click')
      expect(wrapper.emitted('update:page')).toBeFalsy()
    })
  })

  describe('prev/next buttons', () => {
    it('prev button is disabled on first page', () => {
      const wrapper = w({total: 50, page: 1, pageSize: 10})
      const prev = wrapper.find('button[aria-label="Previous page"]')
      expect(prev.attributes('disabled')).toBeDefined()
    })

    it('next button is disabled on last page', () => {
      const wrapper = w({total: 50, page: 5, pageSize: 10})
      const next = wrapper.find('button[aria-label="Next page"]')
      expect(next.attributes('disabled')).toBeDefined()
    })

    it('both prev and next are disabled when total=1 page', () => {
      const wrapper = w({total: 5, page: 1, pageSize: 10})
      expect(wrapper.find('button[aria-label="Previous page"]').attributes('disabled')).toBeDefined()
      expect(wrapper.find('button[aria-label="Next page"]').attributes('disabled')).toBeDefined()
    })

    it('prev is enabled on page > 1', () => {
      const wrapper = w({total: 50, page: 2, pageSize: 10})
      const prev = wrapper.find('button[aria-label="Previous page"]')
      expect(prev.attributes('disabled')).toBeUndefined()
    })

    it('next is enabled when not on last page', () => {
      const wrapper = w({total: 50, page: 1, pageSize: 10})
      const next = wrapper.find('button[aria-label="Next page"]')
      expect(next.attributes('disabled')).toBeUndefined()
    })

    it('prev click emits update:page with page-1', async () => {
      const wrapper = w({total: 50, page: 3, pageSize: 10})
      await wrapper.find('button[aria-label="Previous page"]').trigger('click')
      expect(wrapper.emitted('update:page')![0]).toEqual([2])
    })

    it('next click emits update:page with page+1', async () => {
      const wrapper = w({total: 50, page: 2, pageSize: 10})
      await wrapper.find('button[aria-label="Next page"]').trigger('click')
      expect(wrapper.emitted('update:page')![0]).toEqual([3])
    })

    it('next from page 1 goes to page 2', async () => {
      const wrapper = w({total: 50, page: 1, pageSize: 10})
      await wrapper.find('button[aria-label="Next page"]').trigger('click')
      expect(wrapper.emitted('update:page')![0]).toEqual([2])
    })
  })

  describe('page size select', () => {
    it('renders page size options', () => {
      const wrapper = w({total: 100, page: 1, pageSize: 10})
      const options = wrapper.findAll('select option')
      expect(options).toHaveLength(3)
    })

    it('emits update:pageSize and update:page=1 on size change', async () => {
      const wrapper = w({total: 100, page: 3, pageSize: 10})
      const select = wrapper.find('select')
      await select.setValue('20')
      expect(wrapper.emitted('update:pageSize')).toBeTruthy()
      expect(wrapper.emitted('update:pageSize')![0]).toEqual([20])
      expect(wrapper.emitted('update:page')![0]).toEqual([1])
    })

    it('emits update:page=1 whenever pageSize changes, regardless of current page', async () => {
      const wrapper = w({total: 100, page: 5, pageSize: 10})
      await wrapper.find('select').setValue('50')
      expect(wrapper.emitted('update:page')![0]).toEqual([1])
    })
  })

  describe('window logic', () => {
    it('shows max 5 pages when total pages > 5', () => {
      const wrapper = w({total: 200, page: 5, pageSize: 10})
      const buttons = wrapper.findAll('button.page-btn')
      expect(buttons.length).toBeLessThanOrEqual(5)
    })

    it('shows all pages when total pages <= 5', () => {
      const wrapper = w({total: 30, page: 2, pageSize: 10})
      const buttons = wrapper.findAll('button.page-btn')
      expect(buttons).toHaveLength(3)
    })

    it('page window includes current page when on last page', () => {
      const wrapper = w({total: 200, page: 20, pageSize: 10})
      const buttons = wrapper.findAll('button.page-btn')
      const texts = buttons.map(b => b.text())
      expect(texts).toContain('20')
    })

    it('page window includes current page when on first page of large set', () => {
      const wrapper = w({total: 200, page: 1, pageSize: 10})
      const buttons = wrapper.findAll('button.page-btn')
      const texts = buttons.map(b => b.text())
      expect(texts).toContain('1')
    })
  })
})
