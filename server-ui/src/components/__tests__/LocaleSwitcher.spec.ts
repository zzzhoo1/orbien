import {describe, expect, it, vi} from 'vitest'
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
})
