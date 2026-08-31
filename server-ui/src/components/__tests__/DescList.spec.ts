import {describe, expect, it} from 'vitest'
import {mount} from '@vue/test-utils'
import DescList from '../DescList.vue'
import type {DescItem} from '../DescList.vue'

function makeItems(n: number): DescItem[] {
  return Array.from({length: n}, (_, i) => ({
    key: `k${i}`,
    label: `Label ${i}`,
  }))
}

describe('DescList', () => {
  it('renders a dt for every item label', () => {
    const items = makeItems(3)
    const w = mount(DescList, {props: {items}})
    const labels = w.findAll('dt.desc-label')
    expect(labels.length).toBe(3)
    expect(labels[0].text()).toBe('Label 0')
    expect(labels[2].text()).toBe('Label 2')
  })

  it('renders an empty dd when no slot content is provided for a key', () => {
    const w = mount(DescList, {props: {items: makeItems(1)}})
    expect(w.find('dd.desc-value').exists()).toBe(true)
  })

  it('renders named slot content into the correct dd', () => {
    const items: DescItem[] = [{key: 'status', label: 'Status'}]
    const w = mount(DescList, {
      props: {items},
      slots: {status: '<span class="slot-content">Online</span>'},
    })
    expect(w.find('dd.desc-value .slot-content').text()).toBe('Online')
  })

  it('sets --desc-cols CSS variable to columns prop value', () => {
    const w = mount(DescList, {props: {items: makeItems(2), columns: 3}})
    expect(w.find('dl.desc-list').attributes('style')).toContain('--desc-cols: 3')
  })

  it('defaults --desc-cols to 2 when columns is omitted', () => {
    const w = mount(DescList, {props: {items: makeItems(2)}})
    expect(w.find('dl.desc-list').attributes('style')).toContain('--desc-cols: 2')
  })

  it('sets title on desc-item when tip is provided', () => {
    const items: DescItem[] = [{key: 'x', label: 'X', tip: 'some tip'}]
    const w = mount(DescList, {props: {items}})
    expect(w.find('.desc-item').attributes('title')).toBe('some tip')
  })

  it('renders nothing when items is empty', () => {
    const w = mount(DescList, {props: {items: []}})
    expect(w.findAll('.desc-item').length).toBe(0)
  })

  it('does not set title attribute when tip is absent', () => {
    const items: DescItem[] = [{key: 'x', label: 'X'}]
    const w = mount(DescList, {props: {items}})
    const title = w.find('.desc-item').attributes('title')
    expect(title === undefined || title === '').toBe(true)
  })

  it('multiple named slots render into their own dd without cross-contamination', () => {
    const items: DescItem[] = [
      {key: 'alpha', label: 'Alpha'},
      {key: 'beta', label: 'Beta'},
    ]
    const w = mount(DescList, {
      props: {items},
      slots: {
        alpha: '<span class="a-val">ValueA</span>',
        beta: '<span class="b-val">ValueB</span>',
      },
    })
    const dds = w.findAll('dd.desc-value')
    expect(dds[0].find('.a-val').text()).toBe('ValueA')
    expect(dds[0].find('.b-val').exists()).toBe(false)
    expect(dds[1].find('.b-val').text()).toBe('ValueB')
    expect(dds[1].find('.a-val').exists()).toBe(false)
  })
})
