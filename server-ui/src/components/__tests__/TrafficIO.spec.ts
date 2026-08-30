import {describe, it, expect} from 'vitest'
import {mount} from '@vue/test-utils'
import {createI18n} from 'vue-i18n'
import TrafficIO from '../TrafficIO.vue'

const i18n = createI18n({
  legacy: false,
  locale: 'en-US',
  messages: {
    'en-US': {
      traffic: {out: 'Outbound', in: 'Inbound'},
      status: {online: 'Online', offline: 'Offline'},
      actions: {themeToLight: '', themeToDark: '', locale: '', github: '', collapseSidebar: '', expandSidebar: '', openMenu: '', closeMenu: '', logout: ''},
      nav: {menu: '', monitor: '', tunnels: '', clients: ''},
      common: {notConfigured: '', enabled: '', disabled: '', total: '', perPage: '', pagination: '', prevPage: '', nextPage: '', back: ''},
      login: {title: '', subtitle: '', tabPassword: '', tabFingerprint: '', username: '', usernamePh: '', password: '', passwordPh: '', submit: '', loading: '', registerFingerprint: '', registering: '', scanFingerprint: '', scanning: '', webAuthnHint: '', errorEmpty: '', errorEmptyUser: '', errorFailed: '', errorWebAuthn: '', errorRegister: ''},
      overview: {totalClients: '', onlineClients: '', tunnels: '', connections: '', emptyConfig: '', emptyProxies: ''},
      monitor: {listen: '', tunnelTypes: '', tunnelDist: '', serverConfig: '', chartTotal: '', quicPort: '', kcpPort: '', tcpMux: '', tlsForce: '', httpGwPort: '', httpsGwPort: '', rootDomain: '', maxConnPool: '', heartbeatTimeout: '', version: '', tipPortZero: '', hintClients: '', hintProxies: '', hintConns: '', hintBind: '', hintMuxOn: '', hintMuxOff: '', tokenConns: '', tokenConnsDesc: '', token: '', activeConns: '', allowedTunnels: '', allowedProtocols: '', allowedRemotePorts: '', noRestriction: '', emptyTokens: ''},
      clients: {hostname: '', ip: '', osFamily: {windows: '', macos: '', linux: '', android: '', freebsd: '', other: ''}, tunnels: '', connected: '', disconnected: '', connections: '', empty: '', filter: '', filterAll: '', filterEmpty: '', search: '', uptimeSecs: '', uptimeMins: '', uptimeHours: '', uptimeDays: '', agoSecs: '', agoMins: '', agoHours: '', agoDays: '', kick: '', kickConfirm: '', kickSuccess: '', kickFailed: '', back: '', detail: '', notFound: '', notFoundDesc: '', searchTunnels: '', tunnelsEmpty: '', tunnelsSearchEmpty: ''},
      tunnels: {port: '', domain: '', localAddr: '', client: '', empty: '', traffic: '', activeConnections: '', filter: '', filterAll: '', filterEmpty: '', back: '', lastStarted: '', openClient: '', delete: '', deleteSuccess: '', deleteFailed: ''},
      errors: {unauthorized: '', http: '', api: '', unknown: ''},
    },
  },
})

function w(props = {}) {
  return mount(TrafficIO, {props, global: {plugins: [i18n]}})
}

