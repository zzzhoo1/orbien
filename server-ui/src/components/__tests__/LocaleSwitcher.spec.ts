import {describe, expect, it, vi, beforeEach} from 'vitest'
import {mount} from '@vue/test-utils'
import LocaleSwitcher from '../LocaleSwitcher.vue'

const mockSwitchLocale = vi.fn()

vi.mock('@/composables/useLocale', () => ({
  useLocale: () => ({
    t: (key: string) => key,
    current: 'en-US',
    options: [
      {code: 'en-US', nativeLabel: 'English'},
      {code: 'zh-CN', nativeLabel: '中文'},
    ],
    switchLocale: mockSwitchLocale,
  }),
}))

beforeEach(() => {
  vi.clearAllMocks()
})

describe('LocaleSwitcher', () => {
  it('renders an option for each locale', () => {
    const w = mount(LocaleSwitcher)
    const opts = w.findAll('option')
    expect(opts.length).toBe(2)
    expect(opts[0].attributes('value')).toBe('en-US')
    expect(opts[1].attributes('value')).toBe('zh-CN')
  })

  it('displays nativeLabel as option text', () => {
    const w = mount(LocaleSwitcher)
    expect(w.findAll('option')[0].text()).toBe('English')
    expect(w.findAll('option')[1].text()).toBe('中文')
  })

  it('sets select value to current locale', () => {
    const w = mount(LocaleSwitcher)
    expect(w.find('select').element.value).toBe('en-US')
  })

  it('calls switchLocale with new value on change', async () => {
    const w = mount(LocaleSwitcher)
    const select = w.find('select')
    await select.setValue('zh-CN')
    expect(mockSwitchLocale).toHaveBeenCalledWith('zh-CN')
  })

  it('has aria-label from t("actions.locale")', () => {
    const w = mount(LocaleSwitcher)
    expect(w.find('select').attributes('aria-label')).toBe('actions.locale')
  })

  it('has sr-only label with same text', () => {
    const w = mount(LocaleSwitcher)
    expect(w.find('.sr-only').text()).toBe('actions.locale')
  })

  it('root element is a label', () => {
    const w = mount(LocaleSwitcher)
    expect(w.find('label').exists()).toBe(true)
  })

  it('root label has locale-switch class', () => {
    const w = mount(LocaleSwitcher)
    expect(w.find('label').classes()).toContain('locale-switch')
  })

  it('select has locale-select class', () => {
    const w = mount(LocaleSwitcher)
    expect(w.find('select').classes()).toContain('locale-select')
  })

  it('option keys match locale codes', () => {
    const w = mount(LocaleSwitcher)
    const opts = w.findAll('option')
    expect(opts[0].attributes('value')).toBe('en-US')
    expect(opts[1].attributes('value')).toBe('zh-CN')
  })

  it('calls switchLocale with en-US when selecting first option', async () => {
    const w = mount(LocaleSwitcher)
    await w.find('select').setValue('en-US')
    expect(mockSwitchLocale).toHaveBeenCalledWith('en-US')
  })

  it('sr-only and aria-label carry same translation key', () => {
    const w = mount(LocaleSwitcher)
    const ariaLabel = w.find('select').attributes('aria-label')
    const srOnly = w.find('.sr-only').text()
    expect(ariaLabel).toBe(srOnly)
  })

  it('does not render extra option elements beyond options list', () => {
    const w = mount(LocaleSwitcher)
    expect(w.findAll('option').length).toBe(2)
  })

  it('calls switchLocale with the correct value on each change', async () => {
    const w = mount(LocaleSwitcher)
    await w.find('select').setValue('zh-CN')
    await w.find('select').setValue('en-US')
    expect(mockSwitchLocale).toHaveBeenNthCalledWith(1, 'zh-CN')
    expect(mockSwitchLocale).toHaveBeenNthCalledWith(2, 'en-US')
  })
})
