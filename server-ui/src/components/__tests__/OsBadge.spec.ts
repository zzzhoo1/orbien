import {describe, expect, it, vi} from 'vitest'
import {mount} from '@vue/test-utils'
import OsBadge from '../OsBadge.vue'

vi.mock('@/composables/useLocale', () => ({
  useLocale: () => ({
    t: (key: string) => key,
    current: {value: 'en-US'},
    options: [],
    switchLocale: vi.fn(),
  }),
}))

vi.mock('@/utils/os', () => ({
  normalizeOsFamily: (os: string | null | undefined) => {
    const s = (os ?? '').toLowerCase()
    if (s.includes('windows')) return 'windows'
    if (s.includes('mac') || s.includes('darwin')) return 'macos'
    if (s.includes('linux')) return 'linux'
    if (s.includes('android')) return 'android'
    if (s.includes('freebsd')) return 'freebsd'
    return 'other'
  },
  formatArch: (arch: string | null | undefined) => {
    if (!arch) return ''
    return arch.toLowerCase().includes('arm') ? 'ARM64' : 'x86_64'
  },
}))

vi.mock('@/assets/icon/windows.svg', () => ({default: 'windows.svg'}))
vi.mock('@/assets/icon/macos.svg', () => ({default: 'macos.svg'}))
vi.mock('@/assets/icon/linux.svg', () => ({default: 'linux.svg'}))
vi.mock('@/assets/icon/android.svg', () => ({default: 'android.svg'}))
vi.mock('@/assets/icon/freebsd.svg', () => ({default: 'freebsd.svg'}))
vi.mock('@/assets/icon/device.svg', () => ({default: 'device.svg'}))

describe('OsBadge', () => {
  it('renders correct OS family class for known OS strings', () => {
    const cases = [
      {os: 'Windows 11', family: 'windows'},
      {os: 'darwin', family: 'macos'},
      {os: 'linux', family: 'linux'},
      {os: 'android', family: 'android'},
      {os: 'freebsd', family: 'freebsd'},
      {os: 'unknown', family: 'other'},
    ]
    for (const {os, family} of cases) {
      const w = mount(OsBadge, {props: {os}})
      expect(w.find('.os-badge').classes()).toContain(family)
    }
  })

  it('renders img tag for non-other OS families', () => {
    const w = mount(OsBadge, {props: {os: 'linux'}})
    expect(w.find('img.os-icon').exists()).toBe(true)
    expect(w.find('.os-icon-mask').exists()).toBe(false)
  })

  it('renders mask span (not img) for other/unknown OS', () => {
    const w = mount(OsBadge, {props: {os: 'unknown'}})
    expect(w.find('.os-icon-mask').exists()).toBe(true)
    expect(w.find('img.os-icon').exists()).toBe(false)
  })

  it('shows label text by default', () => {
    const w = mount(OsBadge, {props: {os: 'linux'}})
    expect(w.find('.os-label').exists()).toBe(true)
    expect(w.find('.os-label').text()).toContain('clients.osFamily.linux')
  })

  it('appends arch label when showArch=true and arch is set', () => {
    const w = mount(OsBadge, {props: {os: 'linux', arch: 'arm64', showArch: true}})
    expect(w.find('.os-label').text()).toContain('ARM64')
  })

  it('omits arch label when showArch=false', () => {
    const w = mount(OsBadge, {props: {os: 'linux', arch: 'arm64', showArch: false}})
    expect(w.find('.os-label').text()).not.toContain('ARM64')
  })

  it('hides icon when textOnly=true', () => {
    const w = mount(OsBadge, {props: {os: 'linux', textOnly: true}})
    expect(w.find('img.os-icon').exists()).toBe(false)
    expect(w.find('.os-icon-mask').exists()).toBe(false)
    expect(w.find('.os-label').exists()).toBe(true)
  })

  it('hides label when iconOnly=true', () => {
    const w = mount(OsBadge, {props: {os: 'linux', iconOnly: true}})
    expect(w.find('.os-label').exists()).toBe(false)
    expect(w.find('img.os-icon').exists()).toBe(true)
  })

  it('title and aria-label fall back to OS label when os and arch are empty', () => {
    const w = mount(OsBadge, {props: {os: '', arch: ''}})
    const badge = w.find('.os-badge')
    expect(badge.attributes('title')).toBe('clients.osFamily.other')
    expect(badge.attributes('aria-label')).toBe('clients.osFamily.other')
  })

  it('title includes raw os and arch when both are set', () => {
    const w = mount(OsBadge, {props: {os: 'linux', arch: 'arm64'}})
    const title = w.find('.os-badge').attributes('title') ?? ''
    expect(title).toContain('linux')
    expect(title).toContain('arm64')
  })
})
