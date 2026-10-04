import { cn } from '../utils/helpers'

/**
 * DontCam "DC" infinity mark — brand PNG (white mark on black).
 * LogoTile renders it on a black rounded tile; LogoMark is the raw image
 * (use `mix-blend-screen` on dark surfaces so the black background
 * disappears and only the white mark shows).
 */
export function LogoMark({ className }: { className?: string }) {
  return (
    <img
      src="/icon.png"
      alt=""
      aria-hidden="true"
      draggable={false}
      className={cn('object-contain mix-blend-screen', className)}
    />
  )
}

export function LogoTile({
  className,
  markClassName,
  title = 'DontCam Client',
}: {
  className?: string
  markClassName?: string
  title?: string
}) {
  return (
    <div
      title={title}
      className={cn(
        'rounded-2xl bg-black',
        'flex items-center justify-center overflow-hidden shrink-0',
        'border border-white/10',
        className
      )}
    >
      <img
        src="/icon.png"
        alt="DontCam Client logo"
        draggable={false}
        className={cn('h-full w-full object-contain', markClassName)}
      />
    </div>
  )
}
