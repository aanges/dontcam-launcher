import { forwardRef, ButtonHTMLAttributes } from 'react'
import { cn } from '../../utils/helpers'

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'outline' | 'ghost' | 'destructive'
  size?: 'sm' | 'md' | 'lg' | 'xl' | 'icon'
  loading?: boolean
}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant = 'primary', size = 'md', loading, disabled, children, ...props }, ref) => {
    const baseStyles =
      'inline-flex items-center justify-center font-display font-bold tracking-wide rounded-xl transition-all duration-200 select-none active:scale-[0.97] disabled:opacity-40 disabled:pointer-events-none focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-400/60 focus-visible:ring-offset-2 focus-visible:ring-offset-black'

    const variants = {
      primary:
        'bg-primary-500 text-white hover:bg-primary-400 hover:shadow-[0_0_28px_rgba(46,155,255,0.45)]',
      secondary:
        'bg-white/[0.07] text-white border border-white/10 hover:bg-white/[0.12] hover:border-white/20',
      outline:
        'border border-white/15 text-slate-200 hover:border-primary-400/60 hover:text-primary-200 hover:bg-primary-400/[0.07]',
      ghost: 'text-slate-400 hover:text-white hover:bg-white/[0.06]',
      destructive:
        'bg-red-500/90 text-white hover:bg-red-400 shadow-[0_0_24px_rgba(239,68,68,0.35)]',
    }

    const sizes = {
      sm: 'h-8 px-3 text-[13px] gap-1.5',
      md: 'h-10 px-4 text-sm gap-2',
      lg: 'h-12 px-6 text-[15px] gap-2',
      xl: 'h-14 px-8 text-base gap-2.5',
      icon: 'h-9 w-9 gap-0 p-0',
    }

    return (
      <button
        ref={ref}
        className={cn(baseStyles, variants[variant], sizes[size], className)}
        disabled={disabled || loading}
        {...props}
      >
        {loading && (
          <svg className="animate-spin h-4 w-4 shrink-0" viewBox="0 0 24 24">
            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" />
            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
          </svg>
        )}
        {children}
      </button>
    )
  }
)

Button.displayName = 'Button'
