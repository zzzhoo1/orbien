import {describe, it, expect, vi, beforeEach} from 'vitest'
import {mount} from '@vue/test-utils'
import {defineComponent} from 'vue'
import {createI18n} from 'vue-i18n'
import {useLocale} from '../useLocale'
import {SUPPORTED_LOCALES, LOCALE_META} from '@/i18n'

function makeI18n(locale = 'en-US') {
  return createI18n({
    legacy: false,
    locale,
    messages: {
      'en-US': {actions: {themeToLight: '', themeToDark: '', locale: '', github: '', collapseSidebar: '', expandSidebar: '', openMenu: '', closeMenu: '', logout: ''}, nav: {menu: '', monitor: '', tunnels: '', clients: ''}, common: {notConfigured: '', enabled: '', disabled: '', total: '', perPage: '', pagination: '', prevPage: '', nextPage: '', back: ''}, login: {title: '', subtitle: '', tabPassword: '', tabFingerprint: '', username: '', usernamePh: '', password: '', passwordPh: '', submit: '', loading: '', registerFingerprint: '', registering: '', scanFingerprint: '', scanning: '', webAuthnHint: '', errorEmpty: '', errorEmptyUser: '', errorFailed: '', errorWebAuthn: '', errorRegister: ''}, overview: {totalClients: '', onlineClients: '', tunnels: '', connections: '', emptyConfig: '', emptyProxies: ''}, monitor: {listen: '', tunnelTypes: '', tunnelDist: '', serverConfig: '', chartTotal: '', quicPort: '', kcpPort: '', tcpMux: '', tlsForce: '', httpGwPort: '', httpsGwPort: '', rootDomain: '', maxConnPool: '', heartbeatTimeout: '', version: '', tipPortZero: '', hintClients: '', hintProxies: '', hintConns: '', hintBind: '', hintMuxOn: '', hintMuxOff: '', tokenConns: '', tokenConnsDesc: '', token: '', activeConns: '', allowedTunnels: '', allowedProtocols: '', allowedRemotePorts: '', noRestriction: '', emptyTokens: ''}, clients: {hostname: '', ip: '', osFamily: {windows: '', macos: '', linux: '', android: '', freebsd: '', other: ''}, tunnels: '', connected: '', disconnected: '', connections: '', empty: '', filter: '', filterAll: '', filterEmpty: '', search: '', uptimeSecs: 'Connected {n}s', uptimeMins: 'Connected {n}m', uptimeHours: 'Connected {n}h', uptimeDays: 'Connected {n}d', agoSecs: '{n}s ago', agoMins: '{n}m ago', agoHours: '{n}h ago', agoDays: '{n}d ago', kick: '', kickConfirm: '', kickSuccess: '', kickFailed: '', back: '', detail: '', notFound: '', notFoundDesc: '', searchTunnels: '', tunnelsEmpty: '', tunnelsSearchEmpty: ''}, tunnels: {port: '', domain: '', localAddr: '', client: '', empty: '', traffic: '', activeConnections: '', filter: '', filterAll: '', filterEmpty: '', back: '', lastStarted: '', openClient: '', delete: '', deleteSuccess: '', deleteFailed: ''}, traffic: {in: '', out: '', total: '', today: '', network: '', history: '', historyAll: '', range: '', range24h: '', range7d: '', chartType: '', chartLine: '', chartBar: '', loading: '', failed: '', empty: ''}, status: {online: 'Online', offline: 'Offline'}, errors: {unauthorized: '', http: '', api: '', unknown: ''}},
      'zh-CN': {actions: {themeToLight: '', themeToDark: '', locale: '', github: '', collapseSidebar: '', expandSidebar: '', openMenu: '', closeMenu: '', logout: ''}, nav: {menu: '', monitor: '', tunnels: '', clients: ''}, common: {notConfigured: '', enabled: '', disabled: '', total: '', perPage: '', pagination: '', prevPage: '', nextPage: '', back: ''}, login: {title: '', subtitle: '', tabPassword: '', tabFingerprint: '', username: '', usernamePh: '', password: '', passwordPh: '', submit: '', loading: '', registerFingerprint: '', registering: '', scanFingerprint: '', scanning: '', webAuthnHint: '', errorEmpty: '', errorEmptyUser: '', errorFailed: '', errorWebAuthn: '', errorRegister: ''}, overview: {totalClients: '', onlineClients: '', tunnels: '', connections: '', emptyConfig: '', emptyProxies: ''}, monitor: {listen: '', tunnelTypes: '', tunnelDist: '', serverConfig: '', chartTotal: '', quicPort: '', kcpPort: '', tcpMux: '', tlsForce: '', httpGwPort: '', httpsGwPort: '', rootDomain: '', maxConnPool: '', heartbeatTimeout: '', version: '', tipPortZero: '', hintClients: '', hintProxies: '', hintConns: '', hintBind: '', hintMuxOn: '', hintMuxOff: '', tokenConns: '', tokenConnsDesc: '', token: '', activeConns: '', allowedTunnels: '', allowedProtocols: '', allowedRemotePorts: '', noRestriction: '', emptyTokens: ''}, clients: {hostname: '', ip: '', osFamily: {windows: '', macos: '', linux: '', android: '', freebsd: '', other: ''}, tunnels: '', connected: '', disconnected: '', connections: '', empty: '', filter: '', filterAll: '', filterEmpty: '', search: '', uptimeSecs: '', uptimeMins: '', uptimeHours: '', uptimeDays: '', agoSecs: '', agoMins: '', agoHours: '', agoDays: '', kick: '', kickConfirm: '', kickSuccess: '', kickFailed: '', back: '', detail: '', notFound: '', notFoundDesc: '', searchTunnels: '', tunnelsEmpty: '', tunnelsSearchEmpty: ''}, tunnels: {port: '', domain: '', localAddr: '', client: '', empty: '', traffic: '', activeConnections: '', filter: '', filterAll: '', filterEmpty: '', back: '', lastStarted: '', openClient: '', delete: '', deleteSuccess: '', deleteFailed: ''}, traffic: {in: '', out: '', total: '', today: '', network: '', history: '', historyAll: '', range: '', range24h: '', range7d: '', chartType: '', chartLine: '', chartBar: '', loading: '', failed: '', empty: ''}, status: {online: '\u5728\u7EBF', offline: '\u79BB\u7EBF'}, errors: {unauthorized: '', http: '', api: '', unknown: ''}},
    },
  })
}

