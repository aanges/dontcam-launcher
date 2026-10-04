import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function formatBytes(bytes: number, decimals = 2): string {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const dm = decimals < 0 ? 0 : decimals
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i]
}

export function formatDuration(ms: number): string {
  const seconds = Math.floor(ms / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)

  if (days > 0) return `${days}d ${hours % 24}h`
  if (hours > 0) return `${hours}h ${minutes % 60}m`
  if (minutes > 0) return `${minutes}m ${seconds % 60}s`
  return `${seconds}s`
}

export function formatDate(date: Date | string): string {
  const d = new Date(date)
  return d.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  })
}

export function formatDateTime(date: Date | string): string {
  const d = new Date(date)
  return d.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

export function generateId(): string {
  return crypto.randomUUID()
}

export function debounce<T extends (...args: unknown[]) => unknown>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: ReturnType<typeof setTimeout> | null = null
  return (...args: Parameters<T>) => {
    if (timeout) clearTimeout(timeout)
    timeout = setTimeout(() => func(...args), wait)
  }
}

export function throttle<T extends (...args: unknown[]) => unknown>(
  func: T,
  limit: number
): (...args: Parameters<T>) => void {
  let inThrottle = false
  return (...args: Parameters<T>) => {
    if (!inThrottle) {
      func(...args)
      inThrottle = true
      setTimeout(() => (inThrottle = false), limit)
    }
  }
}

export function parseJvmArgs(args: string): string[] {
  return args
    .split(' ')
    .map((arg) => arg.trim())
    .filter((arg) => arg.length > 0)
}

export function getMemoryArgs(min: number, max: number, unit: 'MB' | 'GB'): string[] {
  const minMb = unit === 'GB' ? min * 1024 : min
  const maxMb = unit === 'GB' ? max * 1024 : max
  return [`-Xms${Math.max(256, minMb)}M`, `-Xmx${Math.max(maxMb, minMb)}M`]
}

/// Recommended Java major for a Minecraft version (mirrors backend logic).
export function recommendedJava(mcVersion: string): number {
  const base = mcVersion.split('-')[0]
  const parts = base.split('.').map((p) => parseInt(p, 10)).filter((n) => !Number.isNaN(n))
  if (parts.length === 0 || parts[0] !== 1) return 21
  const minor = parts[1] ?? 0
  const patch = parts[2] ?? 0
  if (minor <= 16) return 8
  if (minor === 17) return 16
  if (minor === 18 || minor === 19) return 17
  if (minor === 20) return patch >= 5 ? 21 : 17
  return 21
}

export function getOsName(): string {
  if (typeof navigator !== 'undefined') {
    const platform = navigator.platform.toLowerCase()
    if (platform.includes('win')) return 'windows'
    if (platform.includes('mac')) return 'macos'
    if (platform.includes('linux')) return 'linux'
  }
  return 'unknown'
}

export function getArch(): string {
  if (typeof navigator !== 'undefined') {
    const arch = navigator.userAgent.toLowerCase()
    if (arch.includes('x64') || arch.includes('x86_64')) return 'x64'
    if (arch.includes('arm64') || arch.includes('aarch64')) return 'arm64'
    if (arch.includes('x86') || arch.includes('i386') || arch.includes('i686')) return 'x86'
  }
  return 'x64'
}