import {describe, it, expect, vi, beforeEach} from 'vitest'
import {mount, flushPromises} from '@vue/test-utils'
import {createPinia, setActivePinia} from 'pinia'
import {createRouter, createMemoryHistory} from 'vue-router'
import Login from '../Login.vue'

// ── mock assets ──────────────────────────────────────────────────────────────
vi.mock('@/assets/images/logo.png', () => ({default: 'logo.png'}))

// ── mock @/api/client used by auth store ─────────────────────────────────────
vi.mock('@/api/client', () => ({
  fetchAuthStatus: vi.fn().mockResolvedValue({webauthn: false, password: true}),
}))

// ── mock useLocale ────────────────────────────────────────────────────────────
vi.mock('@/composables/useLocale', () => ({
  useLocale: () => ({t: (k: string) => k}),
}))

// ── mock useToast ─────────────────────────────────────────────────────────────
const mockShowToast = vi.fn()
vi.mock('@/composables/useToast', () => ({
  useToast: () => ({show: mockShowToast}),
}))

// ── mock useWebAuthn (reactive refs so Login's computed stays live) ────────────────
import {ref} from 'vue'
const mockRegister = vi.fn()
const mockAuthenticate = vi.fn()
const mockSupported = ref(true)
const mockRegistering = ref(false)
const mockAuthenticating = ref(false)

vi.mock('@/composables/useWebAuthn', () => ({
  useWebAuthn: () => ({
    supported: mockSupported,
    registering: mockRegistering,
    authenticating: mockAuthenticating,
    register: mockRegister,
    authenticate: mockAuthenticate,
  }),
}))

// ── mock auth store ───────────────────────────────────────────────────────────
const mockLoadCapabilities = vi.fn()
const mockLoginWithPassword = vi.fn()
const mockSetAuthenticated = vi.fn()
const mockCapabilities = {webauthn: false, password: true}

vi.mock('@/stores/auth', () => ({
  useAuthStore: () => ({
    loadCapabilities: mockLoadCapabilities,
    loginWithPassword: mockLoginWithPassword,
    setAuthenticated: mockSetAuthenticated,
    get capabilities() { return mockCapabilities },
  }),
}))

// ── helpers ───────────────────────────────────────────────────────────────────
function makeRouter() {
  return createRouter({
    history: createMemoryHistory(),
    routes: [
      {path: '/', component: {template: '<div/>'}},
      {path: '/login', component: Login},
    ],
  })
}

async function mountLogin(webauthn = false) {
  mockCapabilities.webauthn = webauthn
  mockCapabilities.password = true
  const router = makeRouter()
  await router.push('/login')
  const wrapper = mount(Login, {
    global: {
      plugins: [createPinia(), router],
      stubs: {transition: true},
    },
  })
  await flushPromises()
  return {wrapper, router}
}

beforeEach(() => {
  setActivePinia(createPinia())
  vi.clearAllMocks()
  mockCapabilities.webauthn = false
  mockCapabilities.password = true
  mockSupported.value = true
  mockRegistering.value = false
  mockAuthenticating.value = false
  mockLoadCapabilities.mockResolvedValue(undefined)
  mockLoginWithPassword.mockResolvedValue(undefined)
  mockAuthenticate.mockResolvedValue(true)
  mockRegister.mockResolvedValue(undefined)
})

// ── mount & lifecycle ─────────────────────────────────────────────────────────
describe('Login – mount', () => {
  it('renders login title', async () => {
    const {wrapper} = await mountLogin()
    expect(wrapper.text()).toContain('login.title')
  })

  it('calls loadCapabilities on mount', async () => {
    await mountLogin()
    expect(mockLoadCapabilities).toHaveBeenCalledOnce()
  })

  it('defaults to password mode', async () => {
    const {wrapper} = await mountLogin()
    expect(wrapper.find('form').exists()).toBe(true)
  })
})

