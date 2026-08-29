import {describe, it, expect} from 'vitest'
import {mount} from '@vue/test-utils'
import {defineComponent} from 'vue'
import {createI18n} from 'vue-i18n'
import {usePresence} from '../usePresence'

const messages = {
  'en-US': {
    status: {online: 'Online', offline: 'Offline'},
    clients: {
      uptimeSecs: 'Connected {n}s',
      uptimeMins: 'Connected {n}m',
      uptimeHours: 'Connected {n}h',
      uptimeDays: 'Connected {n}d',
      agoSecs: '{n}s ago',
      agoMins: '{n}m ago',
      agoHours: '{n}h ago',
      agoDays: '{n}d ago',
    },
  },
}

function mountPresence() {
  const i18n = createI18n({legacy: false, locale: 'en-US', messages})
  let result: ReturnType<typeof usePresence>
  mount(defineComponent({
    setup() {
      result = usePresence()
      return {}
    },
    template: '<div/>',
  }), {global: {plugins: [i18n]}})
  return result!
}

describe('usePresence \u2013 isOnline', () => {
  it('true for undefined', () => expect(mountPresence().isOnline(undefined)).toBe(true))
  it('true for empty string', () => expect(mountPresence().isOnline('')).toBe(true))
  it('true for "online"', () => expect(mountPresence().isOnline('online')).toBe(true))
  it('false for "offline"', () => expect(mountPresence().isOnline('offline')).toBe(false))
  it('false for arbitrary string', () => expect(mountPresence().isOnline('away')).toBe(false))
  it('false for "ONLINE" (case-sensitive)', () => expect(mountPresence().isOnline('ONLINE')).toBe(false))
})

describe('usePresence \u2013 statusLabel', () => {
  it('returns translated online label for undefined', () => {
    expect(mountPresence().statusLabel(undefined)).toBe('Online')
  })
  it('returns translated online label for "online"', () => {
    expect(mountPresence().statusLabel('online')).toBe('Online')
  })
  it('returns translated online label for empty string', () => {
    expect(mountPresence().statusLabel('')).toBe('Online')
  })
  it('returns translated offline label for "offline"', () => {
    expect(mountPresence().statusLabel('offline')).toBe('Offline')
  })
  it('returns raw string for unknown status', () => {
    expect(mountPresence().statusLabel('away')).toBe('away')
  })
})

describe('usePresence \u2013 formatSeen (online uptime)', () => {
  it('< 60s', () => expect(mountPresence().formatSeen(45, true)).toBe('Connected 45s'))
  it('exactly 0s', () => expect(mountPresence().formatSeen(0, true)).toBe('Connected 0s'))
  it('exactly 59s (still secs)', () => expect(mountPresence().formatSeen(59, true)).toBe('Connected 59s'))
  it('exactly 60s \u2192 1m', () => expect(mountPresence().formatSeen(60, true)).toBe('Connected 1m'))
  it('< 3600s (mins)', () => expect(mountPresence().formatSeen(90, true)).toBe('Connected 1m'))
  it('exactly 3600s \u2192 1h', () => expect(mountPresence().formatSeen(3600, true)).toBe('Connected 1h'))
  it('< 86400s (hours)', () => expect(mountPresence().formatSeen(7200, true)).toBe('Connected 2h'))
  it('exactly 86400s \u2192 1d', () => expect(mountPresence().formatSeen(86400, true)).toBe('Connected 1d'))
  it('>= 86400s (days)', () => expect(mountPresence().formatSeen(172800, true)).toBe('Connected 2d'))
  it('negative clamped to 0', () => expect(mountPresence().formatSeen(-10, true)).toBe('Connected 0s'))
  it('NaN treated as 0', () => expect(mountPresence().formatSeen(NaN, true)).toBe('Connected 0s'))
})

describe('usePresence \u2013 formatSeen (offline ago)', () => {
  it('< 60s ago', () => expect(mountPresence().formatSeen(30, false)).toBe('30s ago'))
  it('exactly 0s ago', () => expect(mountPresence().formatSeen(0, false)).toBe('0s ago'))
  it('exactly 59s (still secs)', () => expect(mountPresence().formatSeen(59, false)).toBe('59s ago'))
  it('exactly 60s \u2192 1m ago', () => expect(mountPresence().formatSeen(60, false)).toBe('1m ago'))
  it('< 3600s ago (mins)', () => expect(mountPresence().formatSeen(120, false)).toBe('2m ago'))
  it('exactly 3600s \u2192 1h ago', () => expect(mountPresence().formatSeen(3600, false)).toBe('1h ago'))
  it('< 86400s ago (hours)', () => expect(mountPresence().formatSeen(7200, false)).toBe('2h ago'))
  it('exactly 86400s \u2192 1d ago', () => expect(mountPresence().formatSeen(86400, false)).toBe('1d ago'))
  it('>= 86400s ago (days)', () => expect(mountPresence().formatSeen(172800, false)).toBe('2d ago'))
  it('negative clamped to 0', () => expect(mountPresence().formatSeen(-5, false)).toBe('0s ago'))
})
