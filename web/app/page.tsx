'use client'

import { useEffect, useRef, useState } from 'react'
import Link from 'next/link'
import { AnimatedCounter } from '@/components/ui/AnimatedCounter'
import { TRUENT_ASCII } from '@/components/ui/AsciiLogo'
import { ParticleHero } from '@/components/ui/ParticleHero'
import { MarketingNav } from '@/components/layout/MarketingNav'
import { MarketingFooter } from '@/components/layout/MarketingFooter'
import { AuthModal } from '@/components/ui/AuthModal'
import { SampleReportModal } from '@/components/ui/SampleReportModal'

// ─────────────────────────────────────────────────────────────────────────────
// Content
// ─────────────────────────────────────────────────────────────────────────────

const steps = [
  { num: '01', icon: '⎇', title: 'Connect your repository', desc: 'Link GitHub, GitLab, or upload contracts directly. Supports Solidity, Rust (Anchor), and Move languages.' },
  { num: '02', icon: '◉', title: 'Deep scan & analysis', desc: '50+ invariant checks run alongside symbolic execution and full data-flow analysis on every function.' },
  { num: '03', icon: '▤', title: 'Actionable reports', desc: 'Get prioritized findings with code-level recommendations, formal proofs, and one-click remediation paths.' },
]

const exploits = [
  { protocol: 'Euler Finance', amount: '$197M', year: '2023', type: 'Flash Loan + Missing Health Check', invariant: 'evm_missing_post_state_health_check' },
  { protocol: 'Nomad Bridge', amount: '$190M', year: '2022', type: 'Merkle Root Zero Initialization', invariant: 'evm_merkle_root_zero_default' },
  { protocol: 'KelpDAO', amount: '$292M', year: '2024', type: 'DVN Single Point of Failure', invariant: 'evm_dvn_single_point_failure' },
]

const registry = [
  { year: '2022', count: 12, note: 'Bridges' },
  { year: '2023', count: 27, note: 'Lending' },
  { year: '2024', count: 44, note: 'Restaking' },
  { year: '2025', count: 58, note: 'Intents' },
  { year: '2026', count: 71, note: 'Shipping now', current: true },
]

const reportPerks = [
  'Gas optimization insights',
  'Formal verification proofs',
  'Automated remediation advice',
  'One-click PDF export',
]

const sevCounts = [
  { label: 'CRITICAL', count: 5, color: 'var(--critical)' },
  { label: 'HIGH', count: 7, color: 'var(--high)' },
  { label: 'MED', count: 6, color: 'var(--medium)' },
  { label: 'LOW', count: 4, color: 'var(--low)' },
]

const plans = [
  { name: 'Starter', price: '$0', per: ' / month', accent: '#8fdcb2', href: '/pricing', caption: 'For solo builders and first audits' },
  { name: 'Professional', price: '$499', per: ' / month', accent: '#34d399', href: '/pricing', caption: 'For production protocols and audit shops', featured: true },
  { name: 'Enterprise', price: 'Custom', per: '', accent: '#a3e635', href: '/contact', caption: 'For large-scale, regulated deployments' },
]

// Kept short: each line must fit the ~160px gutter track.
const driftLeft = [
  { text: 'truent check .', kind: 'cmd' as const },
  { text: 'sum(bal) == supply', kind: 'expr' as const },
  { text: 'REENTRANCY', kind: 'tag' as const },
  { text: '∀ call : invariant', kind: 'expr' as const },
  { text: 'truent registry', kind: 'cmd' as const },
  { text: 'ORACLE STALE', kind: 'tag' as const },
  { text: 'debt <= collateral', kind: 'expr' as const },
  { text: 'exit 1 — blocked', kind: 'expr' as const },
  { text: 'UNCHECKED RET', kind: 'tag' as const },
  { text: 'truent shrink', kind: 'cmd' as const },
]

const driftRight = [
  { text: 'CRITICAL', kind: 'tag' as const },
  { text: 'forge test --PoC', kind: 'cmd' as const },
  { text: 'updatedAt > now-HB', kind: 'expr' as const },
  { text: 'FLASH LOAN', kind: 'tag' as const },
  { text: '--chain solana', kind: 'cmd' as const },
  { text: 'shares * px / 1e18', kind: 'expr' as const },
  { text: 'MERKLE ZERO', kind: 'tag' as const },
  { text: 'nonReentrant', kind: 'expr' as const },
  { text: '--format pdf', kind: 'cmd' as const },
  { text: 'PRECISION LOSS', kind: 'tag' as const },
]

// ─────────────────────────────────────────────────────────────────────────────
// Primitives
// ─────────────────────────────────────────────────────────────────────────────