// ── tabs ──────────────────────────────────────────────────────────────────────
describe('Login – tabs', () => {
  it('hides passkey tab when webauthn disabled on server', async () => {
    const {wrapper} = await mountLogin(false)
    expect(wrapper.findAll('[role="tab"]')).toHaveLength(1)
  })

  it('shows passkey tab when webauthn enabled on server and browser supports it', async () => {
    const {wrapper} = await mountLogin(true)
    expect(wrapper.findAll('[role="tab"]')).toHaveLength(2)
  })

  it('switches to webauthn mode when passkey tab clicked', async () => {
    const {wrapper} = await mountLogin(true)
    await wrapper.findAll('[role="tab"]')[1].trigger('click')
    expect(wrapper.find('form').exists()).toBe(false)
    expect(wrapper.find('.fingerprint-btn').exists()).toBe(true)
  })

  it('hides passkey tab when browser does not support WebAuthn', async () => {
    mockSupported.value = false
    const {wrapper} = await mountLogin(true)
    expect(wrapper.findAll('[role="tab"]')).toHaveLength(1)
  })
})

// ── password login ────────────────────────────────────────────────────────────
describe('Login – password login', () => {
  it('shows error when username is empty', async () => {
    const {wrapper} = await mountLogin()
    await wrapper.find('form').trigger('submit')
    await flushPromises()
    expect(wrapper.find('[role="alert"]').text()).toBe('login.errorEmpty')
  })

  it('shows error when password is empty', async () => {
    const {wrapper} = await mountLogin()
    await wrapper.find('#login-user').setValue('alice')
    await wrapper.find('form').trigger('submit')
    await flushPromises()
    expect(wrapper.find('[role="alert"]').text()).toBe('login.errorEmpty')
  })

  it('calls loginWithPassword with username and password', async () => {
    const {wrapper} = await mountLogin()
    await wrapper.find('#login-user').setValue('alice')
    await wrapper.find('#login-pass').setValue('secret')
    await wrapper.find('form').trigger('submit')
    await flushPromises()
    expect(mockLoginWithPassword).toHaveBeenCalledWith('alice', 'secret')
  })

  it('navigates to / on successful login', async () => {
    const {wrapper, router} = await mountLogin()
    await wrapper.find('#login-user').setValue('alice')
    await wrapper.find('#login-pass').setValue('secret')
    await wrapper.find('form').trigger('submit')
    await flushPromises()
    expect(router.currentRoute.value.path).toBe('/')
  })

  it('shows toast on successful login', async () => {
    const {wrapper} = await mountLogin()
    await wrapper.find('#login-user').setValue('alice')
    await wrapper.find('#login-pass').setValue('secret')
    await wrapper.find('form').trigger('submit')
    await flushPromises()
    expect(mockShowToast).toHaveBeenCalledWith('info', 'login.submit')
  })

  it('shows error message on login failure (UNAUTHORIZED)', async () => {
    mockLoginWithPassword.mockRejectedValue(new Error('UNAUTHORIZED'))
    const {wrapper} = await mountLogin()
    await wrapper.find('#login-user').setValue('alice')
    await wrapper.find('#login-pass').setValue('secret')
    await wrapper.find('form').trigger('submit')
    await flushPromises()
    expect(wrapper.find('[role="alert"]').text()).toBe('login.errorFailed')
  })

  it('shows error message on login failure (HTTP_500)', async () => {
    mockLoginWithPassword.mockRejectedValue(new Error('HTTP_500'))
    const {wrapper} = await mountLogin()
    await wrapper.find('#login-user').setValue('alice')
    await wrapper.find('#login-pass').setValue('secret')
    await wrapper.find('form').trigger('submit')
    await flushPromises()
    expect(wrapper.find('[role="alert"]').text()).toBe('login.errorFailed')
  })

  it('shows generic fallback error for unknown error', async () => {
    mockLoginWithPassword.mockRejectedValue(new Error('SOME_UNKNOWN'))
    const {wrapper} = await mountLogin()
    await wrapper.find('#login-user').setValue('alice')
    await wrapper.find('#login-pass').setValue('secret')
    await wrapper.find('form').trigger('submit')
    await flushPromises()
    expect(wrapper.find('[role="alert"]').text()).toBe('login.errorFailed')
  })
})

