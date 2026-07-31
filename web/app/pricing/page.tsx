'use client'

import { useState } from 'react'
import Link from 'next/link'
import { Check, X, ArrowRight, ChevronDown, ShieldCheck } from 'lucide-react'
import { useReveal } from '@/components/hooks/useReveal'
import { MarketingNav } from '@/components/layout/MarketingNav'
import { MarketingFooter } from '@/components/layout/MarketingFooter'
import { Button } from '@/components/ui/Button'
import { AuthModal } from '@/components/ui/AuthModal'
import clsx from 'clsx'

type BillingCycle = 'monthly' | 'annual'

const PLANS = [
  {
    id: 'starter',
    name: 'Starter',
    description: 'For indie auditors & early-stage projects',
    monthlyPrice: 0,
    cta: 'Get Started Free',
    ctaVariant: 'secondary' as const,
    featured: false,
    badge: null,
    highlights: [
      '5 scans per month',
      'Public invariant library',
      'PDF export & shareable links',
      'Community support',
    ],
  },
  {
    id: 'pro',
    name: 'Professional',
    description: 'For teams shipping to production',
    monthlyPrice: 499,
    cta: 'Start Free Trial',
    ctaVariant: 'primary' as const,
    featured: true,
    badge: 'MOST POPULAR',
    inherits: 'Everything in Starter, plus',
    highlights: [
      'Unlimited scans, 10 in parallel',
      'Full invariant library + symbolic execution',
      'AI Co-Auditor with engine-verified findings',
      'CI/CD gate, REST API, Slack alerts',
      'Formal verification proofs',
    ],
  },
  {
    id: 'enterprise',
    name: 'Enterprise',
    description: 'For organisations securing billions',
    monthlyPrice: null,
    cta: 'Contact Sales',
    ctaVariant: 'secondary' as const,
    featured: false,
    badge: null,
    inherits: 'Everything in Professional, plus',
    highlights: [
      'Custom invariants & private repository',
      'On-premises deployment',
      'SSO / SAML, white-label reports',
      '24/7 security advisor & SLA',
    ],
  },
]

const COMPARISON_ROWS = [
  {
    category: 'Scans',
    rows: [
      { feature: 'Scans per month', starter: '5', pro: 'Unlimited', enterprise: 'Unlimited' },
      { feature: 'Scan depth', starter: 'Standard', pro: 'Deep', enterprise: 'Deep + Custom' },
      { feature: 'Parallel scan jobs', starter: '1', pro: '10', enterprise: 'Unlimited' },
    ],
  },
  {
    category: 'Security Engine',
    rows: [
      { feature: 'Invariant library access', starter: 'Public only', pro: 'Full', enterprise: 'Full + Custom' },
      { feature: 'Symbolic execution', starter: false, pro: true, enterprise: true },
      { feature: 'AI Co-Auditor', starter: false, pro: true, enterprise: true },
      { feature: 'Self-improving engine', starter: false, pro: true, enterprise: true },
    ],
  },
  {
    category: 'Integrations',
    rows: [
      { feature: 'GitHub / GitLab CI/CD', starter: false, pro: true, enterprise: true },
      { feature: 'Slack / Discord alerts', starter: false, pro: true, enterprise: true },
      { feature: 'REST API access', starter: false, pro: true, enterprise: true },
      { feature: 'SSO / SAML', starter: false, pro: false, enterprise: true },
    ],
  },
  {
    category: 'Reports',
    rows: [
      { feature: 'PDF report export', starter: true, pro: true, enterprise: true },
      { feature: 'Shareable report links', starter: true, pro: true, enterprise: true },
      { feature: 'White-label reports', starter: false, pro: false, enterprise: true },
      { feature: 'Formal verification proofs', starter: false, pro: true, enterprise: true },
    ],
  },
  {
    category: 'Support',
    rows: [
      { feature: 'Community support', starter: true, pro: true, enterprise: true },
      { feature: 'Priority email support', starter: false, pro: true, enterprise: true },
      { feature: '24/7 security advisor', starter: false, pro: false, enterprise: true },
      { feature: 'Dedicated onboarding', starter: false, pro: false, enterprise: true },
    ],
  },
  {
    category: 'Deployment',
    rows: [
      { feature: 'Cloud hosted', starter: true, pro: true, enterprise: true },
      { feature: 'On-premises deployment', starter: false, pro: false, enterprise: true },
      { feature: 'Private invariant repository', starter: false, pro: false, enterprise: true },
      { feature: 'SLA guarantee', starter: false, pro: false, enterprise: true },
    ],
  },
]

