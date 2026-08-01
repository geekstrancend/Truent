'use client'

import { useMemo, useState } from 'react'
import Link from 'next/link'
import { MarketingNav } from '@/components/layout/MarketingNav'
import { PageShell } from '@/components/layout/PageShell'
import { SlimFooter } from '@/components/layout/SlimFooter'
import { AsciiLogo } from '@/components/ui/AsciiLogo'

type Severity = 'critical' | 'high' | 'medium' | 'low'

interface Invariant {
  id: string
  severity: Severity
  title: string
  description: string
  tags: string[]
  cvss: number
  version: string
  audits: number
  chain: string
  category: string
}

const SEV_COLOR: Record<Severity, string> = {
  critical: '#ef4444',
  high: '#fbbf24',
  medium: '#818cf8',
  low: '#4ade80',
}

const INVARIANTS: Invariant[] = [
  { id: 'EVM-C01', severity: 'critical', title: 'evm_reentrancy_protection', description: 'Verifies all external calls following state changes are protected by a nonReentrant modifier or the CEI pattern is strictly followed throughout the function.', tags: ['CORE', 'CEI-PATTERN'], cvss: 9.1, version: 'v4.0.2', audits: 1200, chain: 'EVM', category: 'Core Safety' },
  { id: 'EVM-C02', severity: 'critical', title: 'evm_self_destruct_removal', description: 'Detects legacy SELFDESTRUCT opcodes which are deprecated post-Cancun and can lead to unexpected state destruction in proxy contracts.', tags: ['POST-CANCUN', 'GOVERNANCE'], cvss: 9.8, version: 'v1.1.0', audits: 12, chain: 'EVM', category: 'Core Safety' },
  { id: 'EVM-C03', severity: 'critical', title: 'evm_missing_post_state_health_check', description: 'Checks that after any flash-loan or large-value swap, a protocol health assertion (e.g. total assets >= total liabilities) is evaluated before the transaction finalises.', tags: ['FLASH-LOAN', 'DEFI'], cvss: 9.6, version: 'v1.0.0', audits: 8, chain: 'EVM', category: 'DeFi' },
  { id: 'EVM-C04', severity: 'critical', title: 'evm_merkle_root_zero_default', description: 'Ensures no Merkle root is initialised as bytes32(0), which makes all proofs trivially valid—the exact vector used in the $190M Nomad exploit.', tags: ['BRIDGES', 'MERKLE'], cvss: 9.9, version: 'v1.0.1', audits: 5, chain: 'EVM', category: 'Bridges' },
  { id: 'EVM-H01', severity: 'high', title: 'evm_oracle_heartbeat_freshness', description: 'Validates Oracle responses (Chainlink/Pyth) include an updatedAt timestamp within an acceptable heartbeat window before using the price.', tags: ['DEFI', 'ORACLES'], cvss: 7.5, version: 'v3.2.0', audits: 89, chain: 'EVM', category: 'Oracles' },
  { id: 'EVM-H02', severity: 'high', title: 'evm_cross_chain_arbitrary_message_validation', description: 'Ensures all incoming messages from LayerZero or Axelar endpoints are validated against a known source chain and sender address whitelist.', tags: ['BRIDGES', 'INTEROPERABILITY'], cvss: 8.8, version: 'v2.1.0', audits: 42, chain: 'EVM', category: 'Bridges' },
  { id: 'EVM-H03', severity: 'high', title: 'evm_unbacked_synthetic_mint', description: 'Verifies that every minting of a synthetic or wrapped token is backed 1:1 by a corresponding deposit or collateral lock before the mint executes.', tags: ['DEFI', 'TOKENS'], cvss: 8.2, version: 'v1.0.0', audits: 17, chain: 'EVM', category: 'DeFi' },
  { id: 'EVM-H04', severity: 'high', title: 'evm_dvn_single_point_failure', description: 'Ensures LayerZero DVN configurations require signatures from ≥2 distinct verification networks, preventing a single compromised DVN from authorising large transfers.', tags: ['BRIDGES', 'DVN'], cvss: 8.5, version: 'v1.0.0', audits: 3, chain: 'EVM', category: 'Bridges' },
  { id: 'EVM-M01', severity: 'medium', title: 'evm_integer_precision_loss', description: 'Detects division before multiplication patterns that can silently truncate values to zero in fixed-point arithmetic, particularly in share price calculations.', tags: ['MATH', 'PRECISION'], cvss: 5.9, version: 'v2.0.0', audits: 340, chain: 'EVM', category: 'Core Safety' },
  { id: 'EVM-M02', severity: 'medium', title: 'evm_governance_timelock_minimum', description: 'Asserts that all governance function calls are guarded by a timelock of at least 48 hours, giving stakeholders time to react to malicious proposals.', tags: ['GOVERNANCE', 'TIMELOCK'], cvss: 6.1, version: 'v1.3.0', audits: 78, chain: 'EVM', category: 'Governance' },
  { id: 'SOL-H01', severity: 'high', title: 'solana_account_signer_verification', description: 'Mandates that all accounts modifying program-owned state are verified as transaction signers before any state mutation occurs.', tags: ['SOLANA', 'SIGNER'], cvss: 8.0, version: 'v1.2.0', audits: 65, chain: 'Solana', category: 'Core Safety' },
  { id: 'SOL-M01', severity: 'medium', title: 'solana_account_owner_validation', description: 'Mandates that all AccountInfo data is checked for ownership by the calling program before state transitions are applied.', tags: ['SOLANA', 'OWNERSHIP'], cvss: 5.4, version: 'v1.0.4', audits: 186, chain: 'Solana', category: 'Core Safety' },
  { id: 'GEN-L01', severity: 'low', title: 'generic_event_emission_check', description: 'Standardises that every state-changing function emits a corresponding event for off-chain indexing and auditability.', tags: ['LOGGING', 'STANDARDS'], cvss: 2.1, version: 'v0.9.1', audits: 3450, chain: 'Generic', category: 'Standards' },
  { id: 'GEN-L02', severity: 'low', title: 'generic_access_control_two_step', description: 'Verifies that ownership transfers use a two-step accept pattern (propose + accept) rather than a single direct transfer.', tags: ['ACCESS-CONTROL', 'STANDARDS'], cvss: 3.2, version: 'v1.0.0', audits: 890, chain: 'Generic', category: 'Governance' },
]

