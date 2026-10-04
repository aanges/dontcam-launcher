import { createContext, useContext, useState, Children, cloneElement, isValidElement, type ReactNode, type ReactElement } from 'react'
import { cn } from '../../utils/helpers'

interface TabContextValue {
  value: string
  onChange: (v: string) => void
  variant: 'enclosed' | 'underline'
}

const TabContext = createContext<TabContextValue>({
  value: '',
  onChange: () => undefined,
  variant: 'underline',
})

interface TabsProps {
  value?: string
  defaultValue?: string
  onChange?: (value: string) => void
  variant?: 'enclosed' | 'underline'
  children: ReactNode
  className?: string
}

export function Tabs({ value, defaultValue, onChange, variant = 'underline', children, className }: TabsProps) {
  const [internal, setInternal] = useState(defaultValue ?? '')
  const active = value ?? internal
  const handleChange = (v: string) => {
    setInternal(v)
    onChange?.(v)
  }
  return (
    <TabContext.Provider value={{ value: active, onChange: handleChange, variant }}>
      <div className={cn(variant === 'enclosed' ? 'space-y-4' : 'space-y-5', className)}>{children}</div>
    </TabContext.Provider>
  )
}

export function TabsList({
  children,
  className,
  'aria-label': ariaLabel,
}: {
  children: ReactNode
  className?: string
  'aria-label'?: string
}) {
  const { value, onChange, variant } = useContext(TabContext)
  return (
    <div
      role="tablist"
      aria-label={ariaLabel}
      className={cn(
        variant === 'enclosed'
          ? 'flex flex-wrap gap-1 rounded-2xl border border-white/[0.08] bg-black/40 p-1.5'
          : 'flex gap-1 border-b border-white/[0.08]',
        className
      )}
    >
      {Children.map(children, (child) => {
        if (!isValidElement(child)) return child
        const childValue = (child.props as { value: string }).value
        const isActive = childValue === value
        return cloneElement(child as ReactElement<TabsTriggerProps>, {
          active: isActive,
          variant,
          onSelect: () => onChange(childValue),
        })
      })}
    </div>
  )
}

interface TabsTriggerProps {
  value: string
  children: ReactNode
  className?: string
  disabled?: boolean
  active?: boolean
  variant?: 'enclosed' | 'underline'
  onSelect?: () => void
}

export function TabsTrigger({ children, className, disabled, active, variant, onSelect }: TabsTriggerProps) {
  if (variant === 'enclosed') {
    return (
      <button
        role="tab"
        aria-selected={active}
        onClick={onSelect}
        disabled={disabled}
        className={cn(
          'rounded-xl px-4 py-2.5 font-display text-sm font-bold transition-all duration-200 disabled:cursor-not-allowed disabled:opacity-40',
          active
            ? 'bg-primary-500 text-white shadow-[0_0_20px_rgba(46,155,255,0.35)]'
            : 'text-slate-400 hover:text-white hover:bg-white/[0.06]',
          className
        )}
      >
        {children}
      </button>
    )
  }
  return (
    <button
      role="tab"
      aria-selected={active}
      onClick={onSelect}
      disabled={disabled}
      className={cn(
        'relative px-4 py-2.5 font-display text-sm font-bold transition-colors disabled:cursor-not-allowed disabled:opacity-40',
        active ? 'text-white' : 'text-slate-500 hover:text-slate-300',
        active &&
          'after:absolute after:inset-x-0 after:-bottom-px after:h-[2px] after:rounded-full after:bg-primary-400 after:shadow-[0_0_12px_rgba(46,155,255,0.8)]',
        className
      )}
    >
      {children}
    </button>
  )
}

export function TabsContent({ value, children, className }: { value: string; children: ReactNode; className?: string }) {
  const { value: active } = useContext(TabContext)
  if (value !== active) return null
  return <div className={cn('animate-fade-in', className)}>{children}</div>
}
