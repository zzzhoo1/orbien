/**
 * Runtime completeness & consistency guard for locale message files.
 *
 * TypeScript's satisfies already checks structural shape at compile time,
 * but this suite catches:
 *   - Empty string values left as placeholders
 *   - Keys present in one locale but missing in the other (future drift)
 *   - Interpolation variable parity between locales
 */
import {describe, it, expect} from 'vitest'
import enUS from '../messages/en-US'
import zhCN from '../messages/zh-CN'

// ── Helpers ────────────────────────────────────────────────────────────────────

/** Recursively collect all leaf string values as { dotPath -> value } */
function flattenMessages(
  obj: Record<string, unknown>,
  prefix = '',
): Record<string, string> {
  const result: Record<string, string> = {}
  for (const [key, val] of Object.entries(obj)) {
    const path = prefix ? `${prefix}.${key}` : key
    if (typeof val === 'string') {
      result[path] = val
    } else if (val && typeof val === 'object') {
      Object.assign(result, flattenMessages(val as Record<string, unknown>, path))
    }
  }
  return result
}

/** Extract {variable} placeholders from a string */
function extractVars(s: string): Set<string> {
  return new Set([...s.matchAll(/\{(\w+)\}/g)].map(m => m[1]))
}

const en = flattenMessages(enUS as unknown as Record<string, unknown>)
const zh = flattenMessages(zhCN as unknown as Record<string, unknown>)

// ── en-US completeness ──────────────────────────────────────────────────────────────
describe('en-US messages – completeness', () => {
  it('has at least 80 leaf keys', () => {
    expect(Object.keys(en).length).toBeGreaterThanOrEqual(80)
  })

  it('has no empty string values', () => {
    const empty = Object.entries(en).filter(([, v]) => v.trim() === '')
    expect(empty, `Empty keys: ${empty.map(([k]) => k).join(', ')}`).toHaveLength(0)
  })

  it('has all top-level sections', () => {
    const sections = ['nav', 'actions', 'common', 'login', 'overview',
      'monitor', 'clients', 'tunnels', 'traffic', 'status', 'errors']
    for (const s of sections) {
      const keys = Object.keys(en).filter(k => k.startsWith(`${s}.`))
      expect(keys.length, `Section "${s}" is empty`).toBeGreaterThan(0)
    }
  })

  it('nav section has menu, monitor, tunnels, clients', () => {
    expect(en['nav.menu']).toBeTruthy()
    expect(en['nav.monitor']).toBeTruthy()
    expect(en['nav.tunnels']).toBeTruthy()
    expect(en['nav.clients']).toBeTruthy()
  })

  it('errors section has all four codes', () => {
    expect(en['errors.unauthorized']).toBeTruthy()
    expect(en['errors.http']).toBeTruthy()
    expect(en['errors.api']).toBeTruthy()
    expect(en['errors.unknown']).toBeTruthy()
  })

  it('errors.http contains {status} placeholder', () => {
    expect(en['errors.http']).toContain('{status}')
  })
})

// ── zh-CN completeness ──────────────────────────────────────────────────────────────
describe('zh-CN messages – completeness', () => {
  it('has at least 80 leaf keys', () => {
    expect(Object.keys(zh).length).toBeGreaterThanOrEqual(80)
  })

  it('has no empty string values', () => {
    const empty = Object.entries(zh).filter(([, v]) => v.trim() === '')
    expect(empty, `Empty keys: ${empty.map(([k]) => k).join(', ')}`).toHaveLength(0)
  })

  it('has all top-level sections', () => {
    const sections = ['nav', 'actions', 'common', 'login', 'overview',
      'monitor', 'clients', 'tunnels', 'traffic', 'status', 'errors']
    for (const s of sections) {
      const keys = Object.keys(zh).filter(k => k.startsWith(`${s}.`))
      expect(keys.length, `Section "${s}" is empty`).toBeGreaterThan(0)
    }
  })

  it('errors section has all four codes', () => {
    expect(zh['errors.unauthorized']).toBeTruthy()
    expect(zh['errors.http']).toBeTruthy()
    expect(zh['errors.api']).toBeTruthy()
    expect(zh['errors.unknown']).toBeTruthy()
  })

  it('errors.http contains {status} placeholder', () => {
    expect(zh['errors.http']).toContain('{status}')
  })
})

