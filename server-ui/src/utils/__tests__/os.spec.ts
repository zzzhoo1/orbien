import {describe, expect, it} from 'vitest'
import {normalizeOsFamily, formatArch} from '../os'

describe('normalizeOsFamily', () => {
  // windows
  it('windows from "windows"', () => expect(normalizeOsFamily('windows')).toBe('windows'))
  it('windows from "Windows 11"', () => expect(normalizeOsFamily('Windows 11')).toBe('windows'))
  it('windows from "win32"', () => expect(normalizeOsFamily('win32')).toBe('windows'))
  it('windows from "WIN32" (uppercase)', () => expect(normalizeOsFamily('WIN32')).toBe('windows'))

  // macos
  it('macos from "macos"', () => expect(normalizeOsFamily('macos')).toBe('macos'))
  it('macos from "darwin"', () => expect(normalizeOsFamily('darwin')).toBe('macos'))
  it('macos from "osx"', () => expect(normalizeOsFamily('osx')).toBe('macos'))
  it('macos from "macOS 14"', () => expect(normalizeOsFamily('macOS 14')).toBe('macos'))
  it('macos from "Darwin" (uppercase)', () => expect(normalizeOsFamily('Darwin')).toBe('macos'))

  // android
  it('android from "android"', () => expect(normalizeOsFamily('android')).toBe('android'))
  it('android from "Android 14" (uppercase)', () => expect(normalizeOsFamily('Android 14')).toBe('android'))

  // freebsd
  it('freebsd from "freebsd"', () => expect(normalizeOsFamily('freebsd')).toBe('freebsd'))
  it('freebsd from "FreeBSD 14" (uppercase)', () => expect(normalizeOsFamily('FreeBSD 14')).toBe('freebsd'))

  // linux
  it('linux from "linux"', () => expect(normalizeOsFamily('linux')).toBe('linux'))
  it('linux from "LINUX" (uppercase)', () => expect(normalizeOsFamily('LINUX')).toBe('linux'))
  it('linux from "ubuntu"', () => expect(normalizeOsFamily('ubuntu')).toBe('linux'))
  it('linux from "debian"', () => expect(normalizeOsFamily('debian')).toBe('linux'))
  it('linux from "centos"', () => expect(normalizeOsFamily('centos')).toBe('linux'))
  it('linux from "Ubuntu 22.04" (mixed case)', () => expect(normalizeOsFamily('Ubuntu 22.04')).toBe('linux'))

  // other / edge cases
  it('other for empty string', () => expect(normalizeOsFamily('')).toBe('other'))
  it('other for whitespace only', () => expect(normalizeOsFamily('   ')).toBe('other'))
  it('other for null', () => expect(normalizeOsFamily(null)).toBe('other'))
  it('other for undefined', () => expect(normalizeOsFamily(undefined)).toBe('other'))
  it('other for unknown "plan9"', () => expect(normalizeOsFamily('plan9')).toBe('other'))
  it('other for unknown "haiku"', () => expect(normalizeOsFamily('haiku')).toBe('other'))
})

describe('formatArch', () => {
  it('arm64 from "aarch64"', () => expect(formatArch('aarch64')).toBe('arm64'))
  it('arm64 from "arm64"', () => expect(formatArch('arm64')).toBe('arm64'))
  it('arm64 from "AARCH64" (uppercase)', () => expect(formatArch('AARCH64')).toBe('arm64'))
  it('arm64 from "ARM64" (uppercase)', () => expect(formatArch('ARM64')).toBe('arm64'))
  it('x64 from "x86_64"', () => expect(formatArch('x86_64')).toBe('x64'))
  it('x64 from "amd64"', () => expect(formatArch('amd64')).toBe('x64'))
  it('x64 from "x64"', () => expect(formatArch('x64')).toBe('x64'))
  it('x64 from "AMD64" (uppercase)', () => expect(formatArch('AMD64')).toBe('x64'))
  it('x86 from "i386"', () => expect(formatArch('i386')).toBe('x86'))
  it('x86 from "i686"', () => expect(formatArch('i686')).toBe('x86'))
  it('x86 from "x86"', () => expect(formatArch('x86')).toBe('x86'))
  it('returns raw for unknown arch "mips64"', () => expect(formatArch('mips64')).toBe('mips64'))
  it('returns raw for unknown arch "riscv64"', () => expect(formatArch('riscv64')).toBe('riscv64'))
  it('returns empty string for null', () => expect(formatArch(null)).toBe(''))
  it('returns empty string for undefined', () => expect(formatArch(undefined)).toBe(''))
  it('returns empty string for empty string', () => expect(formatArch('')).toBe(''))
  it('returns empty string for whitespace only', () => expect(formatArch('  ')).toBe(''))
})
