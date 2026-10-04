import { forwardRef, type SelectHTMLAttributes } from 'react'
import { cn } from '../../utils/helpers'

interface SelectProps extends SelectHTMLAttributes<HTMLSelectElement> {
  label?: string
  error?: string
  helperText?: string
  options: Array<{ value: string; label: string }>
  placeholder?: string
}

export const Select = forwardRef<HTMLSelectElement, SelectProps>(
  ({ className, label, error, helperText, options, placeholder, id, ...props }, ref) => {
    const selectId = id ?? (label ? `select-${label.replace(/\s+/g, '-').toLowerCase()}` : undefined)
    return (
      <div className="w-full">
        {label && (
          <label htmlFor={selectId} className="mb-1.5 block font-display text-xs font-bold uppercase tracking-[0.14em] text-slate-400">
            {label}
          </label>
        )}
        <select
          ref={ref}
          id={selectId}
          className={cn(
            'h-11 w-full appearance-none rounded-xl border border-white/10 bg-black/50 pl-4 pr-10 text-[15px] text-white transition-all duration-200 focus:border-primary-400/60 focus:outline-none focus:ring-2 focus:ring-primary-400/20 disabled:cursor-not-allowed disabled:opacity-50',
            'bg-[url("data:image/svg+xml,%3csvg xmlns=%27http://www.w3.org/2000/svg%27 fill=%27none%27 viewBox=%270 0 20 20%27%3e%3cpath stroke=%27%232E9BFF%27 stroke-linecap=%27round%27 stroke-linejoin=%27round%27 stroke-width=%271.8%27 d=%27M6 8l4 4 4-4%27/%3e%3c/svg%3e")] bg-[length:1.25rem_1.25rem] bg-[right_0.75rem_center] bg-no-repeat',
            error && 'border-red-500/70 focus:border-red-500/70 focus:ring-red-500/20',
            className
          )}
          aria-invalid={error ? 'true' : 'false'}
          {...props}
        >
          {placeholder && (
            <option value="" disabled>
              {placeholder}
            </option>
          )}
          {options.map((opt) => (
            <option key={opt.value} value={opt.value} className="bg-[#0d1117]">
              {opt.label}
            </option>
          ))}
        </select>
        {error ? (
          <p className="mt-1 text-xs text-red-400">{error}</p>
        ) : helperText ? (
          <p className="mt-1 text-xs text-slate-500">{helperText}</p>
        ) : null}
      </div>
    )
  }
)

Select.displayName = 'Select'

interface CheckboxProps {
  label: string
  checked: boolean
  onChange: (e: { target: { checked: boolean } }) => void
  disabled?: boolean
  className?: string
}

export function Checkbox({ label, checked, onChange, disabled, className }: CheckboxProps) {
  const switchId = `switch-${label.replace(/\s+/g, '-').toLowerCase()}`
  return (
    <div className={cn('flex items-center gap-3', className)}>
      <button
        type="button"
        role="switch"
        aria-checked={checked}
        aria-labelledby={switchId}
        className={cn(
          'relative inline-flex h-7 w-12 shrink-0 items-center rounded-full border transition-all duration-200',
          'focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-400/50',
          checked
            ? 'border-primary-400/50 bg-primary-500 shadow-[0_0_16px_rgba(46,155,255,0.4)]'
            : 'border-white/10 bg-white/[0.08]',
          disabled && 'cursor-not-allowed opacity-50'
        )}
        onClick={() => !disabled && onChange?.({ target: { checked: !checked } })}
      >
        <span
          className={cn(
            'inline-block h-5 w-5 transform rounded-full bg-white shadow transition-transform duration-200',
            checked ? 'translate-x-[22px]' : 'translate-x-[3px]'
          )}
        />
      </button>
      <span id={switchId} className="text-sm text-slate-300">
        {label}
      </span>
    </div>
  )
}