// ── Key parity ─────────────────────────────────────────────────────────────────────
describe('locale key parity', () => {
  const enKeys = new Set(Object.keys(en))
  const zhKeys = new Set(Object.keys(zh))

  it('both locales have the same number of keys', () => {
    expect(Object.keys(en).length).toBe(Object.keys(zh).length)
  })

  it('all en-US keys exist in zh-CN', () => {
    const missing = [...enKeys].filter(k => !zhKeys.has(k))
    expect(missing, `Missing in zh-CN: ${missing.join(', ')}`).toHaveLength(0)
  })

  it('all zh-CN keys exist in en-US', () => {
    const missing = [...zhKeys].filter(k => !enKeys.has(k))
    expect(missing, `Missing in en-US: ${missing.join(', ')}`).toHaveLength(0)
  })
})

// ── Interpolation variable parity ────────────────────────────────────────────────
describe('interpolation variable parity', () => {
  it('every key with variables in en-US has the same variables in zh-CN', () => {
    const mismatches: string[] = []
    for (const key of Object.keys(en)) {
      const enVars = extractVars(en[key])
      if (enVars.size === 0) continue
      const zhVars = extractVars(zh[key] ?? '')
      const missing = [...enVars].filter(v => !zhVars.has(v))
      const extra = [...zhVars].filter(v => !enVars.has(v))
      if (missing.length || extra.length) {
        mismatches.push(
          `${key}: en={${[...enVars]}} zh={${[...zhVars]}}` +
          (missing.length ? ` missing:${missing}` : '') +
          (extra.length ? ` extra:${extra}` : '')
        )
      }
    }
    expect(mismatches, mismatches.join('\n')).toHaveLength(0)
  })

  it('known interpolated keys carry correct variable names', () => {
    expect(extractVars(en['common.total'])).toContain('n')
    expect(extractVars(en['common.perPage'])).toContain('n')
    expect(extractVars(en['clients.uptimeSecs'])).toContain('n')
    expect(extractVars(en['clients.agoMins'])).toContain('n')
    expect(extractVars(en['tunnels.lastStarted'])).toContain('time')
    expect(extractVars(en['tunnels.deleteSuccess'])).toContain('name')
    expect(extractVars(en['tunnels.deleteFailed'])).toContain('name')
    expect(extractVars(en['clients.tunnelsSearchEmpty'])).toContain('q')
  })
})

// ── Spot-check specific translations ──────────────────────────────────────────────
describe('spot-check key translations', () => {
  it('status.online is "Online" in en and non-empty in zh', () => {
    expect(en['status.online']).toBe('Online')
    expect(zh['status.online'].length).toBeGreaterThan(0)
  })

  it('status.offline is "Offline" in en and non-empty in zh', () => {
    expect(en['status.offline']).toBe('Offline')
    expect(zh['status.offline'].length).toBeGreaterThan(0)
  })

  it('clients.osFamily.windows is "Windows" in both locales', () => {
    expect(en['clients.osFamily.windows']).toBe('Windows')
    expect(zh['clients.osFamily.windows']).toBe('Windows')
  })

  it('clients.osFamily.macos is "macOS" in both locales', () => {
    expect(en['clients.osFamily.macos']).toBe('macOS')
    expect(zh['clients.osFamily.macos']).toBe('macOS')
  })

  it('login.title is non-empty in both locales', () => {
    expect(en['login.title'].length).toBeGreaterThan(0)
    expect(zh['login.title'].length).toBeGreaterThan(0)
  })

  it('en and zh login.title are different strings', () => {
    expect(zh['login.title']).not.toBe(en['login.title'])
  })
})