function mountWithLocale(locale = 'en-US') {
  const i18n = makeI18n(locale)
  let result: ReturnType<typeof useLocale>
  mount(defineComponent({
    setup() {
      result = useLocale()
      return {}
    },
    template: '<div/>',
  }), {global: {plugins: [i18n]}})
  return result!
}

describe('useLocale', () => {
  describe('current', () => {
    it('reflects initial locale en-US', () => {
      const {current} = mountWithLocale('en-US')
      expect(current.value).toBe('en-US')
    })
    it('reflects initial locale zh-CN', () => {
      const {current} = mountWithLocale('zh-CN')
      expect(current.value).toBe('zh-CN')
    })
  })

  describe('options', () => {
    it('returns one entry per supported locale', () => {
      const {options} = mountWithLocale()
      expect(options).toHaveLength(SUPPORTED_LOCALES.length)
    })
    it('each option has code, label, nativeLabel, htmlLang', () => {
      const {options} = mountWithLocale()
      for (const opt of options) {
        expect(opt.code).toBeDefined()
        expect(opt.label).toBeDefined()
        expect(opt.nativeLabel).toBeDefined()
        expect(opt.htmlLang).toBeDefined()
      }
    })
    it('option codes match SUPPORTED_LOCALES', () => {
      const {options} = mountWithLocale()
      expect(options.map(o => o.code)).toEqual([...SUPPORTED_LOCALES])
    })
    it('en-US option has correct metadata', () => {
      const {options} = mountWithLocale()
      const en = options.find(o => o.code === 'en-US')!
      expect(en.label).toBe(LOCALE_META['en-US'].label)
      expect(en.nativeLabel).toBe(LOCALE_META['en-US'].nativeLabel)
    })
  })

  describe('switchLocale', () => {
    beforeEach(() => {
      vi.resetModules()
      localStorage.clear()
    })

    it('is a function', () => {
      const {switchLocale} = mountWithLocale()
      expect(typeof switchLocale).toBe('function')
    })

    it('updates localStorage when switching to en-US', () => {
      const {switchLocale} = mountWithLocale('zh-CN')
      switchLocale('en-US')
      expect(localStorage.getItem('orbien-server-ui-locale')).toBe('en-US')
    })

    it('updates localStorage when switching to zh-CN', () => {
      const {switchLocale} = mountWithLocale('en-US')
      switchLocale('zh-CN')
      expect(localStorage.getItem('orbien-server-ui-locale')).toBe('zh-CN')
    })

    it('updates document.documentElement.lang when switching to en-US', () => {
      const {switchLocale} = mountWithLocale('zh-CN')
      switchLocale('en-US')
      expect(document.documentElement.lang).toBe('en')
    })

    it('updates document.documentElement.lang when switching to zh-CN', () => {
      const {switchLocale} = mountWithLocale('en-US')
      switchLocale('zh-CN')
      expect(document.documentElement.lang).toBe('zh-CN')
    })
  })

  describe('t (translation function)', () => {
    it('t is a function', () => {
      const {t} = mountWithLocale()
      expect(typeof t).toBe('function')
    })

    it('t returns translated string for a known key', () => {
      const {t} = mountWithLocale('en-US')
      expect(t('status.online')).toBe('Online')
    })

    it('t returns zh string when locale is zh-CN', () => {
      const {t} = mountWithLocale('zh-CN')
      expect(t('status.online')).toBe('\u5728\u7EBF')
    })
  })
})
