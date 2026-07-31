'use client'

import { useState } from 'react'
import Link from 'next/link'
import { ShieldCheck, BookOpen, Brain, RefreshCw, ArrowRight, Check, Zap, GitBranch, Eye, FileText } from 'lucide-react'
import { useReveal } from '@/components/hooks/useReveal'
import { AnimatedCounter } from '@/components/ui/AnimatedCounter'
import { AsciiLogo } from '@/components/ui/AsciiLogo'
import { ParticleHero } from '@/components/ui/ParticleHero'
import { MarketingNav } from '@/components/layout/MarketingNav'
import { MarketingFooter } from '@/components/layout/MarketingFooter'
import { Button } from '@/components/ui/Button'
import { Terminal } from '@/components/ui/Terminal'
import { SeverityBadge } from '@/components/ui/SeverityBadge'
import { AuthModal } from '@/components/ui/AuthModal'
import { SampleReportModal } from '@/components/ui/SampleReportModal'

export default function HomePage() {
  const [authOpen, setAuthOpen] = useState(false)
  const [authTab, setAuthTab] = useState<'signin' | 'signup'>('signin')
  const [sampleReportOpen, setSampleReportOpen] = useState(false)

  const featuresRef = useReveal()
  const stepsRef = useReveal()
  const exploitsRef = useReveal()
  const reportsLeftRef = useReveal()
  const reportsRightRef = useReveal()
  const starterRef = useReveal()
  const proRef = useReveal()
  const enterpriseRef = useReveal()
  const ctaRef = useReveal()

  return (
    <div className="min-h-screen bg-bg">
      <MarketingNav />

      <ParticleHero
        wordmark="TRUENT"
        eyebrow="Smart contract security intelligence"
        headline="Don't get Hacked!"
        subline={<>Findings a fuzzer <span className="text-acc-text">proved by execution</span> — not a model&apos;s opinion. <span className="text-acc-text">71 detectors</span> across EVM, Solana, Move and Soroban.</>}
        actions={
          <>
            <Button variant="primary" size="lg" icon={<ArrowRight size={18} />} iconPosition="right" onClick={() => { setAuthTab('signup'); setAuthOpen(true) }}>
              Start free trial
            </Button>
            <Button variant="secondary" size="lg" onClick={() => setSampleReportOpen(true)}>
              View a sample report
            </Button>
          </>
        }
      />

      {/* ── Content layer: scrolls over the pinned hero ── */}
      <div className="relative z-[1] bg-bg">

      {/* ─── Chain Ticker ─── */}
      <div className="border-y border-hair bg-surface-2/70 py-3 overflow-hidden w-full mt-8">
        <div className="flex animate-marquee whitespace-nowrap">
          {['Ethereum', 'Solana', 'Arbitrum', 'Base', 'Polygon', 'Optimism', 'Aptos', 'Sui', 'Avalanche', 'BNB Chain', 'Ethereum', 'Solana', 'Arbitrum', 'Base', 'Polygon', 'Optimism', 'Aptos', 'Sui', 'Avalanche', 'BNB Chain'].map((chain, i) => (
            <span key={i} className="text-label-sm text-sec flex-shrink-0 px-8">
              ◆ {chain}
            </span>
          ))}
        </div>
      </div>

      {/* ─── Stats ───
          Every figure here is checkable against the tool itself
          (`truent registry list`, `truent scan --chain <c>`) rather than
          being a marketing number. Keep it that way. */}
      <section className="mx-auto max-w-site px-7 py-20">
        <div className="grid grid-cols-2 gap-8 md:grid-cols-4">
          <div className="text-center">
            <div className="text-display-md font-[600] text-acc-text mb-2">
              <AnimatedCounter value={71} />
            </div>
            <p className="text-body-sm text-sec">Detectors shipping</p>
          </div>
          <div className="text-center">
            <div className="text-display-md font-[600] text-acc-text mb-2">
              <AnimatedCounter value={4} />
            </div>
            <p className="text-body-sm text-sec">Chains, one engine</p>
          </div>
          <div className="text-center">
            <div className="text-display-md font-[600] text-acc-text mb-2">
              <AnimatedCounter value={21} />
            </div>
            <p className="text-body-sm text-sec">Exploits reproduced</p>
          </div>
          <div className="text-center">
            <div className="text-display-md font-[600] text-acc-text mb-2">
              $<AnimatedCounter value={1.76} decimals={2} />B
            </div>
            <p className="text-body-sm text-sec">Losses in the registry</p>
          </div>
        </div>
        <p className="mt-10 text-center font-mono text-xs text-sec">
          Verify any of these yourself:{' '}
          <span className="text-acc-text">truent registry list</span>
        </p>
      </section>

      {/* ─── Live scan: the CLI doing the work ─── */}
      <section className="mx-auto max-w-site px-7 py-20">
        <div className="grid grid-cols-1 gap-10 lg:grid-cols-[0.85fr_1.15fr] lg:items-center">
          <div>
            <span className="font-mono text-xs uppercase tracking-[0.2em] text-acc-text">See it run</span>
            <h2 className="mt-4 font-display text-4xl font-[600] leading-tight tracking-[-0.02em] text-text">
              One command. Every invariant.
            </h2>
            <p className="mt-4 text-body-md leading-relaxed text-sec">
              Point Truent at a contract and it deploys, drives adversarial call sequences,
              and checks every invariant after every call — then shrinks any violation to a
              minimal proof-of-concept you can run.
            </p>
            <div className="mt-7 flex flex-col gap-3 sm:flex-row">
              <Link href="/docs/cli"><Button variant="secondary" size="lg">Read the CLI reference</Button></Link>
            </div>
          </div>
          <div className="rounded-card border border-hair bg-panel p-2">
            <Terminal
              showBanner={true}
              output={[
                { prefix: 'INFO', text: '71 detectors loaded · chain=evm · dynamic engine armed', type: 'info' },
                { prefix: 'SCAN', text: 'Deployed Vault.sol to in-memory EVM, driving call sequences...', type: 'scan' },
                { text: '' },
                { prefix: 'CRITICAL', text: 'Invariant violated: sum(balanceOf) == totalSupply()', type: 'critical' },
                { text: '          sum(balanceOf) = 2000  !=  totalSupply() = 1000' },
                { text: '' },
                { text: '          Minimal reproduction (2 calls):' },
                { text: '            1. airdrop(0x0101..., 1000)   [caller=0x1111...]' },
                { text: '            2. airdrop(0x0101..., 1000)   [caller=0x1111...]' },
                { text: '' },
                { prefix: 'DONE', text: 'Reproduced and shrunk. Exit 1 — deploy blocked.', type: 'done' },
              ]}
            />
          </div>
        </div>
      </section>

      {/* ─── Features Bento ─── */}
      <section id="features" className="mx-auto max-w-site px-7 pb-24 pt-16">
        <div className="text-center mb-14">
          <div className="inline-flex items-center gap-2 px-4 py-2 rounded-pill border border-hair bg-surface-2 mb-4">
            <span className="text-label-sm text-acc-text">CAPABILITIES</span>
          </div>
          <h2 className="font-display text-4xl font-[600] text-text leading-[48px]">
            An engine that can prove it.
          </h2>
          <p className="mx-auto mt-4 max-w-narrow text-body-md leading-relaxed text-sec">
            Static analysis finds patterns. Language models find suspicions. Truent runs the
            code and settles it.
          </p>
        </div>
        <div ref={featuresRef} className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 reveal">
          {/* Invariant Library — spans 2 cols */}
          <div className="lg:col-span-2 bg-panel p-8 rounded-card border border-hair lift-on-hover relative overflow-hidden group">
            <div className="absolute -right-8 -bottom-8 opacity-[0.04] group-hover:opacity-[0.08] transition-opacity duration-500">
              <AsciiLogo />
            </div>
            <div className="w-12 h-12 rounded-lg bg-indigo/10 border border-indigo/20 flex items-center justify-center mb-5 relative z-10">
              <BookOpen size={22} className="text-acc-text" />
            </div>
            <h3 className="font-display text-xl font-[600] text-text mb-3 relative z-10">Every finding is proved, not guessed</h3>
            <p className="text-body-md text-sec leading-6 mb-4 max-w-xl relative z-10">
              A prompt-only auditor stops at &ldquo;this looks like a bug.&rdquo; Truent deploys your
              contract in a real VM, drives adversarial sequences against it, and only reports a
              finding once it has made the bug actually fire — then shrinks the trace to the
              shortest sequence that reproduces it.
            </p>
            <p className="text-body-md text-sec leading-6 mb-6 max-w-xl relative z-10">
              You get a runnable proof-of-concept, so nothing lands in your report that
              can&apos;t be reproduced.
            </p>
            <Link href="/library" className="inline-flex items-center gap-1.5 text-acc-text text-sm font-[600] hover:gap-3 transition-all duration-200 relative z-10">
              Browse the Library <ArrowRight size={14} />
            </Link>
          </div>
          {/* AI Co-Auditor */}
          <div className="bg-panel p-8 rounded-card border border-hair lift-on-hover">
            <div className="w-12 h-12 rounded-lg bg-indigo/10 border border-indigo/20 flex items-center justify-center mb-5">
              <Brain size={22} className="text-acc-text" />
            </div>
            <h3 className="font-display text-xl font-[600] text-text mb-3">AI, held to account</h3>
            <p className="text-body-md text-sec leading-6">
              The model proposes candidate bugs; the engine tries to fire each one. Anything it
              can&apos;t reproduce is labelled as a lead, never as a finding.
            </p>
          </div>
          {/* Self-Improving */}
          <div className="bg-panel p-8 rounded-card border border-hair lift-on-hover">
            <div className="w-12 h-12 rounded-lg bg-indigo/10 border border-indigo/20 flex items-center justify-center mb-5">
              <RefreshCw size={22} className="text-acc-text" />
            </div>
            <h3 className="font-display text-xl font-[600] text-text mb-3">Self-Improving Engine</h3>
            <p className="text-body-md text-sec leading-6">
              Learns from every new exploit in the wild, automatically generating new detection modules within 24 hours.
            </p>
          </div>
          {/* CI/CD */}
          <div className="bg-panel p-8 rounded-card border border-hair lift-on-hover">
            <div className="w-12 h-12 rounded-lg bg-indigo/10 border border-indigo/20 flex items-center justify-center mb-5">
              <GitBranch size={22} className="text-acc-text" />
            </div>
            <h3 className="font-display text-xl font-[600] text-text mb-3">CI/CD Integration</h3>
            <p className="text-body-md text-sec leading-6">
              Native GitHub Actions and GitLab pipeline support. Block deploys on critical findings automatically.
            </p>
          </div>
          {/* Symbolic Execution */}
          <div className="bg-indigo/5 border border-indigo/20 p-8 rounded-card lift-on-hover">
            <div className="w-12 h-12 rounded-lg bg-indigo/10 border border-indigo/20 flex items-center justify-center mb-5">
              <Zap size={22} className="text-acc-text" />
            </div>
            <h3 className="font-display text-xl font-[600] text-text mb-3">Symbolic Execution</h3>
            <p className="text-body-md text-sec leading-6">
              Formal verification explores every execution path. Zero false negatives on all critical code paths.
            </p>
          </div>
        </div>
      </section>

      {/* ─── How It Works ─── */}
      <section className="px-6 py-24 bg-panelest border-y border-hair">
        <div className="max-w-site mx-auto">
          <div className="text-center mb-16">
            <div className="inline-flex items-center gap-2 px-4 py-2 rounded-pill border border-hair bg-surface-2 mb-4">
              <span className="text-label-sm text-acc-text">HOW IT WORKS</span>
            </div>
            <h2 className="font-display text-4xl font-[600] text-text leading-[48px]">
              From code to coverage in minutes
            </h2>
          </div>
          <div ref={stepsRef} className="grid grid-cols-1 md:grid-cols-3 gap-8 reveal">
            {[
              { step: '01', icon: <GitBranch size={22} className="text-acc-text" />, title: 'Connect Your Repository', description: 'Link GitHub, GitLab, or upload contracts directly. Supports Solidity, Rust (Anchor), and Move languages.' },
              { step: '02', icon: <Eye size={22} className="text-acc-text" />, title: 'Deep Scan & Analysis', description: '50+ invariant checks run alongside symbolic execution and full data-flow analysis on every function.' },
              { step: '03', icon: <FileText size={22} className="text-acc-text" />, title: 'Actionable Reports', description: 'Get prioritized findings with code-level recommendations, formal proofs, and one-click remediation paths.' },
            ].map((item, i) => (
              <div key={i} className="bg-panel border border-hair rounded-card p-8">
                <div className="flex items-start gap-4 mb-5">
                  <span className="font-display text-5xl font-[700] text-sec-variant leading-none select-none">{item.step}</span>
                  <div className="w-10 h-10 rounded-lg bg-indigo/10 border border-indigo/20 flex items-center justify-center flex-shrink-0 mt-1">{item.icon}</div>
                </div>
                <h3 className="font-display text-lg font-[600] text-text mb-3">{item.title}</h3>
                <p className="text-body-md text-sec leading-6">{item.description}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* ─── Real Exploits (Social Proof) ─── */}
      <section className="mx-auto max-w-site px-7 py-24">
        <div className="text-center mb-12">
          <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-critical/10 border border-critical/20 mb-4">
            <span className="text-label-sm text-critical">BATTLE-TESTED AGAINST REAL EXPLOITS</span>
          </div>
          <h2 className="font-display text-4xl font-[600] text-text leading-[48px] mb-4">
            We study every major hack so you don&apos;t have to
          </h2>
          <p className="text-body-lg text-sec max-w-2xl mx-auto">
            Every invariant maps directly to a real-world exploit pattern. Truent would have flagged these before deployment.
          </p>
        </div>
        <div ref={exploitsRef} className="grid grid-cols-1 md:grid-cols-3 gap-6 reveal">
          {[
            { protocol: 'Euler Finance', amount: '$197M', year: '2023', type: 'Flash Loan + Missing Health Check', invariant: 'evm_missing_post_state_health_check' },
            { protocol: 'Nomad Bridge', amount: '$190M', year: '2022', type: 'Merkle Root Zero Initialization', invariant: 'evm_merkle_root_zero_default' },
            { protocol: 'KelpDAO', amount: '$292M', year: '2024', type: 'DVN Single Point of Failure', invariant: 'evm_dvn_single_point_failure' },
          ].map((exploit, i) => (
            <div key={i} className="bg-panel border border-hair rounded-card p-8 lift-on-hover relative overflow-hidden">
              <div className="absolute top-0 left-0 right-0 h-0.5 bg-gradient-to-r from-critical via-critical/40 to-transparent" />
              <div className="flex items-start justify-between mb-4">
                <div>
                  <p className="text-label-sm text-sec mb-1">{exploit.year} EXPLOIT</p>
                  <h3 className="font-display text-xl font-[600] text-text">{exploit.protocol}</h3>
                </div>
                <span className="font-display text-2xl font-[700] text-critical">{exploit.amount}</span>
              </div>
              <p className="text-body-md text-sec mb-5">{exploit.type}</p>
              <div className="flex flex-wrap items-center gap-2 mb-4">
                <SeverityBadge level="critical" />
                <code className="text-xs text-text-variant bg-panel border border-hair px-2 py-0.5 rounded font-mono">{exploit.invariant}</code>
              </div>
              <div className="flex items-center gap-1.5 text-low text-xs font-[600]">
                <Check size={13} />
                <span>Truent detects this pattern</span>
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* ─── Professional Reports ─── */}
      <section className="px-6 py-24 bg-panelest border-y border-hair">
        <div className="max-w-site mx-auto">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-16 items-center">
            <div ref={reportsLeftRef} className="reveal">
              <div className="inline-flex items-center gap-2 px-4 py-2 rounded-pill border border-hair bg-surface-2 mb-6">
                <span className="text-label-sm text-acc-text">AUDIT REPORTS</span>
              </div>
              <h2 className="font-display text-4xl font-[600] text-text mb-5 leading-[48px]">
                Professional Grade Reports
              </h2>
              <p className="text-body-lg text-sec mb-8">
                Generate executive-ready summaries with granular technical deep-dives. Integrated with GitHub and GitLab CI/CD pipelines out of the box.
              </p>
              <div className="space-y-4 mb-8">
                {['Gas Optimization Insights', 'Formal Verification Proofs', 'Automated Remediation Advice', 'One-Click PDF Export'].map((item) => (
                  <div key={item} className="flex items-center gap-3">
                    <Check size={16} className="text-low flex-shrink-0" />
                    <span className="text-body-md text-text-variant">{item}</span>
                  </div>
                ))}
              </div>
              <Button variant="secondary" onClick={() => setSampleReportOpen(true)}>
                View Sample Report
              </Button>
            </div>
            <div ref={reportsRightRef} className="reveal">
              <div className="bg-panel border border-hair rounded-card p-6">
                <div className="flex justify-between items-start mb-6">
                  <div>
                    <span className="text-label-sm text-sec block mb-1">AUDIT REPORT</span>
                    <h3 className="font-display text-xl font-[600] text-text">Circle-Pay BCH</h3>
                    <p className="text-body-md text-sec mt-1">Jun 6, 2026 · EVM · v2.1.0</p>
                  </div>
                  <span className="text-xs text-low bg-low/10 border border-low/20 px-2 py-1 rounded font-mono">COMPLETE</span>
                </div>
                <div className="grid grid-cols-4 gap-px bg-hair rounded-lg overflow-hidden mb-6">
                  {[
                    { label: 'CRITICAL', count: 5, color: 'text-critical' },
                    { label: 'HIGH', count: 7, color: 'text-high' },
                    { label: 'MED', count: 6, color: 'text-medium' },
                    { label: 'LOW', count: 4, color: 'text-low' },
                  ].map((item) => (
                    <div key={item.label} className="bg-panel p-3 text-center">
                      <div className={`font-display text-3xl font-[600] ${item.color} mb-1`}>{item.count}</div>
                      <div className="text-label-sm text-sec">{item.label}</div>
                    </div>
                  ))}
                </div>
                <div className="space-y-2 mb-6">
                  {[
                    { title: 'Reentrancy in withdrawAll()', sev: 'critical' as const },
                    { title: 'Unchecked external call return', sev: 'high' as const },
                    { title: 'Missing oracle staleness check', sev: 'high' as const },
                  ].map((f, i) => (
                    <div key={i} className="flex items-center justify-between py-2.5 px-3 bg-panel rounded-lg">
                      <div className="flex items-center gap-3">
                        <SeverityBadge level={f.sev} />
                        <span className="text-body-md text-text-variant">{f.title}</span>
                      </div>
                    </div>
                  ))}
                </div>
                <Button variant="secondary" fullWidth size="sm" onClick={() => setSampleReportOpen(true)}>
                  ↓ View Full Report
                </Button>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* ─── Pricing Preview ─── */}
      <section className="mx-auto max-w-site px-7 py-24">
        <div className="text-center mb-14">
          <div className="inline-flex items-center gap-2 px-4 py-2 rounded-pill border border-hair bg-surface-2 mb-4">
            <span className="text-label-sm text-acc-text">PRICING</span>
          </div>
          <h2 className="font-display text-4xl font-[600] text-text mb-4 leading-[48px]">
            Plans for Every Stage
          </h2>
          <p className="text-body-lg text-sec max-w-2xl mx-auto">
            From indie developers to enterprise security teams.
          </p>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 max-w-5xl mx-auto mb-10">
          <div ref={starterRef} className="bg-panel border border-hair rounded-card p-8 reveal lift-on-hover">
            <span className="text-label-sm text-sec block mb-3">Starter</span>
            <div className="mb-1"><span className="font-display text-5xl font-[700] text-text">$0</span><span className="text-body-md text-sec ml-2">/mo</span></div>
            <p className="text-body-md text-sec mb-6">For early-stage projects</p>
            <div className="border-t border-hair mb-6" />
            <div className="space-y-3 mb-8">
              {[{ text: '5 Scans / month', ok: true }, { text: 'Public Library Access', ok: true }, { text: 'AI Co-Auditor', ok: false }, { text: 'Priority Support', ok: false }].map((f, i) => (
                <div key={i} className={`flex items-center gap-2 ${!f.ok ? 'opacity-40' : ''}`}>
                  {f.ok ? <Check size={15} className="text-low flex-shrink-0" /> : <span className="w-[15px] text-center text-critical text-sm">✗</span>}
                  <span className={`text-body-md ${f.ok ? 'text-text-variant' : 'text-sec line-through'}`}>{f.text}</span>
                </div>
              ))}
            </div>
            <Button variant="secondary" fullWidth onClick={() => { setAuthTab('signup'); setAuthOpen(true) }}>Get Started Free</Button>
          </div>
          <div ref={proRef} className="relative bg-indigo/5 border-2 border-indigo rounded-card p-8 reveal lift-on-hover animate-border-glow">
            <div className="absolute -top-3 left-1/2 -translate-x-1/2 bg-acc/15 border border-indigo text-acc-text px-3 py-1 rounded-full text-label-sm whitespace-nowrap">MOST POPULAR</div>
            <span className="text-label-sm text-acc-text block mb-3">Professional</span>
            <div className="mb-1"><span className="font-display text-5xl font-[700] text-text">$499</span><span className="text-body-md text-sec ml-2">/mo</span></div>
            <p className="text-body-md text-sec mb-6">For production protocols</p>
            <div className="border-t border-indigo/30 mb-6" />
            <div className="space-y-3 mb-8">
              {['Unlimited Scans', 'Priority CI/CD Queues', 'Full AI Co-Auditor', 'Priority Support'].map((item) => (
                <div key={item} className="flex items-center gap-2">
                  <Check size={15} className="text-low flex-shrink-0" />
                  <span className="text-body-md text-text-variant">{item}</span>
                </div>
              ))}
            </div>
            <Button variant="primary" fullWidth onClick={() => { setAuthTab('signup'); setAuthOpen(true) }}>Get Started</Button>
          </div>
          <div ref={enterpriseRef} className="bg-panel border border-hair rounded-card p-8 reveal lift-on-hover">
            <span className="text-label-sm text-sec block mb-3">Enterprise</span>
            <div className="mb-1"><span className="font-display text-4xl font-[700] text-text">Custom</span></div>
            <p className="text-body-md text-sec mb-6">For large-scale deployments</p>
            <div className="border-t border-hair mb-6" />
            <div className="space-y-3 mb-8">
              {['Private Invariant Repo', '24/7 Security Advisor', 'On-prem deployment', 'SLA Guarantee'].map((item) => (
                <div key={item} className="flex items-center gap-2">
                  <Check size={15} className="text-low flex-shrink-0" />
                  <span className="text-body-md text-text-variant">{item}</span>
                </div>
              ))}
            </div>
            <Link href="/contact"><Button variant="secondary" fullWidth>Contact Sales</Button></Link>
          </div>
        </div>
        <div className="text-center">
          <Link href="/pricing" className="inline-flex items-center gap-1.5 text-acc-text text-sm font-[600] hover:gap-3 transition-all duration-200">
            See full pricing & comparison table <ArrowRight size={14} />
          </Link>
        </div>
      </section>

      {/* ─── Final CTA ─── */}
      <section ref={ctaRef} className="mx-auto max-w-site px-7 pb-24 pt-8 reveal">
        <div className="max-w-site mx-auto">
          <div className="relative rounded-card border border-hair bg-panel p-16 text-center overflow-hidden">
            <div className="absolute inset-0 bg-grid-pattern pointer-events-none opacity-20" />
            <div className="absolute -top-32 left-1/2 -translate-x-1/2 w-[600px] h-[400px] bg-indigo/15 rounded-full blur-3xl pointer-events-none" />
            <div className="relative">
              <div className="inline-flex items-center gap-2 px-4 py-2 rounded-pill border border-hair bg-surface-2 mb-6">
                <ShieldCheck size={14} className="text-acc-text" />
                <span className="text-label-sm text-acc-text">START SECURING TODAY</span>
              </div>
              <h2 className="font-display text-5xl font-[600] text-text mb-5 leading-[64px]">
                Ready to audit smarter?
              </h2>
              <p className="text-body-lg text-sec max-w-xl mx-auto mb-10">
                Join teams securing billions in on-chain value with Truent&apos;s invariant-driven security platform.
              </p>
              <div className="flex flex-col sm:flex-row gap-4 justify-center">
                <Button variant="primary" size="lg" icon={<ArrowRight size={18} />} iconPosition="right" onClick={() => { setAuthTab('signup'); setAuthOpen(true) }}>
                  Start Free Trial
                </Button>
                <Link href="/docs">
                  <Button variant="secondary" size="lg">Read Documentation</Button>
                </Link>
              </div>
            </div>
          </div>
        </div>
      </section>

      <AuthModal isOpen={authOpen} onClose={() => setAuthOpen(false)} defaultTab={authTab} />
      <SampleReportModal isOpen={sampleReportOpen} onClose={() => setSampleReportOpen(false)} />

        <MarketingFooter />
      </div>
    </div>
  )
}
