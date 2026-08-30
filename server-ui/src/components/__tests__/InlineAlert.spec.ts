import {describe, it, expect} from 'vitest'
import {mount} from '@vue/test-utils'
import InlineAlert from '../InlineAlert.vue'

describe('InlineAlert', () => {
  it('renders title text', () => {
    const wrapper = mount(InlineAlert, {props: {title: 'Something went wrong'}})
    expect(wrapper.find('.inline-alert__title').text()).toBe('Something went wrong')
  })

  it('renders default slot as message', () => {
    const wrapper = mount(InlineAlert, {
      props: {variant: 'info'},
      slots: {default: 'Extra details here'},
    })
    expect(wrapper.find('.inline-alert__message').text()).toBe('Extra details here')
  })

  it('applies variant class', () => {
    const wrapper = mount(InlineAlert, {props: {variant: 'error'}})
    expect(wrapper.classes()).toContain('inline-alert--error')
  })

  it('defaults to info variant', () => {
    const wrapper = mount(InlineAlert, {})
    expect(wrapper.classes()).toContain('inline-alert--info')
  })

  it('does not render close button when closable=false (default)', () => {
    const wrapper = mount(InlineAlert, {props: {title: 'T'}})
    expect(wrapper.find('.inline-alert__close').exists()).toBe(false)
  })

  it('renders close button when closable=true', () => {
    const wrapper = mount(InlineAlert, {props: {closable: true}})
    expect(wrapper.find('.inline-alert__close').exists()).toBe(true)
  })

  it('emits close event when close button is clicked', async () => {
    const wrapper = mount(InlineAlert, {props: {closable: true}})
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
      const wrapper = mount(InlineAlert, {props: {variant: variant as 'info'}})
      expect(wrapper.find('.inline-alert__icon').text()).toBe(icon)
    }
  })

  it('has role=alert', () => {
    const wrapper = mount(InlineAlert, {})
    expect(wrapper.attributes('role')).toBe('alert')
  })

  // ── additional coverage ─────────────────────────────────────────────────────

  it('does not render title element when title is omitted', () => {
    const wrapper = mount(InlineAlert, {props: {variant: 'info'}})
    expect(wrapper.find('.inline-alert__title').exists()).toBe(false)
  })

  it('does not render message element when slot is omitted', () => {
    const wrapper = mount(InlineAlert, {props: {title: 'T'}})
    expect(wrapper.find('.inline-alert__message').exists()).toBe(false)
  })

  it('renders both title and message when both are provided', () => {
    const wrapper = mount(InlineAlert, {
      props: {title: 'Head', variant: 'warning'},
      slots: {default: 'Body'},
    })
    expect(wrapper.find('.inline-alert__title').text()).toBe('Head')
    expect(wrapper.find('.inline-alert__message').text()).toBe('Body')
  })

  it('applies success variant class', () => {
    const wrapper = mount(InlineAlert, {props: {variant: 'success'}})
    expect(wrapper.classes()).toContain('inline-alert--success')
  })

  it('applies warning variant class', () => {
    const wrapper = mount(InlineAlert, {props: {variant: 'warning'}})
    expect(wrapper.classes()).toContain('inline-alert--warning')
  })

  it('close button has type=button', () => {
    const wrapper = mount(InlineAlert, {props: {closable: true}})
    expect(wrapper.find('.inline-alert__close').attributes('type')).toBe('button')
  })

  it('close button has aria-label', () => {
    const wrapper = mount(InlineAlert, {props: {closable: true}})
    expect(wrapper.find('.inline-alert__close').attributes('aria-label')).toBeTruthy()
  })

  it('icon has aria-hidden=true', () => {
    const wrapper = mount(InlineAlert, {})
    expect(wrapper.find('.inline-alert__icon').attributes('aria-hidden')).toBe('true')
  })

  it('emits close exactly once per click', async () => {
    const wrapper = mount(InlineAlert, {props: {closable: true}})
    await wrapper.find('.inline-alert__close').trigger('click')
    await wrapper.find('.inline-alert__close').trigger('click')
    expect(wrapper.emitted('close')).toHaveLength(2)
  })

  it('inline-alert base class is always present', () => {
    const wrapper = mount(InlineAlert, {props: {variant: 'error'}})
    expect(wrapper.classes()).toContain('inline-alert')
  })
})