const FAQS = [
  {
    q: 'What counts as a "scan"?',
    a: 'A scan is one analysis run on a set of contracts. You can include multiple Solidity, Rust, or Move files in a single scan. Truent runs all 50+ invariant checks plus symbolic execution in one pass.',
  },
  {
    q: 'Can I try Professional features before paying?',
    a: 'Yes — the Professional plan includes a 14-day free trial with full access to the AI Co-Auditor, unlimited scans, and CI/CD integrations. No credit card required to start.',
  },
  {
    q: 'Which chains are supported?',
    a: 'Truent currently supports EVM-compatible chains (Ethereum, Arbitrum, Base, Polygon, Optimism, Avalanche, BNB Chain), Solana (Anchor programs), and Move-based chains (Aptos, Sui). More chains are added regularly.',
  },
  {
    q: 'How does annual billing work?',
    a: 'Annual billing is charged once per year at a 20% discount off the monthly rate. You receive one invoice per year and can cancel before renewal for a prorated refund.',
  },
  {
    q: 'What is the Enterprise SLA?',
    a: 'Enterprise customers receive a 99.9% uptime SLA for the scanning API and a maximum 4-hour response time for P1 security incidents. Custom SLAs are available on request.',
  },
  {
    q: 'Can I use Truent for client audit work?',
    a: 'Yes. The Professional plan allows you to generate reports for up to 10 separate client protocols per month. Enterprise customers have unlimited client workspaces and white-label reporting.',
  },
]

function CellValue({ value }: { value: string | boolean }) {
  if (value === true) return <Check size={16} className="text-low mx-auto" />
  if (value === false) return <X size={14} className="text-sec mx-auto" />
  return <span className="text-body-md text-sec">{value}</span>
}

