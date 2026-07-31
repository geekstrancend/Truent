'use client'

import { useState } from 'react'
import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { ShieldCheck, Menu, X } from 'lucide-react'
import { Button } from '../ui/Button'
import { AuthModal } from '../ui/AuthModal'
import clsx from 'clsx'

interface MarketingNavProps {
  className?: string
}

const navLinks = [
  { label: 'Product', href: '/#product' },
  { label: 'Features', href: '/#features' },
  { label: 'Library', href: '/library' },
  { label: 'Pricing', href: '/pricing' },
  { label: 'Docs', href: '/docs' },
]

export function MarketingNav({ className }: MarketingNavProps) {
  const [authOpen, setAuthOpen] = useState(false)
  const [authTab, setAuthTab] = useState<'signin' | 'signup'>('signin')
  const [mobileOpen, setMobileOpen] = useState(false)
  const pathname = usePathname()

  const handleLogIn = () => { setAuthTab('signin'); setAuthOpen(true) }
  const handleStartTrial = () => { setAuthTab('signup'); setAuthOpen(true) }

  return (
    <>
      <nav className={clsx(
        'sticky top-0 z-40 bg-surface-2/95 backdrop-blur-sm border-b border-hair',
        className,
      )}>
        <div className="max-w-site mx-auto px-7 py-4 flex items-center justify-between">
          {/* Logo */}
          <Link href="/" className="flex items-center gap-2 hover:opacity-80 transition-opacity">
            <ShieldCheck size={20} className="text-acc-text" />
            <span className="font-mono font-[600] text-text text-base tracking-tight">Truent</span>
          </Link>

          {/* Desktop nav links */}
          <div className="hidden md:flex items-center gap-8">
            {navLinks.map((link) => (
              <Link
                key={link.href}
                href={link.href}
                className={clsx(
                  'text-body-md transition-colors',
                  pathname === link.href.replace('/#', '/') || (link.href !== '/' && pathname?.startsWith(link.href.split('#')[0]))
                    ? 'text-text font-[500]'
                    : 'text-sec hover:text-text',
                )}
              >
                {link.label}
              </Link>
            ))}
          </div>

          {/* Desktop buttons */}
          <div className="hidden md:flex items-center gap-3">
            <Button variant="ghost" size="sm" onClick={handleLogIn}>Log In</Button>
            <Button variant="primary" size="sm" onClick={handleStartTrial}>Start Free Trial</Button>
          </div>

          {/* Mobile hamburger */}
          <button
            className="md:hidden p-2 -mr-2 hover:bg-panel rounded-lg transition-colors"
            onClick={() => setMobileOpen(!mobileOpen)}
            aria-label="Toggle navigation"
          >
            {mobileOpen
              ? <X size={20} className="text-text" />
              : <Menu size={20} className="text-text" />}
          </button>
        </div>

        {/* Mobile dropdown */}
        {mobileOpen && (
          <div className="md:hidden border-t border-hair bg-surface-2">
            <div className="px-7 py-4 space-y-1">
              {navLinks.map((link) => (
                <Link
                  key={link.href}
                  href={link.href}
                  className="flex items-center px-3 py-2.5 rounded-lg text-sec hover:text-text hover:bg-panel transition-colors text-body-md"
                  onClick={() => setMobileOpen(false)}
                >
                  {link.label}
                </Link>
              ))}
            </div>
            <div className="px-7 pb-4 border-t border-hair pt-4 flex flex-col gap-3">
              <Button variant="secondary" fullWidth onClick={() => { handleLogIn(); setMobileOpen(false) }}>Log In</Button>
              <Button variant="primary" fullWidth onClick={() => { handleStartTrial(); setMobileOpen(false) }}>Start Free Trial</Button>
            </div>
          </div>
        )}
      </nav>

      <AuthModal isOpen={authOpen} onClose={() => setAuthOpen(false)} defaultTab={authTab} />
    </>
  )
}
