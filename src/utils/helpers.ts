import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'
import { format } from 'date-fns'

export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs))
}

/** Build -Xms/-Xmx flags from the memory settings. */
export function getMemoryArgs(min: number, max: number, unit: 'MB' | 'GB'): string[] {
  const suffix = unit === 'GB' ? 'G' : 'M'
  return [`-Xms${min}${suffix}`, `-Xmx${max}${suffix}`]
}

/** Split a JVM args string on whitespace, respecting double quotes. */
export function parseJvmArgs(raw: string): string[] {
  const out: string[] = []
  let current = ''
  let quoted = false
  for (const ch of raw.trim()) {
    if (ch === '"') {
      quoted = !quoted
      continue
    }
    if (!quoted && /\s/.test(ch)) {
      if (current) {
        out.push(current)
        current = ''
      }
      continue
    }
    current += ch
  }
  if (current) out.push(current)
  return out
}

export function formatDate(iso: string): string {
  try {
    return format(new Date(iso), 'PPpp')
  } catch {
    return iso
  }
}