const SEVERITIES = ['All', 'critical', 'high', 'medium', 'low'] as const
const CHAINS = ['All', 'EVM', 'Solana', 'Move', 'Generic'] as const
const CATEGORIES = ['All', 'Core Safety', 'DeFi', 'Bridges', 'Oracles', 'Governance', 'Standards'] as const

const cvssColor = (v: number) =>
  v >= 9 ? '#ef4444' : v >= 7 ? '#fbbf24' : v >= 4 ? '#818cf8' : '#34d399'

const auditsLabel = (n: number) => (n >= 1000 ? `${(n / 1000).toFixed(1)}k` : String(n))

const chipClass = (active: boolean) =>
  `cursor-pointer rounded-full border px-[13px] py-1.5 font-mono text-[10px] uppercase tracking-[0.1em] transition-colors ${
    active
      ? 'border-acc-text/[0.45] bg-acc-text/[0.08] text-acc-text'
      : 'border-white/[0.09] text-[#748078] hover:text-[#c5cec8]'
  }`

export default function LibraryPage() {
  const [search, setSearch] = useState('')
  const [chain, setChain] = useState<string>('All')
  const [category, setCategory] = useState<string>('All')
  const [severity, setSeverity] = useState<string>('All')

  const filtered = useMemo(() => {
    const q = search.toLowerCase()
    return INVARIANTS.filter(
      (i) =>
        (!q ||
          i.title.toLowerCase().includes(q) ||
          i.description.toLowerCase().includes(q) ||
          i.id.toLowerCase().includes(q) ||
          i.tags.some((t) => t.toLowerCase().includes(q))) &&
        (chain === 'All' || i.chain === chain) &&
        (category === 'All' || i.category === category) &&
        (severity === 'All' || i.severity === severity),
    )
  }, [search, chain, category, severity])

  const hasFilters = Boolean(search) || chain !== 'All' || category !== 'All' || severity !== 'All'

  return (
    <PageShell>
      <MarketingNav />

      {/* ─── Hero ─── */}
      <header className="relative mx-auto flex max-w-[1100px] flex-wrap items-end justify-between gap-8 overflow-hidden border-b border-white/[0.06] px-6 pb-11 pt-20">
        <div className="pointer-events-none absolute right-6 top-16 hidden opacity-[0.13] md:block">
          <AsciiLogo className="text-[clamp(3.4px,0.72vw,8.6px)] !leading-[1.08]" />
        </div>
        <div className="relative">
          <span className="inline-flex items-center gap-2 rounded-full border border-acc-text/20 bg-acc-text/[0.07] px-4 py-[7px] font-mono text-[11px] uppercase tracking-[0.18em] text-[#8fdcb2]">
            <span className="inline-block h-[5px] w-[5px] rounded-full bg-acc-text" />
            Invariant library
          </span>
          <h1 className="m-0 mt-[22px] text-[clamp(36px,5vw,58px)] font-normal tracking-[-0.03em] text-[#f2f6f2]">
            Security{' '}
            <span
              className="bg-clip-text text-transparent"
              style={{ backgroundImage: 'linear-gradient(100deg,#d7ffe9,#34d399)' }}
            >
              invariant library
            </span>
          </h1>
          <p className="m-0 mt-4 max-w-[520px] text-[14.5px] leading-[1.7] text-sec">
            50+ battle-tested security checks for EVM, Solana, and Move. Every invariant is mapped to a
            real exploit.
          </p>
        </div>
        <div className="relative flex flex-shrink-0 gap-2.5">
          <Link
            href="/contact"
            className="inline-flex items-center rounded-full border border-white/[0.14] px-5 py-[11px] text-[12.5px] font-medium text-[#cfd6d1] transition-colors hover:border-acc-text/50 hover:text-text"
          >
            ↗ Request invariant
          </Link>
          <Link
            href="/contact"
            className="inline-flex items-center gap-2.5 rounded-full bg-[#eef2ef] py-[5px] pl-4 pr-[5px] text-[12.5px] font-semibold text-[#0a0d0b] transition-colors hover:bg-white"
          >
            + Custom library
            <span className="flex h-[26px] w-[26px] items-center justify-center rounded-full bg-acc-text text-[13px] text-on-acc">
              →
            </span>
          </Link>
        </div>
      </header>

      {/* ─── Filters + results ─── */}
      <section className="mx-auto max-w-[1100px] px-6 pb-[100px] pt-8">
        <div className="mb-[18px] flex flex-wrap items-center gap-3.5">
          <div className="relative min-w-[260px] max-w-[380px] flex-1">
            <span className="absolute left-4 top-1/2 -translate-y-1/2 text-[13px] text-[#5c665f]">⌕</span>
            <input
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="Search by name, tag, or ID…"
              aria-label="Search invariants"
              className="w-full rounded-full border border-white/[0.09] bg-white/[0.03] py-[11px] pl-[38px] pr-[18px] font-mono text-[12.5px] text-text outline-none transition-colors placeholder:text-[#5c665f] focus:border-acc-text/50"
            />
          </div>
          <div className="flex flex-wrap gap-1.5">
            {SEVERITIES.map((s) => {
              const active = severity === s
              const c = s === 'All' ? '#34d399' : SEV_COLOR[s as Severity]
              return (
                <button
                  key={s}
                  onClick={() => setSeverity(s)}
                  aria-pressed={active}
                  className="cursor-pointer rounded-full border px-[13px] py-1.5 font-mono text-[10px] uppercase tracking-[0.1em] transition-colors"
                  style={{
                    borderColor: active ? c : 'rgba(255,255,255,0.09)',
                    background: active ? `${c}18` : 'transparent',
                    color: active ? c : '#748078',
                  }}
                >
                  {s}
                </button>
              )
            })}
          </div>
          <div className="flex flex-wrap gap-1.5">
            {CHAINS.map((c) => (
              <button key={c} onClick={() => setChain(c)} aria-pressed={chain === c} className={chipClass(chain === c)}>
                {c}
              </button>
            ))}
          </div>
        </div>

        <div className="mb-[26px] flex flex-wrap gap-1 border-b border-white/[0.06] pb-3.5">
          {CATEGORIES.map((c) => (
            <button
              key={c}
              onClick={() => setCategory(c)}
              aria-pressed={category === c}
              className={`cursor-pointer rounded-full px-4 py-2 text-[13px] transition-colors ${
                category === c ? 'bg-acc-text/10 font-medium text-text' : 'text-[#748078] hover:text-text'
              }`}
            >
              {c}
            </button>
          ))}
        </div>

        <p className="m-0 mb-[22px] font-mono text-[12px] text-[#748078]" aria-live="polite">
          <span className="text-acc-text">{filtered.length}</span> invariants
          {hasFilters ? ' matching filters' : ' in the public library'}
        </p>

        {filtered.length > 0 ? (
          <div className="grid gap-3.5 md:grid-cols-2 lg:grid-cols-3">
            {filtered.map((inv) => {
              const c = SEV_COLOR[inv.severity]
              return (
                <div
                  key={inv.id}
                  className="flex flex-col gap-3.5 rounded-[18px] border border-hair bg-white/[0.02] p-6 transition-all duration-[250ms] hover:-translate-y-1 hover:border-acc-text/[0.35] hover:shadow-[0_20px_50px_rgba(0,0,0,0.4)]"
                >
                  <div className="flex items-start justify-between gap-2">
                    <div className="flex flex-wrap gap-1.5">
                      {[inv.id, inv.chain].map((t) => (
                        <span
                          key={t}
                          className="rounded-[5px] border border-white/[0.08] bg-white/[0.03] px-2 py-[3px] font-mono text-[10px] text-[#8fa398]"
                        >
                          {t}
                        </span>
                      ))}
                    </div>
                    <span
                      className="flex-shrink-0 rounded-[5px] border px-2 py-[3px] font-mono text-[9.5px] tracking-[0.1em]"
                      style={{ color: c, background: `${c}1a`, borderColor: `${c}4d` }}
                    >
                      {inv.severity.toUpperCase()}
                    </span>
                  </div>

                  <div className="break-words font-mono text-[12.5px] font-semibold text-[#d7e2da]">
                    {inv.title}
                  </div>
                  <p className="m-0 flex-1 text-[12.5px] leading-[1.65] text-sec">{inv.description}</p>

                  <div className="flex flex-wrap gap-1.5">
                    {inv.tags.map((t) => (
                      <button
                        key={t}
                        onClick={() => setSearch(t.toLowerCase())}
                        className="cursor-pointer rounded-full border border-white/[0.09] px-2.5 py-1 font-mono text-[9.5px] tracking-[0.08em] text-[#748078] transition-colors hover:border-acc-text/40 hover:text-[#c5cec8]"
                      >
                        {t}
                      </button>
                    ))}
                  </div>

                  <div className="flex items-center justify-between border-t border-white/[0.06] pt-3 font-mono text-[10.5px] text-[#748078]">
                    <span>
                      CVSS{' '}
                      <span className="font-semibold" style={{ color: cvssColor(inv.cvss) }}>
                        {inv.cvss}
                      </span>
                      {'  ·  '}
                      {inv.version}
                    </span>
                    <span>{auditsLabel(inv.audits)} audits</span>
                  </div>

                  <Link
                    href="/docs#reports"
                    className="inline-flex items-center gap-2 self-start rounded-full border border-white/[0.14] px-4 py-[9px] font-mono text-[10px] font-semibold uppercase tracking-[0.12em] text-[#cfd6d1] transition-colors hover:border-acc-text/50 hover:text-text"
                  >
                    Learn more
                    <span className="text-[12px] tracking-[-0.12em]">❯❯</span>
                  </Link>
                </div>
              )
            })}
          </div>
        ) : (
          <div className="py-[90px] text-center text-[#748078]">
            <div className="mb-3.5 text-[34px] opacity-35">⌕</div>
            <p className="m-0 mb-1.5 text-[17px] text-sec">No invariants found</p>
            <p className="m-0 text-[13px]">Try adjusting your search or filters</p>
          </div>
        )}
      </section>

      <SlimFooter omit={['Library']} />
    </PageShell>
  )
}