export default function PricingPage() {
  const [billing, setBilling] = useState<BillingCycle>('monthly')
  const [openFaq, setOpenFaq] = useState<number | null>(null)
  const [authOpen, setAuthOpen] = useState(false)
  const [authTab, setAuthTab] = useState<'signin' | 'signup'>('signin')

  const cardsRef = useReveal()
  const tableRef = useReveal()
  const faqRef = useReveal()

  const getPrice = (monthlyPrice: number | null) => {
    if (monthlyPrice === null) return null
    if (monthlyPrice === 0) return 0
    return billing === 'annual' ? Math.round(monthlyPrice * 0.8) : monthlyPrice
  }

  return (
    <div className="min-h-screen bg-bg flex flex-col">
      <MarketingNav />

      <main className="flex-1">
        {/* Hero */}
        <section className="px-6 py-20 max-w-5xl mx-auto text-center">
          <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-indigo/8 border border-indigo/20 mb-6">
            <span className="text-label-sm text-acc-text">SIMPLE, TRANSPARENT PRICING</span>
          </div>
          <h1 className="font-display text-5xl font-[700] text-text mb-4 leading-[64px]">
            Plans for every stage
          </h1>
          <p className="text-body-lg text-sec max-w-xl mx-auto mb-10">
            Start free. Scale when you&apos;re ready. No hidden fees.
          </p>

          {/* Billing toggle */}
          <div className="inline-flex items-center gap-1 p-1 bg-panel border border-hair rounded-lg">
            <button
              onClick={() => setBilling('monthly')}
              className={clsx(
                'px-4 py-1.5 rounded text-sm font-[600] transition-colors',
                billing === 'monthly' ? 'bg-panel text-text' : 'text-sec hover:text-text',
              )}
            >
              Monthly
            </button>
            <button
              onClick={() => setBilling('annual')}
              className={clsx(
                'px-4 py-1.5 rounded text-sm font-[600] transition-colors flex items-center gap-2',
                billing === 'annual' ? 'bg-panel text-text' : 'text-sec hover:text-text',
              )}
            >
              Annual
              <span className="text-xs text-low bg-low/10 border border-low/20 px-1.5 py-0.5 rounded font-mono">-20%</span>
            </button>
          </div>
        </section>

        {/* Pricing cards */}
        <section className="px-6 pb-20 max-w-5xl mx-auto">
          <div ref={cardsRef} className="grid grid-cols-1 md:grid-cols-3 gap-6 reveal">
            {PLANS.map((plan) => {
              const price = getPrice(plan.monthlyPrice)
              return (
                <div
                  key={plan.id}
                  className={clsx(
                    'relative rounded-card p-8 flex flex-col',
                    plan.featured
                      ? 'bg-indigo/5 border-2 border-indigo animate-border-glow'
                      : 'bg-panel border border-hair lift-on-hover',
                  )}
                >
                  {plan.badge && (
                    <div className="absolute -top-3 left-1/2 -translate-x-1/2 bg-acc/15 border border-indigo text-acc-text px-3 py-1 rounded-full text-label-sm whitespace-nowrap">
                      {plan.badge}
                    </div>
                  )}
                  <div className="mb-6">
                    <span className={clsx('text-label-sm block mb-2', plan.featured ? 'text-acc-text' : 'text-sec')}>{plan.name}</span>
                    <div className="flex items-end gap-1 mb-2">
                      {price === null ? (
                        <span className="font-display text-4xl font-[700] text-text">Custom</span>
                      ) : (
                        <>
                          <span className="font-display text-5xl font-[700] text-text">${price}</span>
                          <span className="text-sec text-body-md mb-1.5">/mo</span>
                        </>
                      )}
                    </div>
                    {billing === 'annual' && price !== null && price > 0 && (
                      <p className="text-xs text-low">Billed ${price * 12}/year</p>
                    )}
                    <p className="text-body-md text-sec mt-2">{plan.description}</p>
                  </div>
                  <div className={clsx('border-t mb-6', plan.featured ? 'border-indigo/30' : 'border-hair')} />
                  {plan.inherits && (
                    <p className="mb-4 text-label-sm text-sec">{plan.inherits}</p>
                  )}
                  <ul className="mb-8 flex flex-col gap-3">
                    {plan.highlights.map((h) => (
                      <li key={h} className="flex items-start gap-3">
                        <Check size={16} className="mt-0.5 shrink-0 text-acc-text" aria-hidden="true" />
                        <span className="text-body-md leading-6 text-sec">{h}</span>
                      </li>
                    ))}
                  </ul>
                  <div className="flex-1" />
                  {plan.id === 'starter' ? (
                    <Button variant={plan.ctaVariant} fullWidth onClick={() => { setAuthTab('signup'); setAuthOpen(true) }}>
                      {plan.cta}
                    </Button>
                  ) : plan.id === 'pro' ? (
                    <Button variant={plan.ctaVariant} fullWidth onClick={() => { setAuthTab('signup'); setAuthOpen(true) }}>
                      {plan.cta}
                    </Button>
                  ) : (
                    <Link href="/contact"><Button variant={plan.ctaVariant} fullWidth>{plan.cta}</Button></Link>
                  )}
                  <p className="text-center text-xs text-sec mt-3">
                    {plan.id === 'starter' ? 'No credit card required' : plan.id === 'pro' ? '14-day free trial included' : 'Custom contract & SLA'}
                  </p>
                </div>
              )
            })}
          </div>
        </section>

        {/* Feature comparison table */}
        <section className="px-6 py-24 bg-surface-2 border-y border-hair">
          <div className="max-w-5xl mx-auto">
            <div className="text-center mb-12">
              <h2 className="font-display text-3xl font-[600] text-text mb-3">Full Feature Comparison</h2>
              <p className="text-body-md text-sec">See exactly what&apos;s included at each tier</p>
            </div>
            <div ref={tableRef} className="reveal">
              {/* Header */}
              <div className="grid grid-cols-4 gap-4 mb-4 sticky top-[73px] bg-surface-2 py-4 z-10">
                <div />
                {PLANS.map((plan) => (
                  <div key={plan.id} className={clsx('text-center py-2 rounded-lg', plan.featured && 'bg-indigo/5 border border-indigo/20')}>
                    <span className={clsx('text-label-sm block', plan.featured ? 'text-acc-text' : 'text-text')}>{plan.name}</span>
                  </div>
                ))}
              </div>

              {COMPARISON_ROWS.map((group) => (
                <div key={group.category} className="mb-6">
                  <div className="text-label-sm text-sec bg-panel border border-hair rounded-t-lg px-4 py-2.5">
                    {group.category}
                  </div>
                  {group.rows.map((row, i) => (
                    <div
                      key={row.feature}
                      className={clsx(
                        'grid grid-cols-4 gap-4 px-4 py-3 border-x border-b border-hair',
                        i === group.rows.length - 1 && 'rounded-b-lg',
                        i % 2 === 0 ? 'bg-surface-2' : 'bg-panel/40',
                      )}
                    >
                      <span className="text-body-md text-sec">{row.feature}</span>
                      <div className="text-center"><CellValue value={row.starter} /></div>
                      <div className="text-center"><CellValue value={row.pro} /></div>
                      <div className="text-center"><CellValue value={row.enterprise} /></div>
                    </div>
                  ))}
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* FAQ */}
        <section className="px-6 py-24 max-w-3xl mx-auto">
          <div className="text-center mb-12">
            <h2 className="font-display text-3xl font-[600] text-text mb-3">Frequently Asked Questions</h2>
            <p className="text-body-md text-sec">Everything you need to know about Truent&apos;s plans</p>
          </div>
          <div ref={faqRef} className="space-y-3 reveal">
            {FAQS.map((faq, i) => (
              <div key={i} className="bg-panel border border-hair rounded-card overflow-hidden">
                <button
                  onClick={() => setOpenFaq(openFaq === i ? null : i)}
                  className="w-full flex items-center justify-between px-6 py-5 text-left hover:bg-panel transition-colors"
                >
                  <span className="font-display text-base font-[600] text-text pr-4">{faq.q}</span>
                  <ChevronDown
                    size={18}
                    className={clsx('text-sec flex-shrink-0 transition-transform duration-200', openFaq === i && 'rotate-180')}
                  />
                </button>
                {openFaq === i && (
                  <div className="px-6 pb-5">
                    <p className="text-body-md text-sec leading-6">{faq.a}</p>
                  </div>
                )}
              </div>
            ))}
          </div>
        </section>

        {/* Bottom CTA */}
        <section className="px-6 pb-24">
          <div className="max-w-3xl mx-auto text-center bg-indigo/5 border border-indigo/20 rounded-card p-12">
            <ShieldCheck size={32} className="text-acc-text mx-auto mb-4" />
            <h2 className="font-display text-3xl font-[600] text-text mb-4">Still have questions?</h2>
            <p className="text-body-lg text-sec mb-8">
              Talk to our security team and we&apos;ll help you find the right plan for your protocol.
            </p>
            <div className="flex flex-col sm:flex-row gap-4 justify-center">
              <Button variant="primary" size="lg" icon={<ArrowRight size={16} />} iconPosition="right" onClick={() => { setAuthTab('signup'); setAuthOpen(true) }}>
                Start for Free
              </Button>
              <Link href="/contact">
                <Button variant="secondary" size="lg">Contact Sales</Button>
              </Link>
            </div>
          </div>
        </section>
      </main>

      <AuthModal isOpen={authOpen} onClose={() => setAuthOpen(false)} defaultTab={authTab} />
      <MarketingFooter />
    </div>
  )
}
