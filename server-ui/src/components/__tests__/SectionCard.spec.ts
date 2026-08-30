import {describe, expect, it} from 'vitest'
import {mount} from '@vue/test-utils'
import SectionCard from '../SectionCard.vue'

describe('SectionCard', () => {
  it('renders the title in h2', () => {
    const w = mount(SectionCard, {props: {title: 'My Section'}})
    expect(w.find('h2.section-title').text()).toBe('My Section')
  })

  it('renders default slot content in section-body', () => {
    const w = mount(SectionCard, {
      props: {title: 'T'},
      slots: {default: '<p class="body-content">Hello</p>'},
    })
    expect(w.find('.section-body .body-content').text()).toBe('Hello')
  })

  it('does not render section-extra when extra slot is not provided', () => {
    const w = mount(SectionCard, {props: {title: 'T'}})
    expect(w.find('.section-extra').exists()).toBe(false)
  })

  it('renders extra slot content when provided', () => {
    const w = mount(SectionCard, {
      props: {title: 'T'},
      slots: {extra: '<button class="extra-btn">Action</button>'},
    })
    expect(w.find('.section-extra .extra-btn').text()).toBe('Action')
  })

  it('has section and card classes on root element', () => {
    const w = mount(SectionCard, {props: {title: 'T'}})
    const root = w.find('section')
    expect(root.classes()).toContain('section-card')
    expect(root.classes()).toContain('card')
  })

  it('renders section-head and section-body containers', () => {
    const w = mount(SectionCard, {props: {title: 'T'}})
    expect(w.find('.section-head').exists()).toBe(true)
    expect(w.find('.section-body').exists()).toBe(true)
  })

  it('renders both default and extra slots together', () => {
    const w = mount(SectionCard, {
      props: {title: 'Both'},
      slots: {
        default: '<div class="main-slot">Main</div>',
        extra: '<span class="extra-slot">Extra</span>',
      },
    })
    expect(w.find('.section-body .main-slot').text()).toBe('Main')
    expect(w.find('.section-extra .extra-slot').text()).toBe('Extra')
  })

  it('updates title when props change', async () => {
    const w = mount(SectionCard, {props: {title: 'Old'}})
    await w.setProps({title: 'New'})
    expect(w.find('.section-title').text()).toBe('New')
  })

  it('renders empty section-body when default slot is omitted', () => {
    const w = mount(SectionCard, {props: {title: 'T'}})
    expect(w.find('.section-body').text()).toBe('')
  })

  it('root element is a section tag', () => {
    const w = mount(SectionCard, {props: {title: 'T'}})
    expect(w.find('section').exists()).toBe(true)
  })

  it('does not render extra slot content inside section-body', () => {
    const w = mount(SectionCard, {
      props: {title: 'T'},
      slots: {extra: '<button class="extra-btn">Action</button>'},
    })
    expect(w.find('.section-body .extra-btn').exists()).toBe(false)
  })
})