/** Fires once when the element first enters view (or is already scrolled past). */
function useInView<T extends HTMLElement>(threshold = 0.2) {
  const ref = useRef<T | null>(null)
  const [seen, setSeen] = useState(false)

  useEffect(() => {
    const el = ref.current
    if (!el) return
    // Anything already above the viewport — anchor jump, scroll restoration, a
    // fast fling — should present immediately rather than staying ghosted.
    if (el.getBoundingClientRect().bottom < 0) {
      setSeen(true)
      return
    }
    const io = new IntersectionObserver(
      ([e]) => {
        if (e.isIntersecting) {
          setSeen(true)
          io.disconnect()
        }
      },
      { threshold },
    )
    io.observe(el)
    return () => io.disconnect()
  }, [threshold])

  return { ref, seen }
}

function Eyebrow({ children, tone = 'green' }: { children: React.ReactNode; tone?: 'green' | 'red' }) {
  const tones = {
    green: 'text-[#8fdcb2] border-acc-text/20 bg-acc-text/[0.06]',
    red: 'text-[#f87171] border-[#ef4444]/25 bg-[#ef4444]/[0.07]',
  }
  return (
    <span className={`inline-block rounded-full border px-4 py-[7px] font-mono text-[11px] uppercase tracking-[0.2em] ${tones[tone]}`}>
      {children}
    </span>
  )
}

function SectionHeading({ children }: { children: React.ReactNode }) {
  return (
    <h2 className="m-0 mt-5 text-[clamp(30px,4vw,44px)] font-normal tracking-[-0.02em] text-[#f2f6f2]">
      {children}
    </h2>
  )
}

/** Pill CTA with the design's circular arrow badge. */
function PrimaryCta({
  onClick,
  children,
  className = '',
}: {
  onClick?: () => void
  children: React.ReactNode
  className?: string
}) {
  return (
    <button
      onClick={onClick}
      className={`inline-flex items-center gap-3 rounded-full bg-[#eef2ef] py-[7px] pl-[22px] pr-[7px] text-[14px] font-semibold text-[#0a0d0b] transition-all hover:-translate-y-0.5 hover:bg-white ${className}`}
    >
      {children}
      <span className="flex h-8 w-8 items-center justify-center rounded-full bg-acc-text text-[15px] text-on-acc">
        →
      </span>
    </button>
  )
}

function GhostCta({
  onClick,
  href,
  children,
}: {
  onClick?: () => void
  href?: string
  children: React.ReactNode
}) {
  const cls =
    'inline-flex items-center rounded-full border border-white/[0.16] px-6 py-3.5 text-[14px] font-medium text-[#cfd6d1] transition-colors hover:border-acc-text/50 hover:text-text'
  return href ? (
    <Link href={href} className={cls}>{children}</Link>
  ) : (
    <button onClick={onClick} className={cls}>{children}</button>
  )
}

/** Section wrapper providing the ghost → solid reveal. */
function Reveal({ children, className = '' }: { children: React.ReactNode; className?: string }) {
  const { ref, seen } = useInView<HTMLDivElement>(0.02)
  return (
    <div
      ref={ref}
      className={className}
      style={{ opacity: seen ? 1 : 0.06, transition: 'opacity 0.9s cubic-bezier(0.16,0.84,0.28,1)' }}
    >
      {children}
    </div>
  )
}

// ─────────────────────────────────────────────────────────────────────────────
// Sections
// ─────────────────────────────────────────────────────────────────────────────

