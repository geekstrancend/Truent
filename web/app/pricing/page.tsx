'use client'

import { useState } from 'react'
import Link from 'next/link'
import { MarketingNav } from '@/components/layout/MarketingNav'
import { PageShell } from '@/components/layout/PageShell'
import { SlimFooter } from '@/components/layout/SlimFooter'
import { AsciiLogo } from '@/components/ui/AsciiLogo'
import { AuthModal } from '@/components/ui/AuthModal'

// ─────────────────────────────────────────────────────────────────────────────
// Content
// ─────────────────────────────────────────────────────────────────────────────

type Cell = boolean | string

const CHECK = '✓'
const CROSS = '—'
const YES = '#34d399'
const VAL = '#96a19a'
const NO = '#3d453f'

const cell = (v: Cell) => ({
  text: v === true ? CHECK : v === false ? CROSS : v,
  color: v === true ? YES : v === false ? NO : VAL,
})

const row = (feature: string, starter: Cell, pro: Cell, enterprise: Cell) => ({
  feature,
  starter: cell(starter),
  pro: cell(pro),
  enterprise: cell(enterprise),
})

const comparison = [
  {
    category: 'Scans',
    rows: [
      row('Scans per month', '5', 'Unlimited', 'Unlimited'),
      row('Scan depth', 'Standard', 'Deep', 'Deep + Custom'),
      row('Parallel scan jobs', '1', '10', 'Unlimited'),
    ],
  },
  {
    category: 'Security engine',
    rows: [
      row('Invariant library access', 'Public only', 'Full', 'Full + Custom'),
      row('Symbolic execution', false, true, true),
      row('AI Co-Auditor', false, true, true),
      row('Self-improving engine', false, true, true),
    ],
  },
  {
    category: 'Integrations',
    rows: [
      row('GitHub / GitLab CI/CD', false, true, true),
      row('Slack / Discord alerts', false, true, true),
      row('REST API access', false, true, true),
      row('SSO / SAML', false, false, true),
    ],
  },
  {
    category: 'Reports',
    rows: [
      row('PDF report export', true, true, true),
      row('Shareable report links', true, true, true),
      row('White-label reports', false, false, true),
      row('Formal verification proofs', false, true, true),
    ],
  },
  {
    category: 'Support',
    rows: [
      row('Community support', true, true, true),
      row('Priority email support', false, true, true),
      row('24/7 security advisor', false, false, true),
      row('Dedicated onboarding', false, false, true),
    ],
  },
  {
    category: 'Deployment',
    rows: [
      row('Cloud hosted', true, true, true),
      row('On-premises deployment', false, false, true),
      row('Private invariant repository', false, false, true),
      row('SLA guarantee', false, false, true),
    ],
  },
]

const faqs = [
  { q: 'What counts as a "scan"?', a: 'A scan is one analysis run on a set of contracts. You can include multiple Solidity, Rust, or Move files in a single scan. Truent runs all 50+ invariant checks plus symbolic execution in one pass.' },
  { q: 'Can I try Professional features before paying?', a: 'Yes — the Professional plan includes a 14-day free trial with full access to the AI Co-Auditor, unlimited scans, and CI/CD integrations. No credit card required to start.' },
  { q: 'Which chains are supported?', a: 'Truent currently supports EVM-compatible chains (Ethereum, Arbitrum, Base, Polygon, Optimism, Avalanche, BNB Chain), Solana (Anchor programs), and Move-based chains (Aptos, Sui). More chains are added regularly.' },
  { q: 'How does annual billing work?', a: 'Annual billing is charged once per year at a 20% discount off the monthly rate. You receive one invoice per year and can cancel before renewal for a prorated refund.' },
  { q: 'What is the Enterprise SLA?', a: 'Enterprise customers receive a 99.9% uptime SLA for the scanning API and a maximum 4-hour response time for P1 security incidents. Custom SLAs are available on request.' },
  { q: 'Can I use Truent for client audit work?', a: 'Yes. The Professional plan allows you to generate reports for up to 10 separate client protocols per month. Enterprise customers have unlimited client workspaces and white-label reporting.' },
]

// ─────────────────────────────────────────────────────────────────────────────
// Page
// ─────────────────────────────────────────────────────────────────────────────

