'use client'

import Link from 'next/link'
import { ArrowRight, Terminal, BookOpen, Zap, GitBranch, Brain, Code2 } from 'lucide-react'
import { DocsShell } from '@/components/layout/DocsShell'

const QUICK_START_CARDS = [
  {
    icon: <Terminal size={20} className="text-acc-text" />,
    title: 'Quick Start',
    description: 'Install the CLI and run your first scan in under 5 minutes.',
    href: '/docs/getting-started',
    cta: 'Get started',
  },
  {
    icon: <Code2 size={20} className="text-acc-text" />,
    title: 'CLI Reference',
    description: 'Every command, flag, and configuration option documented.',
    href: '/docs/cli',
    cta: 'View commands',
  },
  {
    icon: <BookOpen size={20} className="text-acc-text" />,
    title: 'Invariant Library',
    description: '50+ security checks. Browse and filter by chain and severity.',
    href: '/library',
    cta: 'Browse invariants',
  },
  {
    icon: <GitBranch size={20} className="text-acc-text" />,
    title: 'CI/CD Integration',
    description: 'Connect Truent to GitHub Actions or GitLab CI in one step.',
    href: '/docs/ci-cd',
    cta: 'Set up pipeline',
  },
  {
    icon: <Brain size={20} className="text-acc-text" />,
    title: 'AI Co-Auditor',
    description: 'How the AI layer detects protocol-level logical vulnerabilities.',
    href: '/docs/ai',
    cta: 'Learn more',
    badge: 'Pro',
  },
  {
    icon: <Zap size={20} className="text-acc-text" />,
    title: 'REST API',
    description: 'Integrate Truent headlessly into your own tooling and workflows.',
    href: '/docs/api',
    cta: 'API reference',
  },
]

const FEATURED_SNIPPETS = [
  {
    label: 'Install',
    code: 'cargo install truent-cli',
  },
  {
    label: 'Scan',
    code: 'truent check ./contracts/ --chain evm',
  },
  {
    label: 'CI',
    code: 'uses: truent-dev/truent-action@v1',
  },
]

export default function DocsPage() {
  return (
    <DocsShell pageTitle="Documentation">
      <article className="space-y-16">
        {/* Hero */}
        <div>
          <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-indigo/8 border border-indigo/20 mb-5">
            <span className="text-label-sm text-acc-text">DOCUMENTATION</span>
          </div>
          <h1 className="font-display text-5xl font-[700] text-text mb-4 leading-[1.1]">
            Truent Documentation
          </h1>
          <p className="text-body-lg text-sec max-w-2xl leading-7">
            Everything you need to audit, secure, and ship smart contracts with confidence. From first scan to CI/CD integration.
          </p>
        </div>

        {/* Quick install bar */}
        <div className="bg-surface-2 border border-hair rounded-card overflow-hidden">
          <div className="flex items-center gap-0 overflow-x-auto">
            {FEATURED_SNIPPETS.map((s, i) => (
              <div key={i} className={`flex items-center gap-4 px-5 py-3 flex-1 min-w-0 ${i < FEATURED_SNIPPETS.length - 1 ? 'border-r border-hair' : ''}`}>
                <span className="text-label-sm text-sec flex-shrink-0">{s.label}</span>
                <code className="font-mono text-sm text-acc-text truncate">{s.code}</code>
              </div>
            ))}
          </div>
        </div>

        {/* Cards grid */}
        <div>
          <h2 className="font-display text-2xl font-[600] text-text mb-6">Explore the Docs</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {QUICK_START_CARDS.map((card, idx) => (
              <Link
                key={idx}
                href={card.href}
                className="group bg-panel border border-hair rounded-card p-6 hover:border-indigo transition-all duration-200 hover:bg-panel flex flex-col gap-3"
              >
                <div className="flex items-center justify-between">
                  <div className="w-9 h-9 rounded-lg bg-indigo/10 border border-indigo/20 flex items-center justify-center">
                    {card.icon}
                  </div>
                  {card.badge && (
                    <span className="text-xs text-acc-text bg-indigo/15 border border-indigo/20 px-1.5 py-0.5 rounded font-[600]">
                      {card.badge}
                    </span>
                  )}
                </div>
                <div>
                  <h3 className="font-display text-base font-[600] text-text mb-1.5 group-hover:text-acc-text transition-colors">
                    {card.title}
                  </h3>
                  <p className="text-body-md text-sec leading-5">{card.description}</p>
                </div>
                <div className="flex items-center gap-1 text-acc-text text-xs font-[600] mt-auto group-hover:gap-2 transition-all">
                  {card.cta} <ArrowRight size={12} />
                </div>
              </Link>
            ))}
          </div>
        </div>

        {/* Pro tip box */}
        <div className="bg-indigo/8 border border-indigo/20 rounded-card p-6">
          <h3 className="font-display text-lg font-[600] text-text mb-4">Pro Tips</h3>
          <ul className="space-y-3">
            {[
              <>Add a <code className="bg-panel border border-hair px-1.5 py-0.5 rounded font-mono text-xs text-acc-text">TRUENT.md</code> in your repo root to give context to the AI Co-Auditor</>,
              'Integrate Truent into CI/CD to scan on every pull request before merge',
              'Use custom invariant libraries to define protocol-specific security rules',
              'Share reports directly with your audit team via GitHub and GitLab integrations',
            ].map((tip, i) => (
              <li key={i} className="flex items-start gap-3 text-body-md text-sec">
                <span className="text-acc-text font-[700] text-sm flex-shrink-0 mt-0.5">{String(i + 1).padStart(2, '0')}</span>
                <span className="leading-5">{tip}</span>
              </li>
            ))}
          </ul>
        </div>
      </article>
    </DocsShell>
  )
}
