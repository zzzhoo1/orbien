import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import EmptyText from '../EmptyText.vue'

// Stub the i18n composable used inside EmptyText
vi.mock('@/composables/useLocale', () => ({
  useLocale: () => ({ t: (k: string) => k }),
}))

describe('EmptyText', () => {
  it('renders slot content when empty=false (legacy inline mode)', () => {
    const wrapper = mount(EmptyText, {
      props: { empty: false },
      slots: { default: 'hello world' },
    })
    expect(wrapper.text()).toContain('hello world')
    expect(wrapper.find('.empty-state').exists()).toBe(false)
  })

  it('renders empty-state when empty=true (default)', () => {
    const wrapper = mount(EmptyText, { props: { title: 'Nothing here' } })
    expect(wrapper.find('.empty-state').exists()).toBe(true)
    expect(wrapper.text()).toContain('Nothing here')
  })

  it('renders icon when provided', () => {
    const wrapper = mount(EmptyText, { props: { icon: '⎕', title: 'Empty' } })
    expect(wrapper.find('.empty-state__icon').exists()).toBe(true)
    expect(wrapper.find('.empty-state__icon').text()).toBe('⎕')
  })

  it('does not render icon element when icon prop is omitted', () => {
    const wrapper = mount(EmptyText, { props: { title: 'No icon' } })
    expect(wrapper.find('.empty-state__icon').exists()).toBe(false)
  })

  it('renders description slot', () => {
    const wrapper = mount(EmptyText, {
      props: { title: 'T' },
      slots: { description: 'Some description text' },
    })
    expect(wrapper.find('.empty-state__description').text()).toBe('Some description text')
  })

  it('renders action slot', () => {
    const wrapper = mount(EmptyText, {
      props: { title: 'T' },
      slots: { action: '<button>Go back</button>' },
    })
    expect(wrapper.find('.empty-state__action button').text()).toBe('Go back')
  })

  it('has role=status on empty-state container', () => {
    const wrapper = mount(EmptyText, { props: { title: 'T' } })
    expect(wrapper.find('.empty-state').attributes('role')).toBe('status')
  })
})
