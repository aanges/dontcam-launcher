import { Home, Box, UserCog, Settings, Users, Sun, Moon } from 'lucide-react'
import { cn } from '../../utils/helpers'
import { LogoTile } from '../Logo'
import { useThemeStore, useAuthStore } from '../../store/authStore'
import { useNavigate, useLocation, Link } from 'react-router-dom'

const navigation = [
  { name: 'Home', href: '/', icon: Home },
  { name: 'Versions', href: '/versions', icon: Box },
  { name: 'Profiles', href: '/profiles', icon: UserCog },
  { name: 'Accounts', href: '/accounts', icon: Users },
  { name: 'Settings', href: '/settings', icon: Settings },
]

export function Sidebar() {
  const { resolvedTheme, setTheme } = useThemeStore()
  const { currentAccount } = useAuthStore()
  const navigate = useNavigate()
  const location = useLocation()

  return (
    <div className="flex shrink-0 py-4 pl-4">
      <div className="panel flex w-[76px] flex-col items-center gap-1.5 !rounded-[22px] px-2 py-4">
        <button
          onClick={() => navigate('/')}
          className="mb-2 transition-transform duration-300 hover:scale-105"
          aria-label="DontCam Client home"
          title="DontCam Client"
        >
          <LogoTile className="h-12 w-12 !rounded-2xl" markClassName="h-full w-full" />
        </button>

        <nav className="flex w-full flex-1 flex-col items-center gap-1" aria-label="Main navigation">
          {navigation.map((item) => {
            const isActive = location.pathname === item.href
            const Icon = item.icon
            return (
              <button
                key={item.name}
                onClick={() => navigate(item.href)}
                aria-current={isActive ? 'page' : undefined}
                title={item.name}
                className="group relative flex w-full justify-center py-1"
              >
                <span
                  className={cn(
                    'flex h-11 w-11 items-center justify-center rounded-2xl transition-all duration-200',
                    isActive
                      ? 'bg-primary-500 text-white shadow-[0_0_24px_rgba(46,155,255,0.45)]'
                      : 'text-slate-500 hover:bg-white/[0.07] hover:text-white'
                  )}
                >
                  <Icon className="h-5 w-5" strokeWidth={isActive ? 2.5 : 2} />
                </span>
                <span className="pointer-events-none absolute left-[62px] z-50 whitespace-nowrap rounded-lg border border-white/10 bg-[#0d1117] px-2.5 py-1.5 font-display text-xs font-bold text-white opacity-0 shadow-xl transition-all duration-200 group-hover:translate-x-0 group-hover:opacity-100 translate-x-1">
                  {item.name}
                </span>
              </button>
            )
          })}
        </nav>

        <div className="my-1 h-px w-10 bg-white/[0.08]" />

        <div className="flex flex-col items-center gap-2">
          <button
            onClick={() => setTheme(resolvedTheme === 'dark' ? 'light' : 'dark')}
            aria-label="Toggle theme"
            title="Toggle theme"
            className="flex h-10 w-10 items-center justify-center rounded-xl text-slate-500 transition-all hover:bg-white/[0.07] hover:text-primary-300"
          >
            {resolvedTheme === 'dark' ? <Sun className="h-[18px] w-[18px]" /> : <Moon className="h-[18px] w-[18px]" />}
          </button>
          <Link
            to="/accounts"
            title={currentAccount ? `${currentAccount.username} — Accounts` : 'Accounts'}
            className="group relative"
          >
            <div className="rounded-full bg-gradient-to-br from-primary-400 to-violet-600 p-[2px] transition-transform group-hover:scale-105">
              <div className="flex h-10 w-10 items-center justify-center overflow-hidden rounded-full bg-black">
                <span className="font-display text-sm font-bold text-primary-300">
                  {currentAccount ? currentAccount.username.charAt(0).toUpperCase() : '?'}
                </span>
              </div>
            </div>
            {currentAccount && (
              <span className="absolute bottom-0 right-0 h-3 w-3 rounded-full border-2 border-black bg-primary-400" />
            )}
          </Link>
        </div>
      </div>
    </div>
  )
}