// ── webauthn login ────────────────────────────────────────────────────────────
describe('Login – webauthn authenticate', () => {
  async function switchToWebAuthn() {
    const {wrapper, router} = await mountLogin(true)
    await wrapper.findAll('[role="tab"]')[1].trigger('click')
    return {wrapper, router}
  }

  it('calls authenticate when fingerprint button clicked', async () => {
    const {wrapper} = await switchToWebAuthn()
    await wrapper.find('.fingerprint-btn').trigger('click')
    await flushPromises()
    expect(mockAuthenticate).toHaveBeenCalledOnce()
  })

  it('navigates to / on successful authenticate', async () => {
    const {wrapper, router} = await switchToWebAuthn()
    await wrapper.find('.fingerprint-btn').trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.path).toBe('/')
  })

  it('shows webauthn error when authenticate returns false', async () => {
    mockAuthenticate.mockResolvedValue(false)
    const {wrapper} = await switchToWebAuthn()
    await wrapper.find('.fingerprint-btn').trigger('click')
    await flushPromises()
    expect(wrapper.find('[role="alert"]').text()).toBe('login.errorWebAuthn')
  })

  it('shows webauthn error when authenticate throws WEBAUTHN_INIT_FAILED', async () => {
    mockAuthenticate.mockRejectedValue(new Error('WEBAUTHN_INIT_FAILED'))
    const {wrapper} = await switchToWebAuthn()
    await wrapper.find('.fingerprint-btn').trigger('click')
    await flushPromises()
    expect(wrapper.find('[role="alert"]').text()).toBe('login.errorWebAuthn')
  })
})

// ── register fingerprint ──────────────────────────────────────────────────────
describe('Login – register fingerprint', () => {
  it('shows errorEmptyUser when username is empty', async () => {
    const {wrapper} = await mountLogin(true)
    await wrapper.find('.btn-ghost').trigger('click')
    await flushPromises()
    expect(wrapper.find('[role="alert"]').text()).toBe('login.errorEmptyUser')
  })

  it('calls register with username', async () => {
    const {wrapper} = await mountLogin(true)
    await wrapper.find('#login-user').setValue('alice')
    await wrapper.find('.btn-ghost').trigger('click')
    await flushPromises()
    expect(mockRegister).toHaveBeenCalledWith('alice')
  })

  it('switches to webauthn mode after successful register', async () => {
    const {wrapper} = await mountLogin(true)
    await wrapper.find('#login-user').setValue('alice')
    await wrapper.find('.btn-ghost').trigger('click')
    await flushPromises()
    expect(wrapper.find('.fingerprint-btn').exists()).toBe(true)
  })

  it('shows register error when register throws REGISTER_FAILED', async () => {
    mockRegister.mockRejectedValue(new Error('REGISTER_FAILED'))
    const {wrapper} = await mountLogin(true)
    await wrapper.find('#login-user').setValue('alice')
    await wrapper.find('.btn-ghost').trigger('click')
    await flushPromises()
    expect(wrapper.find('[role="alert"]').text()).toBe('login.errorRegister')
  })
})

// ── toUserMessage error code mapping ─────────────────────────────────────────────────
describe('Login – toUserMessage error code mapping', () => {
  const cases: [string, string][] = [
    ['WEBAUTHN_NOT_SUPPORTED', 'login.errorWebAuthn'],
    ['WEBAUTHN_FAILED',        'login.errorWebAuthn'],
    ['REGISTER_INIT_FAILED',   'login.errorRegister'],
    ['REGISTER_FAILED',        'login.errorRegister'],
    ['UNEXPECTED_RESPONSE',    'login.errorFailed'],
  ]

  for (const [code, expected] of cases) {
    it(`maps ${code} → ${expected}`, async () => {
      mockLoginWithPassword.mockRejectedValue(new Error(code))
      const {wrapper} = await mountLogin()
      await wrapper.find('#login-user').setValue('alice')
      await wrapper.find('#login-pass').setValue('secret')
      await wrapper.find('form').trigger('submit')
      await flushPromises()
      expect(wrapper.find('[role="alert"]').text()).toBe(expected)
    })
  }
})
