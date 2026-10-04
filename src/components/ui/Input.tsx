import { forwardRef, type InputHTMLAttributes, type TextareaHTMLAttributes } from 'react'
import { cn } from '../../utils/helpers'

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string
  error?: string
  helperText?: string
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ className, label, error, helperText, id, ...props }, ref) => {
    const inputId = id ?? (label ? `input-${label.replace(/\s+/g, '-').toLowerCase()}` : undefined)
    return (
      <div className="w-full">
        {label && (
          <label htmlFor={inputId} className="mb-1.5 block font-display text-xs font-bold uppercase tracking-[0.14em] text-slate-400">
            {label}
          </label>
        )}
        <input
          ref={ref}
          id={inputId}
          className={cn(
            'h-11 w-full rounded-xl border border-white/10 bg-black/50 px-4 text-[15px] text-white placeholder:text-slate-600 transition-all focus:border-primary-400/60 focus:outline-none focus:ring-2 focus:ring-primary-400/20 disabled:cursor-not-allowed disabled:opacity-50',
            error && 'border-red-500/70 focus:border-red-500/70 focus:ring-red-500/20',
            className
          )}
          {...props}
        />
        {error ? (
          <p className="mt-1 text-xs text-red-400">{error}</p>
        ) : helperText ? (
          <p className="mt-1 text-xs text-slate-500">{helperText}</p>
        ) : null}
      </div>
    )
  }
)

Input.displayName = 'Input'

interface TextareaProps extends TextareaHTMLAttributes<HTMLTextAreaElement> {
  label?: string
}

export const Textarea = forwardRef<HTMLTextAreaElement, TextareaProps>(
  ({ className, label, id, ...props }, ref) => {
    const inputId = id ?? (label ? `textarea-${label.replace(/\s+/g, '-').toLowerCase()}` : undefined)
    return (
      <div className="w-full">
        {label && (
          <label htmlFor={inputId} className="mb-1.5 block font-display text-xs font-bold uppercase tracking-[0.14em] text-slate-400">
            {label}
          </label>
        )}
        <textarea
          ref={ref}
          id={inputId}
          className={cn(
            'w-full rounded-xl border border-white/10 bg-black/50 px-4 py-3 text-[15px] text-white placeholder:text-slate-600 transition-all focus:border-primary-400/60 focus:outline-none focus:ring-2 focus:ring-primary-400/20',
            className
          )}
          {...props}
        />
      </div>
    )
  }
)

Textarea.displayName = 'Textarea'
