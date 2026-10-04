import { forwardRef, SelectHTMLAttributes } from 'react'
import { cn } from '../../utils/helpers'

const labelStyles = 'mb-1.5 block text-[13px] font-semibold text-slate-300'

interface SelectProps extends SelectHTMLAttributes<HTMLSelectElement> {
  label?: string
  error?: string
  helperText?: string
  options: { value: string; label: string; disabled?: boolean }[]
  placeholder?: string
}

export const Select = forwardRef<HTMLSelectElement, SelectProps>(
  ({ className, label, error, helperText, options, placeholder, id, ...props }, ref) => {
    const selectId = id || label?.toLowerCase().replace(/\s+/g, '-')

    return (
      <div className="w-full">
        {label && (
          <label htmlFor={selectId} className={labelStyles}>
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
          aria-describedby={error ? `${selectId}-error` : helperText ? `${selectId}-helper` : undefined}
          {...props}
        >
          {placeholder && (
            <option value="" disabled>
              {placeholder}
            </option>
          )}
          {options.map((option) => (
            <option key={option.value} value={option.value} disabled={option.disabled}>
              {option.label}
            </option>
          ))}
        </select>
        {error && (
          <p id={`${selectId}-error`} className="mt-1.5 text-[13px] text-red-400" role="alert">
            {error}
          </p>
        )}
        {helperText && !error && (
          <p id={`${selectId}-helper`} className="mt-1.5 text-[13px] text-slate-500">
            {helperText}
          </p>
        )}
      </div>
    )
  }
)

Select.displayName = 'Select'

interface CheckboxProps extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'type'> {
  label: string
  description?: string
}

export const Checkbox = forwardRef<HTMLInputElement, CheckboxProps>(
  ({ className, label, description, id, ...props }, ref) => {
    const checkboxId = id || label.toLowerCase().replace(/\s+/g, '-')

    return (
      <div className="flex cursor-pointer items-start gap-3 rounded-xl border border-transparent p-1 transition-colors hover:border-white/[0.07] hover:bg-white/[0.03]">
        <input
          ref={ref}
          type="checkbox"
          id={checkboxId}
          className={cn(
            'mt-0.5 h-5 w-5 shrink-0 cursor-pointer appearance-none rounded-md border border-white/20 bg-black/50 transition-all duration-150',
            'checked:border-primary-400 checked:bg-primary-400 checked:bg-[url("data:image/svg+xml,%3csvg xmlns=%27http://www.w3.org/2000/svg%27 viewBox=%270 0 20 20%27 fill=%27none%27 stroke=%27%23000%27 stroke-width=%273%27 stroke-linecap=%27round%27 stroke-linejoin=%27round%27%3e%3cpath d=%27M4 10l4 4 8-8%27/%3e%3c/svg%3e")] checked:bg-center checked:bg-no-repeat',
            'focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-400/50',
            className
          )}
          {...props}
        />
        <div className="flex flex-col">
          <label htmlFor={checkboxId} className="cursor-pointer text-sm font-semibold text-white">
            {label}
          </label>
          {description && (
            <p className="mt-0.5 text-[13px] text-slate-500">
              {description}
            </p>
          )}
        </div>
      </div>
    )
  }
)

Checkbox.displayName = 'Checkbox'

interface SwitchProps extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'type'> {
  label: string
  description?: string
}

export const Switch = forwardRef<HTMLInputElement, SwitchProps>(
  ({ className, label, description, id, ...props }, _ref) => {
    const switchId = id || label.toLowerCase().replace(/\s+/g, '-')

    return (
      <div className="flex items-center gap-3">
        <button
          type="button"
          role="switch"
          aria-checked={props.checked}
          aria-labelledby={switchId}
          className={cn(
            'relative inline-flex h-7 w-12 shrink-0 items-center rounded-full border transition-all duration-200',
            'focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-400/50',
            props.checked
              ? 'border-primary-400/50 bg-primary-500 shadow-[0_0_16px_rgba(46,155,255,0.4)]'
              : 'border-white/10 bg-white/[0.08]',
            props.disabled && 'cursor-not-allowed opacity-50',
            className
          )}
          onClick={() => !props.disabled && props.onChange?.({ target: { checked: !props.checked } } as any)}
        >
          <span
            className={cn(
              'inline-block h-5 w-5 transform rounded-full shadow transition-transform duration-200',
              props.checked ? 'translate-x-[22px] bg-black' : 'translate-x-[3px] bg-slate-300'
            )}
          />
        </button>
        <div className="flex flex-col">
          <label id={switchId} className="cursor-pointer text-sm font-semibold text-white">
            {label}
          </label>
          {description && (
            <p className="text-[13px] text-slate-500">
              {description}
            </p>
          )}
        </div>
      </div>
    )
  }
)

Switch.displayName = 'Switch'