export default function PricingPage() {
  const [annual, setAnnual] = useState(false)
  const [openFaq, setOpenFaq] = useState<number | null>(null)
  const [authOpen, setAuthOpen] = useState(false)

  const proPrice = annual ? 399 : 499

  const plans = [
    {
      num: '1',
      name: 'Starter',
      price: '$0',
      per: ' / month',
      accent: '#8fdcb2',
      cta: 'Get started free',
      blurb: 'For indie auditors and early-stage projects finding their footing. Five scans a month, the public invariant library, and shareable reports.',
      caption: 'For solo builders and first audits',
      features: ['5 scans per month', 'Public invariant library', 'PDF export & shareable links', 'Community support'],
      glow: 'radial-gradient(120% 120% at 78% 22%, rgba(143,220,178,0.30), rgba(20,44,32,0.55) 55%, rgba(8,14,11,0.9) 100%)',
    },
    {
      num: '2',
      name: 'Professional',
      price: `$${proPrice}`,
      per: ' / month',
      accent: '#34d399',
      cta: 'Start free trial',
      featured: true,
      blurb: 'For teams shipping to production. Unlimited scans, the full engine with symbolic execution, and the AI Co-Auditor held to engine-verified findings.',
      caption: 'For production protocols and audit shops',
      features: ['Unlimited scans, 10 in parallel', 'Full library + symbolic execution', 'AI Co-Auditor, engine-verified', 'CI/CD gate, REST API, Slack alerts', 'Formal verification proofs'],
      glow: 'radial-gradient(120% 120% at 74% 26%, rgba(253,224,71,0.28), rgba(52,211,153,0.42) 40%, rgba(14,60,40,0.85) 78%, rgba(8,14,11,0.95) 100%)',
    },
    {
      num: '3',
      name: 'Enterprise',
      price: 'Custom',
      per: '',
      accent: '#a3e635',
      cta: 'Contact sales',
      href: '/contact',
      blurb: 'For organisations securing billions on-chain. Private invariant repositories, on-premises deployment, SSO, and a 24/7 security advisor.',
      caption: 'For large-scale, regulated deployments',
      features: ['Custom invariants & private repo', 'On-premises deployment', 'SSO / SAML, white-label reports', '24/7 security advisor & SLA'],
      glow: 'radial-gradient(120% 120% at 80% 20%, rgba(163,230,53,0.24), rgba(16,80,52,0.6) 50%, rgba(8,14,11,0.95) 100%)',
    },
  ]

  const toggleBtn = (active: boolean) =>
    `inline-flex items-center rounded-full px-[18px] py-2.5 text-[13px] font-medium transition-colors ${
      active ? 'bg-acc-text/[0.12] text-text' : 'text-[#8a948d] hover:text-text'
    }`

  return (
    <PageShell>
      <MarketingNav />

      {/* ─── Hero ─── */}
      <header className="relative overflow-hidden px-6 pb-12 pt-[90px] text-center">
        <div className="pointer-events-none absolute left-1/2 top-14 -translate-x-1/2 opacity-[0.12]">
          <AsciiLogo className="text-[clamp(4px,0.95vw,11px)] !leading-[1.08]" />
        </div>
        <span className="relative inline-flex items-center gap-2 rounded-full border border-acc-text/20 bg-acc-text/[0.07] px-4 py-[7px] font-mono text-[11px] uppercase tracking-[0.18em] text-[#8fdcb2]">
          <span className="inline-block h-[5px] w-[5px] rounded-full bg-acc-text" />
          Simple, transparent pricing
        </span>
        <h1 className="m-0 mt-[26px] text-[clamp(40px,6vw,64px)] font-normal tracking-[-0.03em] text-[#f2f6f2]">
          Plans for{' '}
          <span
            className="bg-clip-text text-transparent"
            style={{ backgroundImage: 'linear-gradient(100deg,#d7ffe9,#34d399)' }}
          >
            every stage
          </span>
        </h1>
        <p className="mx-auto mt-[18px] max-w-[420px] text-[15px] leading-[1.65] text-sec">
          Start free. Scale when you&apos;re ready. No hidden fees.
        </p>
        <div className="mt-8 inline-flex items-center gap-1 rounded-full border border-white/[0.08] bg-white/[0.03] p-1">
          <button onClick={() => setAnnual(false)} className={toggleBtn(!annual)}>
            Monthly
          </button>
          <button onClick={() => setAnnual(true)} className={toggleBtn(annual)}>
            Annual
            <span className="ml-2 rounded-[5px] border border-acc-text/25 bg-acc-text/10 px-1.5 py-0.5 font-mono text-[10px] text-acc-text">
              -20%
            </span>
          </button>
        </div>
      </header>

      {/* ─── Plan cards ─── */}
      <section className="mx-auto max-w-[1060px] px-6 pb-[90px] pt-10">
        <div className="mb-11 text-center">
          <h2 className="m-0 text-[clamp(26px,3.6vw,38px)] font-normal tracking-[-0.025em] text-[#f2f6f2]">
            Choose one of our plans
          </h2>
          <p className="m-0 mt-2.5 text-[13.5px] text-[#8a948d]">
            What stage are you at, and how much do you need to prove?
          </p>
        </div>
        <div className="flex flex-col gap-4">
          {plans.map((plan) => (
            <div
              key={plan.name}
              className={`relative grid min-h-[250px] overflow-hidden rounded-[20px] md:grid-cols-[1fr_1.05fr] ${
                plan.featured
                  ? 'border border-acc-text/40 bg-acc-text/[0.045] shadow-[0_0_60px_rgba(52,211,153,0.08)]'
                  : 'border border-hair bg-white/[0.02]'
              }`}
            >
              <div className="relative flex flex-col p-[34px] md:px-9">
                <div
                  className="mb-auto text-[30px] font-medium tracking-[-0.02em]"
                  style={{ color: plan.accent }}
                >
                  {plan.num}
                </div>
                <div className="mt-[34px]">
                  <div className="text-[29px] font-normal tracking-[-0.025em] text-[#f2f6f2]">
                    {plan.name}
                  </div>
                  <div className="mt-2 text-[15px] text-text">
                    {plan.price}
                    <span className="text-[12.5px] text-[#5c665f]">{plan.per}</span>
                  </div>
                  <p className="m-0 mt-3 max-w-[340px] text-[12.5px] leading-[1.7] text-[#8a948d]">
                    {plan.blurb}
                  </p>
                  {plan.href ? (
                    <Link
                      href={plan.href}
                      className="mt-[22px] inline-flex items-center gap-2.5 self-start rounded-full border border-white/[0.18] px-[18px] py-2.5 font-mono text-[11.5px] font-semibold uppercase tracking-[0.1em] text-[#cfd6d1] transition-colors hover:border-acc-text/50 hover:text-text"
                    >
                      {plan.cta}
                      <span className="text-[13px] tracking-[-0.12em]">❯❯</span>
                    </Link>
                  ) : (
                    <button
                      onClick={() => setAuthOpen(true)}
                      className={`mt-[22px] inline-flex items-center gap-2.5 self-start rounded-full px-[18px] py-2.5 font-mono text-[11.5px] font-semibold uppercase tracking-[0.1em] transition-colors ${
                        plan.featured
                          ? 'bg-[#eef2ef] text-[#0a0d0b] hover:bg-white'
                          : 'border border-white/[0.18] text-[#cfd6d1] hover:border-acc-text/50 hover:text-text'
                      }`}
                    >
                      {plan.cta}
                      <span className="text-[13px] tracking-[-0.12em]">❯❯</span>
                    </button>
                  )}
                </div>
              </div>

              <div className="relative m-3 flex overflow-hidden rounded-[14px] md:ml-0">
                <div className="absolute inset-0" style={{ background: plan.glow }} />
                <div className="relative flex min-h-0 flex-1 flex-col p-[26px] md:px-7">
                  <div className="flex items-center gap-2 font-mono text-[10.5px] tracking-[0.12em] text-[#e6efe9]">
                    <span style={{ color: plan.accent }}>✧</span>
                    {plan.caption}
                  </div>
                  <div className="mt-auto flex flex-col gap-[9px] pt-8">
                    {plan.features.map((f) => (
                      <div key={f} className="flex gap-2.5 text-[12.5px] leading-[1.5] text-[#d7e2da]">
                        <span className="text-[#86efac]">✓</span>
                        {f}
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* ─── Comparison ─── */}
      <section className="border-y border-white/[0.06] bg-white/[0.012] px-6 py-[90px]">
        <div className="mx-auto max-w-[900px]">
          <div className="mb-12 text-center">
            <h2 className="m-0 text-[clamp(26px,3.5vw,36px)] font-normal tracking-[-0.02em] text-[#f2f6f2]">
              Full feature comparison
            </h2>
            <p className="m-0 mt-3 text-[13px] text-sec">See exactly what&apos;s included at each tier</p>
          </div>

          {/* Four columns cannot compress below ~620px, so the table scrolls
              inside its own container rather than the page. */}
          <div className="overflow-x-auto">
            <div className="min-w-[620px]">
              <div className="grid grid-cols-[1.6fr_1fr_1fr_1fr] px-4 pb-3.5">
                <div />
                <div className="text-center font-mono text-[10.5px] uppercase tracking-[0.14em] text-sec">Starter</div>
                <div className="text-center font-mono text-[10.5px] uppercase tracking-[0.14em] text-acc-text">Professional</div>
                <div className="text-center font-mono text-[10.5px] uppercase tracking-[0.14em] text-sec">Enterprise</div>
              </div>
              {comparison.map((group) => (
                <div key={group.category} className="mb-[18px] overflow-hidden rounded-[14px] border border-hair">
                  <div className="border-b border-white/[0.06] bg-acc-text/[0.05] px-4 py-[11px] font-mono text-[10.5px] uppercase tracking-[0.16em] text-[#8fdcb2]">
                    {group.category}
                  </div>
                  {group.rows.map((r) => (
                    <div
                      key={r.feature}
                      className="grid grid-cols-[1.6fr_1fr_1fr_1fr] items-center border-b border-white/[0.04] px-4 py-[11px] last:border-b-0"
                    >
                      <span className="text-[13px] text-sec">{r.feature}</span>
                      {[r.starter, r.pro, r.enterprise].map((c, i) => (
                        <span key={i} className="text-center text-[12.5px]" style={{ color: c.color }}>
                          {c.text}
                        </span>
                      ))}
                    </div>
                  ))}
                </div>
              ))}
            </div>
          </div>
        </div>
      </section>

      {/* ─── FAQ ─── */}
      <section className="mx-auto max-w-[720px] px-6 py-[90px]">
        <div className="mb-11 text-center">
          <h2 className="m-0 text-[clamp(26px,3.5vw,36px)] font-normal tracking-[-0.02em] text-[#f2f6f2]">
            Frequently asked questions
          </h2>
          <p className="m-0 mt-3 text-[13px] text-sec">
            Everything you need to know about Truent&apos;s plans
          </p>
        </div>
        <div className="flex flex-col gap-2.5">
          {faqs.map((faq, i) => {
            const open = openFaq === i
            return (
              <div key={faq.q} className="overflow-hidden rounded-[14px] border border-hair bg-white/[0.02]">
                <button
                  onClick={() => setOpenFaq(open ? null : i)}
                  aria-expanded={open}
                  className="flex w-full items-center justify-between gap-4 px-[22px] py-[18px] text-left"
                >
                  <span className="text-[14.5px] font-medium text-text">{faq.q}</span>
                  <span className="flex-shrink-0 text-[13px] text-[#748078]">{open ? '▲' : '▼'}</span>
                </button>
                {open && (
                  <p className="m-0 px-[22px] pb-[18px] text-[13px] leading-[1.7] text-sec">{faq.a}</p>
                )}
              </div>
            )
          })}
        </div>
      </section>

      {/* ─── CTA ─── */}
      <section className="mx-auto max-w-[760px] px-6 pb-[100px]">
        <div className="relative overflow-hidden rounded-[22px] border border-acc-text/[0.22] bg-acc-text/[0.04] px-10 py-14 text-center">
          <div
            className="pointer-events-none absolute -top-[140px] left-1/2 h-[340px] w-[520px] -translate-x-1/2"
            style={{ background: 'radial-gradient(closest-side,rgba(52,211,153,0.14),transparent)' }}
          />
          <div className="relative">
            <div className="mb-3.5 text-[28px]">⬡</div>
            <h2 className="m-0 text-[clamp(24px,3.5vw,34px)] font-normal tracking-[-0.02em] text-[#f2f6f2]">
              Still have questions?
            </h2>
            <p className="mx-auto mt-3.5 max-w-[420px] text-[13.5px] leading-[1.7] text-sec">
              Talk to our security team and we&apos;ll help you find the right plan for your protocol.
            </p>
            <div className="mt-[30px] flex flex-wrap justify-center gap-3">
              <button
                onClick={() => setAuthOpen(true)}
                className="inline-flex items-center gap-2.5 rounded-full bg-[#eef2ef] py-1.5 pl-5 pr-1.5 text-[13px] font-semibold text-[#0a0d0b] transition-colors hover:bg-white"
              >
                Start for free
                <span className="flex h-[30px] w-[30px] items-center justify-center rounded-full bg-acc-text text-[14px] text-on-acc">
                  →
                </span>
              </button>
              <Link
                href="/contact"
                className="inline-flex items-center rounded-full border border-white/[0.16] px-[22px] py-3 text-[13px] font-medium text-[#cfd6d1] transition-colors hover:border-acc-text/50 hover:text-text"
              >
                Contact sales
              </Link>
            </div>
          </div>
        </div>
      </section>

      <SlimFooter omit={['Pricing']} />
      <AuthModal isOpen={authOpen} onClose={() => setAuthOpen(false)} defaultTab="signup" />
    </PageShell>
  )
}
