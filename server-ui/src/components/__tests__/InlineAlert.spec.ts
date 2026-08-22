import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import InlineAlert from '../InlineAlert.vue'

describe('InlineAlert', () => {
  it('renders title text', () => {
    const wrapper = mount(InlineAlert, { props: { title: 'Something went wrong' } })
    expect(wrapper.find('.inline-alert__title').text()).toBe('Something went wrong')
  })

  it('renders default slot as message', () => {
    const wrapper = mount(InlineAlert, {
      props: { variant: 'info' },
      slots: { default: 'Extra details here' },
    })
    expect(wrapper.find('.inline-alert__message').text()).toBe('Extra details here')
  })

  it('applies variant class', () => {
    const wrapper = mount(InlineAlert, { props: { variant: 'error' } })
    expect(wrapper.classes()).toContain('inline-alert--error')
  })

  it('defaults to info variant', () => {
    const wrapper = mount(InlineAlert, {})
    expect(wrapper.classes()).toContain('inline-alert--info')
  })

  it('does not render close button when closable=false (default)', () => {
    const wrapper = mount(InlineAlert, { props: { title: 'T' } })
    expect(wrapper.find('.inline-alert__close').exists()).toBe(false)
  })

  it('renders close button when closable=true', () => {
    const wrapper = mount(InlineAlert, { props: { closable: true } })
    expect(wrapper.find('.inline-alert__close').exists()).toBe(true)
  })

  it('emits close event when close button is clicked', async () => {
    const wrapper = mount(InlineAlert, { props: { closable: true } })
    await wrapper.find('.inline-alert__close').trigger('click')
    expect(wrapper.emitted('close')).toHaveLength(1)
  })

  it('shows correct icon for each variant', () => {
    const cases: Array<[string, string]> = [
      ['success', '✓'],
      ['warning', '⚠'],
      ['error', '✕'],
      ['info', 'ℹ'],
    ]
    for (const [variant, icon] of cases) {
      const wrapper = mount(InlineAlert, { props: { variant: variant as 'info' } })
      expect(wrapper.find('.inline-alert__icon').text()).toBe(icon)
    }
  })

  it('has role=alert', () => {
    const wrapper = mount(InlineAlert, {})
    expect(wrapper.attributes('role')).toBe('alert')
  })
})
