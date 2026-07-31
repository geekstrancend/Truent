'use client'

import { useState } from 'react'
import Link from 'next/link'
import { signOut } from 'next-auth/react'
import {
  LayoutDashboard,
  Shield,
  BookOpen,
  Settings,
  HelpCircle,
  BookMarked,
  LogOut,
  Plus,
  ShieldCheck,
  Menu,
  X,
  ChevronRight,
} from 'lucide-react'
import clsx from 'clsx'

interface AppShellProps {
  children: React.ReactNode
  rightPanel?: React.ReactNode
  currentPage?: 'dashboard' | 'audits' | 'library' | 'settings' | 'support'
  onNewScan?: () => void
}

const NAV_ITEMS = [
  { id: 'dashboard', label: 'Dashboard',  icon: LayoutDashboard, href: '/dashboard' },
  { id: 'audits',    label: 'Audits',      icon: Shield,           href: '/dashboard' },
  { id: 'library',   label: 'Library',     icon: BookOpen,         href: '/library' },
  { id: 'settings',  label: 'Settings',    icon: Settings,         href: '/dashboard/settings' },
  { id: 'support',   label: 'Support',     icon: HelpCircle,       href: '/dashboard/support' },
]

export function AppShell({ children, rightPanel, currentPage = 'dashboard', onNewScan }: AppShellProps) {
  const [sidebarOpen, setSidebarOpen] = useState(false)

  return (
    <div className="flex h-screen bg-bg overflow-hidden">
      {/* ── Sidebar ── */}
      <aside
        className={clsx(
          'fixed inset-y-0 left-0 z-40 w-60 bg-surface-2 border-r border-hair flex flex-col',
          'transform transition-transform duration-300',
          sidebarOpen ? 'translate-x-0' : '-translate-x-full md:translate-x-0',
        )}
      >
        {/* Logo */}
        <div className="p-4 border-b border-hair">
          <Link href="/dashboard" className="flex items-center gap-2.5 hover:opacity-80 transition-opacity">
            <ShieldCheck size={18} className="text-acc-text" />
            <span className="font-display text-base font-[600] text-text">Truent</span>
          </Link>
          <div className="mt-3 flex items-center gap-2">
            <div className="w-2 h-2 rounded-full bg-low animate-pulse-dot" />
            <span className="text-xs text-low font-[600]">All systems operational</span>
          </div>
        </div>

        {/* New Scan button */}
        <div className="px-3 pt-4 pb-2">
          {onNewScan ? (
            <button
              onClick={onNewScan}
              className="w-full flex items-center justify-center gap-2 px-3 py-2.5 bg-acc/15 border border-indigo text-acc-text rounded-lg text-body-md font-[600] hover:bg-indigo/90 transition-colors"
            >
              <Plus size={15} />
              New Scan
            </button>
          ) : (
            <Link
              href="/dashboard/scan"
              className="w-full flex items-center justify-center gap-2 px-3 py-2.5 bg-acc/15 border border-indigo text-acc-text rounded-lg text-body-md font-[600] hover:bg-indigo/90 transition-colors"
            >
              <Plus size={15} />
              New Scan
            </Link>
          )}
        </div>

        {/* Nav */}
        <nav className="flex-1 px-2 py-2 space-y-0.5 overflow-y-auto">
          {NAV_ITEMS.map((item) => {
            const Icon = item.icon
            const isActive = currentPage === item.id
            return (
              <Link
                key={item.id}
                href={item.href}
                onClick={() => setSidebarOpen(false)}
                className={clsx(
                  'flex items-center gap-3 px-3 py-2.5 rounded-lg text-body-md transition-colors',
                  isActive
                    ? 'bg-indigo/10 text-text font-[500] border-l-2 border-indigo pl-[10px]'
                    : 'text-sec hover:bg-panel hover:text-text',
                )}
              >
                <Icon size={17} className="flex-shrink-0" />
                <span>{item.label}</span>
              </Link>
            )
          })}
        </nav>

        {/* Bottom links */}
        <div className="border-t border-hair p-3 space-y-0.5">
          <Link href="/docs"
            className="flex items-center gap-3 px-3 py-2 rounded-lg text-sec hover:bg-panel hover:text-text transition-colors text-body-md">
            <BookMarked size={16} />
            <span>Documentation</span>
          </Link>
          <button
            onClick={() => signOut({ callbackUrl: '/' })}
            className="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sec hover:bg-panel hover:text-text transition-colors text-body-md"
          >
            <LogOut size={16} />
            <span>Sign Out</span>
          </button>
        </div>

        {/* User profile strip */}
        <div className="border-t border-hair p-3">
          <div className="flex items-center gap-3 px-2 py-2">
            <div className="w-7 h-7 rounded-full bg-indigo/20 border border-indigo/30 flex items-center justify-center flex-shrink-0">
              <span className="text-xs font-[700] text-acc-text">A</span>
            </div>
            <div className="flex-1 min-w-0">
              <p className="text-body-md font-[500] text-text truncate">Alex Developer</p>
              <p className="text-xs text-sec truncate">Pro Plan</p>
            </div>
            <ChevronRight size={14} className="text-sec flex-shrink-0" />
          </div>
        </div>
      </aside>

      {/* ── Main ── */}
      <div className="flex-1 flex flex-col md:ml-60 overflow-hidden">
        {/* Mobile header */}
        <header className="md:hidden flex items-center justify-between px-4 py-3 border-b border-hair bg-surface-2">
          <div className="flex items-center gap-2">
            <ShieldCheck size={18} className="text-acc-text" />
            <span className="font-display text-sm font-[600] text-text">Truent</span>
          </div>
          <button
            onClick={() => setSidebarOpen(!sidebarOpen)}
            aria-label={sidebarOpen ? 'Close menu' : 'Open menu'}
            className="p-2 hover:bg-panel rounded-lg text-sec transition-colors"
          >
            {sidebarOpen ? <X size={20} /> : <Menu size={20} />}
          </button>
        </header>

        {/* Content area */}
        <div className="flex-1 flex overflow-hidden">
          <main className="flex-1 overflow-y-auto">{children}</main>
          {rightPanel && (
            <div className="hidden lg:block w-72 border-l border-hair bg-surface-2 overflow-y-auto">
              {rightPanel}
            </div>
          )}
        </div>
      </div>

      {/* Mobile overlay */}
      {sidebarOpen && (
        <div className="fixed inset-0 bg-black/50 z-30 md:hidden" onClick={() => setSidebarOpen(false)} />
      )}
    </div>
  )
}