/** Detectors-per-release-year bars, grown off the baseline on reveal. */
function RegistryChart() {
  const { ref, seen } = useInView<HTMLDivElement>(0.25)
  const max = 80

  return (
    <div
      ref={ref}
      className="relative mt-12 overflow-hidden rounded-[20px] border border-hair bg-white/[0.015] px-[30px] pb-[22px] pt-[26px]"
    >
      <div
        className="pointer-events-none absolute left-[12%] top-[16%] h-[260px] w-[420px]"
        style={{
          background: 'radial-gradient(closest-side,rgba(52,211,153,0.1),transparent)',
          animation: 'glowpulse 6s ease-in-out infinite',
        }}
      />
      <div className="relative mb-[26px] flex flex-wrap items-baseline justify-between gap-2">
        <span className="font-mono text-[10.5px] uppercase tracking-[0.16em] text-[#8fdcb2]">
          Detectors in the registry, by release year
        </span>
        <span className="font-mono text-[11px] text-[#8a948d]">truent registry list --count</span>
      </div>
      <div className="relative grid grid-cols-[34px_1fr] gap-3.5">
        <div className="relative h-[250px] font-mono text-[11px] text-[#8a948d]">
          {[80, 60, 40, 20, 0].map((v, i) => (
            <span key={v} className="absolute right-0" style={{ top: `calc(${i * 25}% - 5px)` }}>
              {v}
            </span>
          ))}
        </div>
        <div>
          <div className="relative h-[250px]">
            <div className="pointer-events-none absolute inset-0">
              {[0, 25, 50, 75].map((t) => (
                <div
                  key={t}
                  className="absolute left-0 right-0 h-px"
                  style={{
                    top: `${t}%`,
                    background: t === 0 ? 'rgba(255,255,255,0.07)' : 'rgba(255,255,255,0.05)',
                  }}
                />
              ))}
              <div className="absolute bottom-0 left-0 right-0 h-px bg-white/[0.14]" />
            </div>
            <div className="absolute bottom-px left-0 right-0 top-0 grid grid-cols-5 items-end gap-[22px]">
              {registry.map((bar, i) => (
                <div key={bar.year} className="relative flex h-full flex-col justify-end">
                  <div
                    className="mb-[9px] text-center font-mono text-[12px] font-semibold"
                    style={{
                      color: bar.current ? '#86efac' : '#8a948d',
                      opacity: seen ? 1 : 0,
                      transition: `opacity 0.5s ease ${180 + i * 130}ms`,
                    }}
                  >
                    {bar.count}
                  </div>
                  <div
                    style={{
                      height: seen ? `${(bar.count / max) * 100}%` : '0%',
                      borderRadius: '6px 6px 0 0',
                      transition: `height 1.05s cubic-bezier(0.16,0.84,0.28,1) ${180 + i * 130}ms`,
                      background: `linear-gradient(to top, ${
                        bar.current ? 'rgba(52,211,153,0.75)' : 'rgba(52,211,153,0.4)'
                      }, rgba(134,239,172,0.07))`,
                      borderTop: `2px solid ${bar.current ? '#86efac' : 'rgba(134,239,172,0.6)'}`,
                      boxShadow: bar.current ? '0 -8px 40px rgba(52,211,153,0.35)' : undefined,
                    }}
                  />
                </div>
              ))}
            </div>
          </div>
          <div className="grid grid-cols-5 gap-[22px] pt-3">
            {registry.map((bar) => (
              <div key={bar.year} className="text-center">
                <div
                  className="font-mono text-[11.5px] tracking-[0.06em]"
                  style={{ color: bar.current ? '#eef2ef' : '#8a948d' }}
                >
                  {bar.year}
                </div>
                <div className="mt-[5px] text-[11.5px] text-[#8a948d]">{bar.note}</div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}

/** Scan output, revealed one line at a time. */
function TerminalPanel() {
  const { ref, seen } = useInView<HTMLDivElement>(0.4)

  // Wrapped in objects rather than a bare JSX array so each line carries its
  // own stable id for the key.
  const lines: Array<{ id: string; el: React.ReactNode }> = [
    {
      id: 'info',
      el: (
        <>
          <span className="text-acc-text">INFO</span>
          <span className="text-sec">{'  '}71 detectors loaded · chain=evm · dynamic engine armed</span>
        </>
      ),
    },
    {
      id: 'scan',
      el: (
        <>
          <span className="text-[#8fdcb2]">SCAN</span>
          <span className="text-sec">{'  '}Deployed Vault.sol to in-memory EVM, driving call sequences…</span>
        </>
      ),
    },
    { id: 'gap-1', el: <>&nbsp;</> },
    {
      id: 'critical',
      el: (
        <>
          <span className="text-[#ef4444]">CRITICAL</span>
          <span className="text-[#d8ddd9]">{'  '}Invariant violated: sum(balanceOf) == totalSupply()</span>
        </>
      ),
    },
    {
      id: 'values',
      el: <span className="text-[#748078]">{'      '}sum(balanceOf) = 2000{'  '}!={'  '}totalSupply() = 1000</span>,
    },
    { id: 'repro', el: <span className="text-[#748078]">{'      '}Minimal reproduction (2 calls):</span> },
    {
      id: 'call-1',
      el: <span className="text-[#748078]">{'        '}1. airdrop(0x0101…, 1000){'  '}[caller=0x1111…]</span>,
    },
    {
      id: 'call-2',
      el: <span className="text-[#748078]">{'        '}2. airdrop(0x0101…, 1000){'  '}[caller=0x1111…]</span>,
    },
    { id: 'gap-2', el: <>&nbsp;</> },
    {
      id: 'done',
      el: (
        <>
          <span className="text-acc-text">DONE</span>
          <span className="text-sec">{'  '}Reproduced and shrunk. Exit 1 — deploy blocked.</span>
          <span className="text-acc-text" style={{ animation: 'blink 1s infinite' }}>▊</span>
        </>
      ),
    },
  ]

  return (
    <div className="rounded-[18px] border border-white/[0.08] bg-[rgba(6,10,8,0.9)] p-1.5 shadow-[0_30px_80px_rgba(0,0,0,0.5),0_0_60px_rgba(52,211,153,0.05)]">
      <div className="flex items-center gap-1.5 border-b border-white/[0.06] px-4 py-3">
        <span className="h-2.5 w-2.5 rounded-full bg-[#ef4444] opacity-80" />
        <span className="h-2.5 w-2.5 rounded-full bg-[#fbbf24] opacity-80" />
        <span className="h-2.5 w-2.5 rounded-full bg-acc-text opacity-80" />
        <span className="ml-2.5 font-mono text-[11px] text-[#5c665f]">
          truent check ./contracts --chain evm
        </span>
      </div>
      <div
        ref={ref}
        className="overflow-x-auto px-5 pb-[22px] pt-[18px] text-left font-mono text-[12.5px] leading-[1.85]"
      >
        <pre
          aria-hidden="true"
          className="mb-3.5 overflow-hidden whitespace-pre font-mono text-[clamp(3.4px,0.62vw,7.4px)] leading-[1.08] text-acc-text/50"
          style={{ opacity: seen ? 1 : 0, transition: 'opacity 0.25s ease 250ms' }}
        >
          {TRUENT_ASCII}
        </pre>
        {lines.map((line, i) => (
          <div
            key={line.id}
            style={{ opacity: seen ? 1 : 0, transition: `opacity 0.25s ease ${250 + (i + 1) * 320}ms` }}
          >
            {line.el}
          </div>
        ))}
      </div>
    </div>
  )
}

function FeatureCard({
  icon,
  title,
  children,
  span = false,
  featured = false,
  watermark,
  footer,
}: {
  icon: string
  title: string
  children: React.ReactNode
  span?: boolean
  featured?: boolean
  watermark?: string
  footer?: React.ReactNode
}) {
  return (
    <div
      className={`relative overflow-hidden rounded-[18px] border p-[34px] transition-all duration-[250ms] hover:-translate-y-1 hover:shadow-[0_20px_50px_rgba(0,0,0,0.4)] ${
        span ? 'md:col-span-2' : ''
      } ${
        featured
          ? 'border-acc-text/[0.22] bg-acc-text/[0.04] hover:border-acc-text/[0.45]'
          : 'border-hair bg-white/[0.02] hover:border-acc-text/[0.35]'
      }`}
    >
      {watermark && (
        <div className="pointer-events-none absolute -bottom-[60px] -right-10 font-mono text-[120px] text-acc-text/[0.04]">
          {watermark}
        </div>
      )}
      <div className="relative mb-5 flex h-11 w-11 items-center justify-center rounded-xl border border-acc-text/20 bg-acc-text/[0.08] text-[19px]">
        {icon}
      </div>
      <h3 className="relative m-0 mb-3 text-[18px] font-medium text-text">{title}</h3>
      <div className="relative text-[13.5px] leading-[1.7] text-sec">{children}</div>
      {footer}
    </div>
  )
}

// ─────────────────────────────────────────────────────────────────────────────
// Page
// ─────────────────────────────────────────────────────────────────────────────

export default function HomePage() {
  const [authOpen, setAuthOpen] = useState(false)
  const [authTab, setAuthTab] = useState<'signin' | 'signup'>('signin')
  const [sampleReportOpen, setSampleReportOpen] = useState(false)

  const startTrial = () => {
    setAuthTab('signup')
    setAuthOpen(true)
  }

  return (
    <div className="min-h-screen bg-bg p-2.5">
      <div
        className="relative overflow-clip rounded-[22px] border border-white/[0.05]"
        style={{
          background:
            'radial-gradient(1100px 600px at 50% -100px, rgba(52,211,153,0.16), rgba(6,9,8,0) 60%), #060908',
        }}
      >
        <MarketingNav />

        <ParticleHero
          ascii={TRUENT_ASCII}
          wordmark="TRUENT"
          driftLeft={driftLeft}
          driftRight={driftRight}
          eyebrow={
            <span className="inline-flex items-center gap-2 rounded-full border border-acc-text/20 bg-acc-text/[0.07] px-4 py-[7px] font-mono text-[11px] uppercase tracking-[0.18em] text-[#8fdcb2]">
              <span className="inline-block h-[5px] w-[5px] rounded-full bg-acc-text" />
              Smart contract security intelligence
            </span>
          }
          headline={
            <>
              Don&apos;t get hacked.{' '}
              <span
                className="bg-clip-text text-transparent"
                style={{ backgroundImage: 'linear-gradient(100deg,#d7ffe9 0%,#34d399 55%,#8fdcb2 100%)' }}
              >
                Prove it by execution.
              </span>
            </>
          }
          subline={
            <>
              Findings a fuzzer <span className="text-acc-text">proved by execution</span> — not a
              model&apos;s opinion. <span className="text-acc-text">71 detectors</span> across EVM,
              Solana, Move and Soroban.
            </>
          }
          bullets={
            <>
              {[
                ['◆', 'Runnable PoCs'],
                ['◇', '4 chains, one engine'],
                ['◇', 'Zero unproved findings'],
                ['◇', 'CI-native'],
              ].map(([mark, label]) => (
                <span key={label} className="flex items-center gap-1.5">
                  <span className="text-acc-text">{mark}</span>
                  {label}
                </span>
              ))}
            </>
          }
          actions={
            <>
              <PrimaryCta onClick={startTrial} className="shadow-[0_0_40px_rgba(52,211,153,0.15)]">
                Start free trial
              </PrimaryCta>
              <GhostCta onClick={() => setSampleReportOpen(true)}>View a sample report</GhostCta>
            </>
          }
        />

        {/* ─── Track record ─── */}
        <Reveal className="relative mx-auto max-w-[1180px] px-6 pb-24 pt-[110px]">
          <div className="grid items-end gap-14 md:grid-cols-[0.9fr_1.1fr]">
            <div>
              <span className="font-mono text-[10.5px] tracking-[0.18em] text-[#4d564f]">
                [ Truent record ]
              </span>
              <h2 className="m-0 mt-5 text-[clamp(28px,3.6vw,42px)] font-normal leading-[1.16] tracking-[-0.025em] text-[#f2f6f2]">
                Our engine has reproduced every major exploit class in{' '}
                <span
                  className="bg-clip-text text-transparent"
                  style={{ backgroundImage: 'linear-gradient(96deg,#34d399 0%,#a3e635 45%,#fde047 100%)' }}
                >
                  a fraction of an audit cycle
                </span>
              </h2>
            </div>
            <p className="m-0 max-w-[420px] text-[13.5px] leading-[1.8] text-[#8a948d]">
              Every detector in the registry was written against a real incident and is kept alive by a
              reproduction test. The chart below is the registry itself — one bar per year, counting the
              detectors shipping in that release.
            </p>
          </div>

          <div className="mt-14 grid grid-cols-1 border-t border-white/[0.09] sm:grid-cols-3">
            {[
              { value: 71, decimals: 0, prefix: '', suffix: '+', label: 'Detectors shipping today' },
              { value: 21, decimals: 0, prefix: '', suffix: '+', label: 'Historic exploits reproduced end-to-end' },
              { value: 1.76, decimals: 2, prefix: '$', suffix: 'B', label: 'Real-world losses covered by the registry' },
            ].map((s, i) => (
              <div
                key={s.label}
                className={`py-[26px] ${i === 0 ? 'sm:pr-[30px]' : i === 1 ? 'sm:px-[30px]' : 'sm:pl-[30px]'} ${
                  i < 2 ? 'sm:border-r sm:border-white/[0.07]' : ''
                }`}
              >
                <div className="text-[clamp(34px,4vw,46px)] font-normal leading-none tracking-[-0.03em] text-text">
                  {s.prefix}
                  <AnimatedCounter value={s.value} decimals={s.decimals} />
                  <span className="text-acc-text">{s.suffix}</span>
                </div>
                <div className="mt-2.5 text-[12.5px] leading-[1.5] text-[#748078]">{s.label}</div>
              </div>
            ))}
          </div>

          <RegistryChart />
        </Reveal>

        {/* ─── See it run ─── */}
        <Reveal className="mx-auto grid max-w-[1100px] items-center gap-14 px-6 pb-[100px] pt-16 md:grid-cols-[0.85fr_1.15fr]">
          <div>
            <span className="font-mono text-[11px] uppercase tracking-[0.2em] text-acc-text">
              See it run
            </span>
            <h2 className="m-0 mt-[18px] text-[38px] font-normal leading-[1.15] tracking-[-0.02em] text-[#f2f6f2]">
              One command.
              <br />
              Every invariant.
            </h2>
            <p className="m-0 mt-[18px] text-[14px] leading-[1.75] text-sec">
              Point Truent at a contract and it deploys, drives adversarial call sequences, and checks
              every invariant after every call — then shrinks any violation to a minimal
              proof-of-concept you can run.
            </p>
            <Link
              href="/docs#cli"
              className="mt-6 inline-flex items-center gap-2 rounded-full border border-white/[0.14] px-[22px] py-3 text-[13px] font-medium text-[#cfd6d1] transition-colors hover:border-acc-text/50 hover:text-text"
            >
              Read the CLI reference →
            </Link>
          </div>
          <TerminalPanel />
        </Reveal>

        {/* ─── Capabilities ─── */}
        <Reveal className="mx-auto max-w-[1100px] px-6 pb-[100px] pt-10">
          <div id="features" className="mb-14 text-center">
            <Eyebrow>Capabilities</Eyebrow>
            <SectionHeading>An engine that can prove it.</SectionHeading>
            <p className="mx-auto mt-4 max-w-[520px] text-[14px] leading-[1.7] text-sec">
              Static analysis finds patterns. Language models find suspicions. Truent runs the code and
              settles it.
            </p>
          </div>
          <div className="grid gap-3.5 md:grid-cols-3">
            <FeatureCard
              icon="◈"
              title="Every finding is proved, not guessed"
              span
              watermark="✓"
              footer={
                <Link
                  href="/library"
                  className="relative mt-4 inline-block text-[13px] font-semibold text-acc-text"
                >
                  Browse the Library →
                </Link>
              }
            >
              <p className="m-0 mb-3 max-w-[560px]">
                A prompt-only auditor stops at &ldquo;this looks like a bug.&rdquo; Truent deploys your
                contract in a real VM, drives adversarial sequences against it, and only reports a
                finding once it has made the bug actually fire — then shrinks the trace to the shortest
                sequence that reproduces it.
              </p>
              <p className="m-0 max-w-[560px]">
                You get a runnable proof-of-concept, so nothing lands in your report that can&apos;t be
                reproduced.
              </p>
            </FeatureCard>
            <FeatureCard icon="◆" title="AI, held to account">
              The model proposes candidate bugs; the engine tries to fire each one. Anything it
              can&apos;t reproduce is labelled as a lead, never as a finding.
            </FeatureCard>
            <FeatureCard icon="↺" title="Self-improving engine">
              Learns from every new exploit in the wild, automatically generating new detection modules
              within 24 hours.
            </FeatureCard>
            <FeatureCard icon="⎇" title="CI/CD integration">
              Native GitHub Actions and GitLab pipeline support. Block deploys on critical findings
              automatically.
            </FeatureCard>
            <FeatureCard icon="⟐" title="Symbolic execution" featured>
              Formal verification explores every execution path. Zero false negatives on all critical
              code paths.
            </FeatureCard>
          </div>
        </Reveal>

        {/* ─── How it works ─── */}
        <Reveal className="border-y border-white/[0.06] bg-white/[0.012] px-6 py-[100px]">
          <div className="mx-auto max-w-[1100px]">
            <div className="mb-[60px] text-center">
              <Eyebrow>How it works</Eyebrow>
              <SectionHeading>From code to coverage in minutes</SectionHeading>
            </div>
            <div className="grid gap-3.5 md:grid-cols-3">
              {steps.map((s) => (
                <div key={s.num} className="rounded-[18px] border border-hair bg-white/[0.02] p-[34px]">
                  <div className="mb-[18px] flex items-baseline gap-3.5">
                    <span className="text-[46px] font-light tracking-[-0.02em] text-[#8fdcb2]/35">
                      {s.num}
                    </span>
                    <span className="text-[20px]">{s.icon}</span>
                  </div>
                  <h3 className="m-0 mb-2.5 text-[16.5px] font-medium text-text">{s.title}</h3>
                  <p className="m-0 text-[13px] leading-[1.7] text-sec">{s.desc}</p>
                </div>
              ))}
            </div>
          </div>
        </Reveal>

        {/* ─── Real exploits ─── */}
        <Reveal className="mx-auto max-w-[1100px] px-6 py-[100px]">
          <div className="mb-[52px] text-center">
            <Eyebrow tone="red">Battle-tested against real exploits</Eyebrow>
            <SectionHeading>
              We study every major hack
              <br />
              so you don&apos;t have to.
            </SectionHeading>
            <p className="mx-auto mt-4 max-w-[540px] text-[14px] leading-[1.7] text-sec">
              Every invariant maps directly to a real-world exploit pattern. Truent would have flagged
              these before deployment.
            </p>
          </div>
          <div className="grid gap-3.5 md:grid-cols-3">
            {exploits.map((e) => (
              <div
                key={e.protocol}
                className="relative overflow-hidden rounded-[18px] border border-hair bg-white/[0.02] p-[30px] transition-all duration-[250ms] hover:-translate-y-1 hover:border-[#ef4444]/[0.35] hover:shadow-[0_20px_50px_rgba(0,0,0,0.4)]"
              >
                <div
                  className="absolute left-0 right-0 top-0 h-0.5"
                  style={{ background: 'linear-gradient(90deg,#ef4444,rgba(239,68,68,0.3),transparent)' }}
                />
                <div className="mb-4 flex items-start justify-between gap-3">
                  <div>
                    <div className="mb-1.5 font-mono text-[10.5px] tracking-[0.14em] text-[#748078]">
                      {e.year} EXPLOIT
                    </div>
                    <div className="text-[19px] font-medium text-text">{e.protocol}</div>
                  </div>
                  <span className="text-[23px] font-semibold tracking-[-0.02em] text-[#ef4444]">
                    {e.amount}
                  </span>
                </div>
                <p className="m-0 mb-[18px] text-[13px] leading-[1.6] text-sec">{e.type}</p>
                <div className="mb-4 flex flex-wrap items-center gap-2">
                  <span className="rounded-[5px] border border-[#ef4444]/30 bg-[#ef4444]/10 px-2 py-[3px] font-mono text-[10px] tracking-[0.1em] text-[#ef4444]">
                    CRITICAL
                  </span>
                  <code className="break-all rounded-[5px] border border-white/[0.08] bg-white/[0.03] px-2 py-[3px] font-mono text-[10.5px] text-[#8fa398]">
                    {e.invariant}
                  </code>
                </div>
                <div className="flex items-center gap-[7px] text-[12px] font-semibold text-acc-text">
                  ✓ Truent detects this pattern
                </div>
              </div>
            ))}
          </div>
        </Reveal>

        {/* ─── Reports ─── */}
        <Reveal className="border-y border-white/[0.06] bg-white/[0.012] px-6 py-[100px]">
          <div className="mx-auto grid max-w-[1100px] items-center gap-16 md:grid-cols-2">
            <div>
              <Eyebrow>Audit reports</Eyebrow>
              <h2 className="m-0 mt-[22px] text-[clamp(30px,4vw,42px)] font-normal tracking-[-0.02em] text-[#f2f6f2]">
                Professional grade reports
              </h2>
              <p className="mb-7 mt-[18px] text-[14px] leading-[1.75] text-sec">
                Generate executive-ready summaries with granular technical deep-dives. Integrated with
                GitHub and GitLab CI/CD pipelines out of the box.
              </p>
              <div className="mb-8 flex flex-col gap-3">
                {reportPerks.map((perk) => (
                  <div key={perk} className="flex items-center gap-3 text-[13.5px] text-[#c5cec8]">
                    <span className="text-acc-text">✓</span>
                    {perk}
                  </div>
                ))}
              </div>
              <GhostCta onClick={() => setSampleReportOpen(true)}>View sample report →</GhostCta>
            </div>

            <div className="rounded-[18px] border border-white/[0.08] bg-[rgba(6,10,8,0.85)] p-[26px] shadow-[0_30px_80px_rgba(0,0,0,0.45)]">
              <div className="mb-[22px] flex items-start justify-between gap-3">
                <div>
                  <div className="mb-1.5 font-mono text-[10px] tracking-[0.16em] text-[#748078]">
                    AUDIT REPORT
                  </div>
                  <div className="text-[19px] font-medium text-text">Circle-Pay BCH</div>
                  <div className="mt-1 text-[12px] text-[#748078]">Jun 6, 2026 · EVM · v2.1.0</div>
                </div>
                <span className="whitespace-nowrap rounded-[5px] border border-acc-text/25 bg-acc-text/[0.08] px-[9px] py-1 font-mono text-[10px] tracking-[0.12em] text-acc-text">
                  COMPLETE
                </span>
              </div>
              <div className="mb-5 grid grid-cols-4 gap-px overflow-hidden rounded-[10px] bg-white/[0.07]">
                {sevCounts.map((sv) => (
                  <div key={sv.label} className="bg-[#0a0f0c] px-2 py-4 text-center">
                    <div className="text-[26px] font-semibold" style={{ color: sv.color }}>
                      {sv.count}
                    </div>
                    <div className="mt-1 font-mono text-[9.5px] tracking-[0.14em] text-[#748078]">
                      {sv.label}
                    </div>
                  </div>
                ))}
              </div>
              <div className="mb-5 flex flex-col gap-2">
                {[
                  { sev: 'CRITICAL', color: '#ef4444', text: 'Reentrancy in withdrawAll()' },
                  { sev: 'HIGH', color: '#fbbf24', text: 'Unchecked external call return' },
                  { sev: 'HIGH', color: '#fbbf24', text: 'Missing oracle staleness check' },
                ].map((f) => (
                  <div
                    key={f.text}
                    className="flex items-center gap-3 rounded-[10px] border border-white/[0.05] bg-white/[0.02] px-3.5 py-[11px]"
                  >
                    <span
                      className="whitespace-nowrap rounded border px-[7px] py-0.5 font-mono text-[9.5px] tracking-[0.1em]"
                      style={{ color: f.color, borderColor: `${f.color}4d`, background: `${f.color}1a` }}
                    >
                      {f.sev}
                    </span>
                    <span className="text-[13px] text-[#c5cec8]">{f.text}</span>
                  </div>
                ))}
              </div>
              <button
                onClick={() => setSampleReportOpen(true)}
                className="block w-full rounded-full border border-white/[0.12] py-[11px] text-center text-[12.5px] font-medium text-[#cfd6d1] transition-colors hover:border-acc-text/50 hover:text-text"
              >
                ↓ View full report
              </button>
            </div>
          </div>
        </Reveal>

        {/* ─── Pricing preview ─── */}
        <Reveal className="mx-auto max-w-[1100px] px-6 pb-20 pt-[100px]">
          <div className="mb-14 text-center">
            <Eyebrow>Pricing</Eyebrow>
            <SectionHeading>Start free. Scale when you ship.</SectionHeading>
            <p className="mx-auto mt-4 max-w-[440px] text-[14px] leading-[1.7] text-sec">
              From indie developers to enterprise security teams.
            </p>
          </div>
          {/* Signal only — the full table lives on the Pricing page. */}
          <div className="grid overflow-hidden rounded-[18px] border border-hair md:grid-cols-3">
            {plans.map((plan, i) => (
              <Link
                key={plan.name}
                href={plan.href}
                className={`block px-[30px] py-7 transition-colors hover:bg-white/[0.03] ${
                  i < 2 ? 'md:border-r md:border-white/[0.07]' : ''
                } ${plan.featured ? 'bg-acc-text/[0.05]' : ''}`}
              >
                <div
                  className="font-mono text-[10.5px] uppercase tracking-[0.16em]"
                  style={{ color: plan.accent }}
                >
                  {plan.name}
                </div>
                <div className="mt-3.5 text-[34px] font-normal tracking-[-0.025em] text-[#f2f6f2]">
                  {plan.price}
                  <span className="text-[12.5px] text-[#5c665f]">{plan.per}</span>
                </div>
                <p className="m-0 mt-2.5 text-[12.5px] leading-[1.6] text-[#8a948d]">{plan.caption}</p>
              </Link>
            ))}
          </div>
          <div className="mt-7 text-center">
            <Link
              href="/pricing"
              className="inline-flex items-center gap-2.5 rounded-full border border-white/[0.16] px-5 py-[11px] font-mono text-[11px] font-semibold uppercase tracking-[0.12em] text-[#cfd6d1] transition-colors hover:border-acc-text/50 hover:text-text"
            >
              Compare plans in full
              <span className="text-[13px] tracking-[-0.12em]">❯❯</span>
            </Link>
          </div>
        </Reveal>

        {/* ─── Let's talk band ─── */}
        <section className="relative mt-10">
          <div
            className="relative flex h-[190px] items-center justify-center overflow-hidden"
            style={{
              background:
                'linear-gradient(104deg,#08301e 0%,#0d6b45 24%,#16915f 48%,#1fae74 64%,#0f7a4f 82%,#062418 100%)',
            }}
          >
            <div
              className="absolute inset-0"
              style={{
                background:
                  'radial-gradient(60% 120% at 22% 40%, rgba(180,140,20,0.22), transparent 60%), radial-gradient(50% 120% at 68% 60%, rgba(16,120,80,0.45), transparent 65%)',
              }}
            />
            <Link
              href="/contact"
              className="relative whitespace-nowrap text-center text-[clamp(78px,13vw,190px)] font-bold leading-[0.92] tracking-[-0.05em] text-[#f4faf6] drop-shadow-[0_2px_40px_rgba(3,20,12,0.45)]"
            >
              LET&apos;S TALK
            </Link>
          </div>
        </section>

        {/* ─── Final CTA ─── */}
        <Reveal className="mx-auto max-w-[640px] px-6 pb-[90px] pt-[70px] text-center">
          <h2 className="m-0 text-[clamp(26px,3.6vw,36px)] font-normal tracking-[-0.025em] text-[#f2f6f2]">
            Ready to audit smarter?
          </h2>
          <p className="mx-auto mt-4 max-w-[470px] text-[13.5px] leading-[1.8] text-[#8a948d]">
            Join the teams securing billions in on-chain value with invariant-driven security. Start a
            free scan, or talk to a security engineer about your protocol — we will run a
            proof-of-concept audit on your contracts before you commit to anything.
          </p>
          <div className="mt-8 flex flex-wrap justify-center gap-3">
            <PrimaryCta onClick={startTrial}>Start free trial</PrimaryCta>
            <GhostCta href="/contact">Talk to us</GhostCta>
          </div>
        </Reveal>

        <MarketingFooter />
      </div>

      <AuthModal isOpen={authOpen} onClose={() => setAuthOpen(false)} defaultTab={authTab} />
      <SampleReportModal isOpen={sampleReportOpen} onClose={() => setSampleReportOpen(false)} />
    </div>
  )
}
