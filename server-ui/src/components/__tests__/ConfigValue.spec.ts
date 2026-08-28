import {describe, expect, it, vi} from 'vitest'
import {mount} from '@vue/test-utils'
import ConfigValue from '../ConfigValue.vue'

vi.mock('@/composables/useLocale', () => ({
  useLocale: () => ({
    t: (key: string) => key,
    current: {value: 'en-US'},
    options: [],
    switchLocale: vi.fn(),
  }),
}))

vi.mock('@/utils/format', () => ({
  formatPort: (n: number) => (n > 0 ? `port:${n}` : null),
  formatText: (s: string) => (s ? `text:${s}` : null),
  isUnsetPort: (n: number) => n <= 0,
  isUnsetText: (s: string) => !s || s.trim() === '',
}))

vi.mock('@/components/EmptyText.vue', () => ({
  default: {
    name: 'EmptyText',
    props: {empty: {default: true}},
    template: `<span class="empty-stub" :data-empty="empty"><slot/></span>`,
  },
}))

function mount$(props: object) {
  return mount(ConfigValue, {props})
}

describe('ConfigValue – bool type', () => {
  it('shows enabled tag when value is true', () => {
    const w = mount$({type: 'bool', value: true})
    expect(w.find('.bool-tag.is-on').exists()).toBe(true)
    expect(w.text()).toContain('common.enabled')
  })

  it('shows disabled tag when value is false', () => {
    const w = mount$({type: 'bool', value: false})
    expect(w.find('.bool-tag.is-off').exists()).toBe(true)
    expect(w.text()).toContain('common.disabled')
  })

  it('treats falsy non-boolean (0) as disabled', () => {
    const w = mount$({type: 'bool', value: 0})
    expect(w.find('.bool-tag.is-off').exists()).toBe(true)
  })
})

describe('ConfigValue – port type', () => {
  it('renders formatted port for valid port number', () => {
    const w = mount$({type: 'port', value: 8080})
    const stub = w.find('.empty-stub')
    expect(stub.attributes('data-empty')).toBe('false')
    expect(stub.text()).toBe('port:8080')
  })

  it('renders EmptyText for port = 0 (unset)', () => {
    const w = mount$({type: 'port', value: 0})
    expect(w.find('.empty-stub').attributes('data-empty')).toBe('true')
  })

  it('converts string port value via Number()', () => {
    const w = mount$({type: 'port', value: '3000'})
    expect(w.find('.empty-stub').attributes('data-empty')).toBe('false')
    expect(w.find('.empty-stub').text()).toBe('port:3000')
  })
})

describe('ConfigValue – text type', () => {
  it('renders formatted text for non-empty string', () => {
    const w = mount$({type: 'text', value: 'hello'})
    const stub = w.find('.empty-stub')
    expect(stub.attributes('data-empty')).toBe('false')
    expect(stub.text()).toBe('text:hello')
  })

  it('renders EmptyText for empty string (unset)', () => {
    const w = mount$({type: 'text', value: ''})
    expect(w.find('.empty-stub').attributes('data-empty')).toBe('true')
  })

  it('renders EmptyText for whitespace-only string', () => {
    const w = mount$({type: 'text', value: '   '})
    expect(w.find('.empty-stub').attributes('data-empty')).toBe('true')
  })

  it('converts non-string value to string', () => {
    const w = mount$({type: 'text', value: 42})
    expect(w.find('.empty-stub').text()).toBe('text:42')
  })
})

describe('ConfigValue – raw type (default)', () => {
  it('renders the value as string when provided', () => {
    const w = mount$({value: 'raw-value'})
    const stub = w.find('.empty-stub')
    expect(stub.attributes('data-empty')).toBe('false')
    expect(stub.text()).toBe('raw-value')
  })

  it('renders EmptyText for null', () => {
    const w = mount$({value: null})
    expect(w.find('.empty-stub').attributes('data-empty')).toBe('true')
  })

  // Explicitly pass undefined rather than omitting the prop: when the prop
  // type union includes Boolean, Vue coerces a missing prop to false, so
  // mount$({}) would receive value===false instead of undefined.
  it('renders EmptyText for undefined', () => {
    const w = mount$({value: undefined})
    expect(w.find('.empty-stub').attributes('data-empty')).toBe('true')
  })

  it('renders EmptyText for empty string', () => {
    const w = mount$({value: ''})
    expect(w.find('.empty-stub').attributes('data-empty')).toBe('true')
  })
})
