import { ReactNode, createContext, useContext, useState } from 'react'
import { cn } from '../../utils/helpers'

interface TabsContextValue {
  value: string
  onChange: (value: string) => void
  variant: 'line' | 'enclosed' | 'soft'
}

const TabsContext = createContext<TabsContextValue>({
  value: '',
  onChange: () => undefined,
  variant: 'line',
})

interface TabsProps {
  defaultValue?: string
  value?: string
  onChange?: (value: string) => void
  children: ReactNode
  className?: string
  variant?: 'line' | 'enclosed' | 'soft'
}

export function Tabs({ defaultValue, value, onChange, children, className, variant = 'line' }: TabsProps) {
  // uncontrolled if `value` is undefined
  return (
    <UncontrolledTabs
      defaultValue={defaultValue ?? value ?? ''}
      controlledValue={value}
      onChange={onChange}
      className={className}
      variant={variant}
    >
      {children}
    </UncontrolledTabs>
  )
}

function UncontrolledTabs({
  defaultValue,
  controlledValue,
  onChange,
  children,
  className,
  variant,
}: {
  defaultValue: string
  controlledValue?: string
  onChange?: (value: string) => void
  children: ReactNode
  className?: string
  variant: 'line' | 'enclosed' | 'soft'
}) {
  const [internal, setInternal] = useState(defaultValue)
  const current = controlledValue ?? internal

  const handleChange = (newValue: string) => {
    if (controlledValue === undefined) setInternal(newValue)
    onChange?.(newValue)
  }

  return (
    <TabsContext.Provider value={{ value: current, onChange: handleChange, variant }}>
      <div className={cn('', className)} data-variant={variant}>
        {children}
      </div>
    </TabsContext.Provider>
  )
}

interface TabsListProps {
  children: ReactNode
  className?: string
  'aria-label'?: string
}

export function TabsList({ children, className, 'aria-label': ariaLabel }: TabsListProps) {
  const { variant } = useContext(TabsContext)
  return (
    <div
      role="tablist"
      aria-label={ariaLabel}
      data-variant={variant}
      className={cn(
        'flex flex-wrap',
        variant === 'line' && 'gap-6 border-b border-white/[0.08]',
        variant === 'enclosed' && 'gap-1 rounded-2xl border border-white/[0.08] bg-black/40 p-1.5',
        variant === 'soft' && 'gap-2',
        className
      )}
    >
      {children}
    </div>
  )
}

interface TabsTriggerProps {
  value: string
  children: ReactNode
  className?: string
  disabled?: boolean
}

export function TabsTrigger({ value, children, className, disabled }: TabsTriggerProps) {
  const { value: current, onChange, variant } = useContext(TabsContext)
  const isActive = current === value

  const variants = {
    line: cn(
      'relative px-1 py-3 font-display text-sm font-bold tracking-wide transition-colors duration-200',
      'focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-400/50 rounded-t-lg',
      isActive ? 'text-white' : 'text-slate-500 hover:text-slate-200',
      isActive && 'after:absolute after:inset-x-0 after:-bottom-px after:h-[2px] after:rounded-full after:bg-primary-400 after:shadow-[0_0_12px_rgba(46,155,255,0.8)]',
      disabled && 'cursor-not-allowed opacity-40'
    ),
    enclosed: cn(
      'flex-1 rounded-xl px-4 py-2.5 font-display text-sm font-bold tracking-wide transition-all duration-200 sm:flex-none',
      'focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-400/50',
      isActive
        ? 'bg-primary-500 text-white shadow-[0_0_20px_rgba(46,155,255,0.35)]'
        : 'text-slate-400 hover:bg-white/[0.06] hover:text-white',
      disabled && 'cursor-not-allowed opacity-40'
    ),
    soft: cn(
      'rounded-full border px-4 py-2 font-display text-[13px] font-bold tracking-wide transition-all duration-200',
      'focus:outline-none focus-visible:ring-2 focus-visible:ring-primary-400/50',
      isActive
        ? 'border-primary-400/50 bg-primary-400/15 text-primary-200'
        : 'border-white/10 bg-white/[0.03] text-slate-400 hover:border-white/20 hover:text-white',
      disabled && 'cursor-not-allowed opacity-40'
    ),
  }

  return (
    <button
      role="tab"
      aria-selected={isActive}
      aria-controls={`panel-${value}`}
      id={`tab-${value}`}
      onClick={() => !disabled && onChange(value)}
      disabled={disabled}
      className={cn(variants[variant], className)}
    >
      {children}
    </button>
  )
}

interface TabsContentProps {
  value: string
  children: ReactNode
  className?: string
}

export function TabsContent({ value, children, className }: TabsContentProps) {
  const { value: current } = useContext(TabsContext)
  if (current !== value) return null
  return (
    <div
      role="tabpanel"
      id={`panel-${value}`}
      aria-labelledby={`tab-${value}`}
      className={cn('mt-5 animate-fade-in', className)}
    >
      {children}
    </div>
  )
}
