import {describe, expect, it, vi} from 'vitest'
import {mount} from '@vue/test-utils'
import PaginationBar from '../PaginationBar.vue'

vi.mock('@/composables/useLocale', () => ({
  useLocale: () => ({
    t: (key: string, params?: Record<string, unknown>) =>
      params ? `${key}:${JSON.stringify(params)}` : key,
    current: {value: 'en-US'},
    options: [],
    switchLocale: vi.fn(),
  }),
}))

function mountBar(props: {total: number, page: number, pageSize: number, pageSizes?: number[]}) {
  return mount(PaginationBar, {props})
}

function navButtons(wrapper: ReturnType<typeof mountBar>) {
  return wrapper.findAll('button.nav-btn')
}

function pageButtons(wrapper: ReturnType<typeof mountBar>) {
  return wrapper.findAll('button.page-btn')
}

describe('PaginationBar', () => {
  it('emits update:page with 2 when clicking next on page 1 of 3 (total=23, pageSize=10)', () => {
    const wrapper = mountBar({total: 23, page: 1, pageSize: 10})
    const nextBtn = navButtons(wrapper).at(1)
    expect(nextBtn).toBeDefined()
    expect(nextBtn!.attributes('disabled')).toBeUndefined()
    nextBtn!.trigger('click')
    expect(wrapper.emitted('update:page')).toEqual([[2]])
    expect(wrapper.emitted('update:pageSize')).toBeUndefined()
  })

  it('does not render the pagination bar when total = 0', () => {
    const wrapper = mountBar({total: 0, page: 1, pageSize: 10})
    expect(wrapper.find('.pagination-bar').exists()).toBe(false)
  })

  it('disables both prev and next on a single page', () => {
    const wrapper = mountBar({total: 10, page: 1, pageSize: 10})
    const [prev, next] = navButtons(wrapper)
    expect(prev.attributes('disabled')).toBeDefined()
    expect(next.attributes('disabled')).toBeDefined()
  })

  it('disables prev on the first page and next on the last page', () => {
    const first = mountBar({total: 23, page: 1, pageSize: 10})
    expect(navButtons(first).at(0)!.attributes('disabled')).toBeDefined()
    expect(navButtons(first).at(1)!.attributes('disabled')).toBeUndefined()

    const last = mountBar({total: 23, page: 3, pageSize: 10})
    expect(navButtons(last).at(0)!.attributes('disabled')).toBeUndefined()
    expect(navButtons(last).at(1)!.attributes('disabled')).toBeDefined()
  })

  it('emits update:page with the clicked page number', () => {
    const wrapper = mountBar({total: 50, page: 1, pageSize: 10})
    const btn = pageButtons(wrapper).find((b) => b.text() === '3')
    expect(btn).toBeDefined()
    btn!.trigger('click')
    expect(wrapper.emitted('update:page')).toEqual([[3]])
  })

  it('emits update:page with the previous page when clicking prev', () => {
    const wrapper = mountBar({total: 50, page: 3, pageSize: 10})
    navButtons(wrapper).at(0)!.trigger('click')
    expect(wrapper.emitted('update:page')).toEqual([[2]])
  })

  it('emits update:page with the next page when clicking next', () => {
    const wrapper = mountBar({total: 50, page: 3, pageSize: 10})
    navButtons(wrapper).at(1)!.trigger('click')
    expect(wrapper.emitted('update:page')).toEqual([[4]])
  })

  it('clamps page = -1 to 1 for active state and button disabled state', () => {
    const wrapper = mountBar({total: 23, page: -1, pageSize: 10})
    const active = pageButtons(wrapper).find((b) => b.classes().includes('active'))
    expect(active).toBeDefined()
    expect(active!.text()).toBe('1')
    // 夹紧后 current=1: 上一页禁用, 下一页可用
    expect(navButtons(wrapper).at(0)!.attributes('disabled')).toBeDefined()
    expect(navButtons(wrapper).at(1)!.attributes('disabled')).toBeUndefined()
  })

  it('clamps page = 999 to the last page for active state and button disabled state', () => {
    const wrapper = mountBar({total: 23, page: 999, pageSize: 10})
    const active = pageButtons(wrapper).find((b) => b.classes().includes('active'))
    expect(active).toBeDefined()
    expect(active!.text()).toBe('3')
    // 夹紧后 current=3: 上一页可用, 下一页禁用
    expect(navButtons(wrapper).at(0)!.attributes('disabled')).toBeUndefined()
    expect(navButtons(wrapper).at(1)!.attributes('disabled')).toBeDefined()
  })

  it('emits update:pageSize then update:page(1) when changing page size', () => {
    const wrapper = mountBar({total: 23, page: 2, pageSize: 10})
    const select = wrapper.find('select')
    select.setValue('20')
    const emitted = wrapper.emitted()
    expect(emitted['update:pageSize']).toEqual([[20]])
    expect(emitted['update:page']).toEqual([[1]])
  })

  it('does not crash when pageSize = 0 and computes page count with the guard', () => {
    const wrapper = mountBar({total: 23, page: 1, pageSize: 0})
    // Math.max(pageSize, 1) => pageCount = ceil(23/1) = 23, 但页码窗口只显示 5 个
    const btns = pageButtons(wrapper)
    expect(btns.length).toBe(5)
    expect(btns.at(0)!.text()).toBe('1')
    // 下一页可用 (current=1 < 23)
    expect(navButtons(wrapper).at(1)!.attributes('disabled')).toBeUndefined()
  })

  it('shows only 5 consecutive page numbers when total pages > 5', () => {
    const wrapper = mountBar({total: 100, page: 1, pageSize: 10})
    const btns = pageButtons(wrapper)
    expect(btns.length).toBe(5)
    expect(btns.map((b) => b.text())).toEqual(['1', '2', '3', '4', '5'])

    const middle = mountBar({total: 1000, page: 50, pageSize: 10})
    expect(pageButtons(middle).map((b) => b.text())).toEqual(['48', '49', '50', '51', '52'])

    const end = mountBar({total: 1000, page: 100, pageSize: 10})
    expect(pageButtons(end).map((b) => b.text())).toEqual(['96', '97', '98', '99', '100'])
  })
})
