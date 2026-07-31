'use client'

import Link from 'next/link'
import { ArrowRight, ShieldCheck, BookOpen, LayoutDashboard } from 'lucide-react'
import { Button } from '@/components/ui/Button'
import { MarketingNav } from '@/components/layout/MarketingNav'
import { MarketingFooter } from '@/components/layout/MarketingFooter'

export default function NotFound() {
  return (
    <div className="min-h-screen bg-bg flex flex-col">
      <MarketingNav />

      <main className="flex-1 flex items-center justify-center px-6 py-24 relative">
        <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
          <div className="w-[600px] h-[400px] bg-indigo/5 rounded-full blur-3xl" />
        </div>

        <div className="text-center max-w-lg relative">
          <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-critical/10 border border-critical/20 mb-8">
            <span className="text-label-sm text-critical">404 · PAGE NOT FOUND</span>
          </div>

          <h1 className="font-display text-8xl font-[700] text-text mb-2 leading-none tracking-tight">
            404
          </h1>
          <h2 className="font-display text-2xl font-[600] text-sec mb-5">
            Nothing to audit here
          </h2>
          <p className="text-body-lg text-sec mb-10 leading-7">
            This page doesn&apos;t exist or was moved. Let&apos;s get you somewhere useful.
          </p>

          <div className="grid grid-cols-1 sm:grid-cols-3 gap-3 mb-10">
            {[
              { icon: ShieldCheck, label: 'Homepage', href: '/' },
              { icon: BookOpen, label: 'Invariant Library', href: '/library' },
              { icon: LayoutDashboard, label: 'Dashboard', href: '/dashboard' },
            ].map((link) => {
              const Icon = link.icon
              return (
                <Link
                  key={link.href}
                  href={link.href}
                  className="flex items-center justify-center gap-2 px-4 py-3 bg-panel border border-hair rounded-card text-body-md text-sec hover:border-indigo hover:text-text transition-colors"
                >
                  <Icon size={16} className="text-acc-text" />
                  {link.label}
                </Link>
              )
            })}
          </div>

          <div className="flex flex-col sm:flex-row gap-3 justify-center mb-12">
            <Link href="/">
              <Button variant="primary" size="lg" icon={<ArrowRight size={16} />} iconPosition="right">
                Back to Home
              </Button>
            </Link>
            <a href="mailto:support@truent.dev">
              <Button variant="secondary" size="lg">Contact Support</Button>
            </a>
          </div>

          <div className="bg-surface-2 border border-hair rounded-card p-4 text-left font-mono text-xs">
            <div className="flex gap-1.5 mb-3">
              <div className="w-2 h-2 rounded-full bg-critical" />
              <div className="w-2 h-2 rounded-full bg-high" />
              <div className="w-2 h-2 rounded-full bg-low" />
            </div>
            <p className="text-sec"><span className="text-low">$</span> truent check --url /404</p>
            <p className="text-critical mt-1">[CRITICAL] Route not found in manifest</p>
            <p className="text-sec mt-1">[INFO] Suggestion: navigate to /dashboard</p>
            <p className="text-low mt-1">[DONE] Redirecting you to safety... <span className="animate-blink-cursor">▊</span></p>
          </div>
        </div>
      </main>

      <MarketingFooter />
    </div>
  )
}