describe('TrafficIO', () => {
  describe('default rendering (stack, plain)', () => {
    it('shows formatted outbound in .out .val', () => {
      const wrapper = w({trafficOut: 1024})
      expect(wrapper.find('.row.out .val').text()).toBe('1.00 KB')
    })

    it('shows formatted inbound in .in .val', () => {
      const wrapper = w({trafficIn: 2048})
      expect(wrapper.find('.row.in .val').text()).toBe('2.00 KB')
    })

    it('defaults both values to 0 B when props omitted', () => {
      const wrapper = w()
      expect(wrapper.find('.row.out .val').text()).toBe('0 B')
      expect(wrapper.find('.row.in .val').text()).toBe('0 B')
    })

    it('handles null trafficIn/trafficOut as 0', () => {
      const wrapper = w({trafficIn: null, trafficOut: null})
      expect(wrapper.find('.row.out .val').text()).toBe('0 B')
      expect(wrapper.find('.row.in .val').text()).toBe('0 B')
    })

    it('does NOT render .sep in stack layout', () => {
      const wrapper = w({layout: 'stack'})
      expect(wrapper.find('.sep').exists()).toBe(false)
    })

    it('applies .stack class by default', () => {
      const wrapper = w()
      expect(wrapper.find('.traffic-io').classes()).toContain('stack')
    })

    it('applies .plain class by default', () => {
      const wrapper = w()
      expect(wrapper.find('.traffic-io').classes()).toContain('plain')
    })
  })

  describe('inline layout', () => {
    it('renders .sep in inline layout', () => {
      const wrapper = w({layout: 'inline'})
      expect(wrapper.find('.sep').exists()).toBe(true)
    })

    it('applies .inline class', () => {
      const wrapper = w({layout: 'inline'})
      expect(wrapper.find('.traffic-io').classes()).toContain('inline')
    })

    it('.sep text is "/"', () => {
      const wrapper = w({layout: 'inline'})
      expect(wrapper.find('.sep').text()).toBe('/')
    })

    it('.sep has aria-hidden="true"', () => {
      const wrapper = w({layout: 'inline'})
      expect(wrapper.find('.sep').attributes('aria-hidden')).toBe('true')
    })
  })

  describe('chip variant', () => {
    it('applies .chip class', () => {
      const wrapper = w({variant: 'chip'})
      expect(wrapper.find('.traffic-io').classes()).toContain('chip')
    })

    it('chip+inline: both classes present', () => {
      const wrapper = w({variant: 'chip', layout: 'inline'})
      expect(wrapper.find('.traffic-io').classes()).toContain('chip')
      expect(wrapper.find('.traffic-io').classes()).toContain('inline')
    })
  })

  describe('title attribute', () => {
    it('title contains Outbound and Inbound labels', () => {
      const wrapper = w({trafficIn: 1024, trafficOut: 512})
      const title = wrapper.find('.traffic-io').attributes('title') ?? ''
      expect(title).toContain('Outbound')
      expect(title).toContain('Inbound')
    })

    it('title contains formatted file sizes', () => {
      const wrapper = w({trafficIn: 1024, trafficOut: 512})
      const title = wrapper.find('.traffic-io').attributes('title') ?? ''
      expect(title).toContain('1.00 KB')
      expect(title).toContain('512 B')
    })

    it('title format is "Outbound: X · Inbound: Y"', () => {
      const wrapper = w({trafficIn: 0, trafficOut: 0})
      const title = wrapper.find('.traffic-io').attributes('title') ?? ''
      expect(title).toMatch(/Outbound.*Inbound/)
    })
  })

  describe('large values', () => {
    it('formats MB correctly', () => {
      const wrapper = w({trafficOut: 1024 * 1024})
      expect(wrapper.find('.row.out .val').text()).toBe('1.00 MB')
    })

    it('formats GB correctly', () => {
      const wrapper = w({trafficIn: 1024 * 1024 * 1024})
      expect(wrapper.find('.row.in .val').text()).toBe('1.00 GB')
    })

    it('formats TB correctly', () => {
      const wrapper = w({trafficOut: 1024 ** 4})
      expect(wrapper.find('.row.out .val').text()).toBe('1.00 TB')
    })
  })

  describe('arrow icons', () => {
    it('.row.out .arrow has aria-hidden="true"', () => {
      const wrapper = w()
      expect(wrapper.find('.row.out .arrow').attributes('aria-hidden')).toBe('true')
    })

    it('.row.in .arrow has aria-hidden="true"', () => {
      const wrapper = w()
      expect(wrapper.find('.row.in .arrow').attributes('aria-hidden')).toBe('true')
    })
  })
})
