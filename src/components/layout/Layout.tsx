import { Outlet } from 'react-router-dom'
import { Sidebar } from './Sidebar'
import { TitleBar } from './TitleBar'

export function Layout() {
  return (
    <div className="flex h-screen w-full flex-col overflow-hidden bg-[#0A0A12] text-white">
      <TitleBar />
      <div className="flex min-h-0 flex-1">
        <Sidebar />
        <main className="app-bg min-w-0 flex-1 overflow-y-auto transition-all duration-300">
          <div className="mx-auto max-w-7xl p-6 lg:p-8">
            <Outlet />
          </div>
        </main>
      </div>
    </div>
  )
}
