import { cn } from '../utils/helpers'

export function LogoMark({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 64 64" fill="none" className={className} aria-hidden="true">
      <rect x="4" y="4" width="56" height="56" rx="16" fill="url(#dc-bg)" />
      <path
        d="M20 44V20h9.5c5.8 0 10.5 4.4 10.5 12s-4.7 12-10.5 12H20zm5.5-4.8h3.6c3.4 0 5.4-2.6 5.4-7.2s-2-7.2-5.4-7.2h-3.6v14.4z"
        fill="white"
      />
      <circle cx="46" cy="42" r="5" fill="#2E9BFF" />
      <defs>
        <linearGradient id="dc-bg" x1="4" y1="4" x2="60" y2="60" gradientUnits="userSpaceOnUse">
          <stop stopColor="#2E9BFF" />
          <stop offset="1" stopColor="#7C3AED" />
        </linearGradient>
      </defs>
    </svg>
  )
}

export function LogoTile({ className, markClassName }: { className?: string; markClassName?: string }) {
  return (
    <span
      className={cn(
        'flex items-center justify-center overflow-hidden rounded-2xl bg-gradient-to-br from-primary-400 to-violet-600 shadow-[0_0_24px_rgba(46,155,255,0.35)]',
        className
      )}
    >
      <LogoMark className={markClassName} />
    </span>
  )
}
