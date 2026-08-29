import {describe, it, expect, vi} from 'vitest'
import {mount} from '@vue/test-utils'
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
})
