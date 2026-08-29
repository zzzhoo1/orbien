import {describe, expect, it} from 'vitest'
import {
  formatFileSize,
  isUnsetPort,
  isUnsetText,
  formatPort,
  formatText,
  isHttpTunnelType,
  formatTunnelEndpoint,
} from '../format'

describe('formatFileSize', () => {
  it('returns "0 B" for 0', () => expect(formatFileSize(0)).toBe('0 B'))
  it('returns "0 B" for null', () => expect(formatFileSize(null)).toBe('0 B'))
  it('returns "0 B" for undefined', () => expect(formatFileSize(undefined)).toBe('0 B'))
  it('returns "0 B" for negative', () => expect(formatFileSize(-1)).toBe('0 B'))
  it('returns "0 B" for NaN', () => expect(formatFileSize(NaN)).toBe('0 B'))
  it('returns "0 B" for Infinity', () => expect(formatFileSize(Infinity)).toBe('0 B'))
  it('formats 1 byte', () => expect(formatFileSize(1)).toBe('1 B'))
  it('formats bytes exactly', () => expect(formatFileSize(512)).toBe('512 B'))
  it('formats 1023 B (just under KB)', () => expect(formatFileSize(1023)).toBe('1023 B'))
  it('formats KB with 2 decimals for < 10', () => expect(formatFileSize(1024)).toBe('1.00 KB'))
  it('formats KB with 1 decimal for 10-99', () => expect(formatFileSize(10 * 1024)).toBe('10.0 KB'))
  it('formats KB with 0 decimals for >= 100', () => expect(formatFileSize(100 * 1024)).toBe('100 KB'))
  it('formats MB', () => expect(formatFileSize(1024 * 1024)).toBe('1.00 MB'))
  it('formats GB', () => expect(formatFileSize(1024 ** 3)).toBe('1.00 GB'))
  it('formats TB', () => expect(formatFileSize(1024 ** 4)).toBe('1.00 TB'))
  it('formats large TB value', () => {
    const val = 5 * 1024 ** 4
    expect(formatFileSize(val)).toContain('TB')
  })
})

describe('isUnsetPort', () => {
  it('true for null', () => expect(isUnsetPort(null)).toBe(true))
  it('true for undefined', () => expect(isUnsetPort(undefined)).toBe(true))
  it('true for 0', () => expect(isUnsetPort(0)).toBe(true))
  it('false for positive number', () => expect(isUnsetPort(8080)).toBe(false))
  it('false for port 1', () => expect(isUnsetPort(1)).toBe(false))
  it('false for port 65535', () => expect(isUnsetPort(65535)).toBe(false))
})

describe('isUnsetText', () => {
  it('true for null', () => expect(isUnsetText(null)).toBe(true))
  it('true for undefined', () => expect(isUnsetText(undefined)).toBe(true))
  it('true for empty string', () => expect(isUnsetText('')).toBe(true))
  it('true for whitespace only', () => expect(isUnsetText('   ')).toBe(true))
  it('true for tab only', () => expect(isUnsetText('\t')).toBe(true))
  it('false for non-empty', () => expect(isUnsetText('hello')).toBe(false))
  it('false for padded non-empty', () => expect(isUnsetText('  a  ')).toBe(false))
})

describe('formatPort', () => {
  it('returns null for 0', () => expect(formatPort(0)).toBeNull())
  it('returns null for null', () => expect(formatPort(null)).toBeNull())
  it('returns null for undefined', () => expect(formatPort(undefined)).toBeNull())
  it('returns string for valid port', () => expect(formatPort(3000)).toBe('3000'))
  it('returns string for port 80', () => expect(formatPort(80)).toBe('80'))
  it('returns string for port 65535', () => expect(formatPort(65535)).toBe('65535'))
})

describe('formatText', () => {
  it('returns null for empty', () => expect(formatText('')).toBeNull())
  it('returns null for whitespace', () => expect(formatText('  ')).toBeNull())
  it('returns null for null', () => expect(formatText(null)).toBeNull())
  it('returns null for undefined', () => expect(formatText(undefined)).toBeNull())
  it('trims and returns value', () => expect(formatText('  hello  ')).toBe('hello'))
  it('returns value unchanged when no padding', () => expect(formatText('world')).toBe('world'))
})

describe('isHttpTunnelType', () => {
  it('true for "http"', () => expect(isHttpTunnelType('http')).toBe(true))
  it('true for "https"', () => expect(isHttpTunnelType('https')).toBe(true))
  it('true for mixed case "HTTP"', () => expect(isHttpTunnelType('HTTP')).toBe(true))
  it('true for mixed case "HTTPS"', () => expect(isHttpTunnelType('HTTPS')).toBe(true))
  it('false for "tcp"', () => expect(isHttpTunnelType('tcp')).toBe(false))
  it('false for "udp"', () => expect(isHttpTunnelType('udp')).toBe(false))
  it('false for null', () => expect(isHttpTunnelType(null)).toBe(false))
  it('false for undefined', () => expect(isHttpTunnelType(undefined)).toBe(false))
  it('false for empty string', () => expect(isHttpTunnelType('')).toBe(false))
})

describe('formatTunnelEndpoint', () => {
  it('returns "\u2014" for empty remoteAddr', () => expect(formatTunnelEndpoint('tcp', '')).toBe('\u2014'))
  it('returns "\u2014" for null remoteAddr', () => expect(formatTunnelEndpoint('tcp', null)).toBe('\u2014'))
  it('returns "\u2014" for undefined remoteAddr', () => expect(formatTunnelEndpoint('tcp', undefined)).toBe('\u2014'))
  it('returns "\u2014" for whitespace-only remoteAddr', () => expect(formatTunnelEndpoint('tcp', '   ')).toBe('\u2014'))
  it('returns raw for http type', () => expect(formatTunnelEndpoint('http', 'example.com')).toBe('example.com'))
  it('returns raw for https type', () => expect(formatTunnelEndpoint('https', 'example.com')).toBe('example.com'))
  it('strips leading colon for non-http', () => expect(formatTunnelEndpoint('tcp', ':9000')).toBe('9000'))
  it('returns addr as-is for non-http without colon', () => expect(formatTunnelEndpoint('tcp', '0.0.0.0:9000')).toBe('0.0.0.0:9000'))
  it('returns "\u2014" when only colon remains after strip', () => expect(formatTunnelEndpoint('tcp', ':')).toBe('\u2014'))
  it('returns "\u2014" when type is null and remoteAddr is empty', () => expect(formatTunnelEndpoint(null, '')).toBe('\u2014'))
  it('strips leading colon when type is null (non-http fallback)', () => expect(formatTunnelEndpoint(null, ':8080')).toBe('8080'))
  it('strips leading colon when type is undefined', () => expect(formatTunnelEndpoint(undefined, ':443')).toBe('443'))
})
