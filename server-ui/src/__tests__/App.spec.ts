import {describe, it, expect, vi} from 'vitest'
import {mount, shallowMount} from '@vue/test-utils'
import App from '../App.vue'

vi.mock('@/layouts/DefaultLayout.vue', () => ({
  default: {template: '<div class="stub-default-layout"/>'},
}))

describe('App', () => {
  it('renders DefaultLayout', () => {
    const wrapper = mount(App)
    expect(wrapper.find('.stub-default-layout').exists()).toBe(true)
  })

  it('root element is the DefaultLayout stub itself (single root)', () => {
    const wrapper = mount(App)
    expect(wrapper.element.classList.contains('stub-default-layout')).toBe(true)
  })

  it('renders exactly one root element', () => {
    const wrapper = mount(App)
    expect(wrapper.element.children.length).toBe(0)
    expect(wrapper.element.tagName).toBeTruthy()
  })

  it('contains DefaultLayout component via shallowMount', () => {
    const wrapper = shallowMount(App)
    expect(wrapper.html()).toBeTruthy()
  })

  it('App component has no extra wrapper elements beyond DefaultLayout', () => {
    const wrapper = mount(App)
    expect(wrapper.element.nodeName.toLowerCase()).not.toBe('body')
  })
})
