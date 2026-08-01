'use client'

import { useEffect, useState } from 'react'
import Link from 'next/link'
import { MarketingNav } from '@/components/layout/MarketingNav'
import { PageShell } from '@/components/layout/PageShell'
import { SlimFooter } from '@/components/layout/SlimFooter'
import { AsciiLogo } from '@/components/ui/AsciiLogo'

const PAGES = [
  { id: 'overview', label: 'Overview' },
  { id: 'getting-started', label: 'Getting Started' },
  { id: 'cli', label: 'CLI Reference' },
  { id: 'ai', label: 'AI Co-Auditor' },
  { id: 'api', label: 'REST API' },
  { id: 'ci-cd', label: 'CI/CD Integration' },
  { id: 'reports', label: 'Audit Reports' },
] as const

type PageId = (typeof PAGES)[number]['id']

const severityRows = [
  { level: 'Critical', color: '#ef4444', desc: 'Immediate fund loss or protocol compromise possible. Deploy-blocking.' },
  { level: 'High', color: '#fbbf24', desc: 'Significant vulnerability requiring urgent remediation before deployment.' },
  { level: 'Medium', color: '#818cf8', desc: 'Notable security issue that should be addressed before deployment.' },
  { level: 'Low', color: '#4ade80', desc: 'Minor issue or optimization opportunity with limited impact.' },
  { level: 'Info', color: '#96a19a', desc: 'Informational finding or best practice recommendation.' },
]

const cliFlags = [
  { flag: '--chain', desc: 'Blockchain target: evm, solana, move', def: 'evm' },
  { flag: '--format', desc: 'Output format: json, html, pdf', def: 'pdf' },
  { flag: '--output', desc: 'Output file path', def: 'report.pdf' },
  { flag: '--seed', desc: 'Reproducible randomness seed', def: 'auto' },
  { flag: '--verbose', desc: 'Enable verbose logging', def: 'false' },
]

const exitCodes = [
  { code: '0', color: '#34d399', meaning: 'Scan completed successfully, no critical/high findings' },
  { code: '1', color: '#818cf8', meaning: 'Scan found medium or low severity findings' },
  { code: '2', color: '#fbbf24', meaning: 'Scan found high severity findings' },
  { code: '3', color: '#ef4444', meaning: 'Scan found critical findings or scan failed' },
]

// ─────────────────────────────────────────────────────────────────────────────
// Primitives
// ─────────────────────────────────────────────────────────────────────────────

const H1 = ({ children }: { children: React.ReactNode }) => (
  <h1 className="m-0 text-[clamp(32px,4vw,46px)] font-normal tracking-[-0.03em] text-[#f2f6f2]">{children}</h1>
)

const Lede = ({ children }: { children: React.ReactNode }) => (
  <p className="m-0 mt-4 max-w-[560px] text-[15px] leading-[1.75] text-sec">{children}</p>
)

const H2 = ({ children, mono }: { children: React.ReactNode; mono?: boolean }) => (
  <h2
    className={`mb-3.5 mt-[52px] text-[22px] font-normal tracking-[-0.02em] text-[#f2f6f2] ${mono ? 'font-mono' : ''}`}
  >
    {children}
  </h2>
)

const H3 = ({ children }: { children: React.ReactNode }) => (
  <h3 className="mb-3 mt-[26px] text-[15.5px] font-medium text-[#d7e2da]">{children}</h3>
)

const P = ({ children }: { children: React.ReactNode }) => (
  <p className="m-0 mb-5 text-[13.5px] leading-[1.75] text-sec">{children}</p>
)

const Code = ({ children }: { children: React.ReactNode }) => (
  <code className="rounded border border-white/[0.08] bg-white/[0.04] px-1.5 py-0.5 font-mono text-[11.5px] text-acc-text">
    {children}
  </code>
)

/** Single shell command, with the design's dim prompt. */
const Cmd = ({ children }: { children: React.ReactNode }) => (
  <div className="overflow-x-auto rounded-xl border border-white/[0.08] bg-[#080c0a] px-[18px] py-4 font-mono text-[12.5px] text-acc-text">
    <span className="text-[#5c665f]">$ </span>
    {children}
  </div>
)

/** Multi-line block; children carry their own highlighting. */
const Block = ({ children }: { children: React.ReactNode }) => (
  <pre className="overflow-x-auto rounded-xl border border-white/[0.08] bg-[#080c0a] p-[18px] font-mono text-[12.5px] leading-[1.9] text-sec">
    {children}
  </pre>
)

const K = ({ children }: { children: React.ReactNode }) => <span className="text-[#8fdcb2]">{children}</span>
const V = ({ children }: { children: React.ReactNode }) => <span className="text-acc-text">{children}</span>

/** Bordered grid table. `cols` is a CSS grid template. */
function Table({
  cols,
  head,
  rows,
}: {
  cols: string
  head?: string[]
  rows: React.ReactNode[][]
}) {
  return (
    <div className="overflow-x-auto rounded-xl border border-hair">
      <div className="min-w-[420px]">
        {head && (
          <div
            className="grid border-b border-white/[0.06] bg-white/[0.03] font-mono text-[10px] uppercase tracking-[0.14em] text-[#8fdcb2]"
            style={{ gridTemplateColumns: cols }}
          >
            {head.map((h) => (
              <div key={h} className="px-4 py-[11px]">{h}</div>
            ))}
          </div>
        )}
        {rows.map((r, i) => (
          <div
            key={i}
            className="grid border-b border-white/[0.04] last:border-b-0"
            style={{ gridTemplateColumns: cols }}
          >
            {r.map((c, j) => (
              <div key={j} className="px-4 py-3 text-[12.5px] text-sec">{c}</div>
            ))}
          </div>
        ))}
      </div>
    </div>
  )
}

const SeverityTable = () => (
  <Table
    cols="1fr 2.6fr"
    rows={severityRows.map((r) => [
      <span key="l" className="font-semibold" style={{ color: r.color }}>{r.level}</span>,
      r.desc,
    ])}
  />
)

// ─────────────────────────────────────────────────────────────────────────────
// Sections
// ─────────────────────────────────────────────────────────────────────────────

function Overview({ go }: { go: (p: PageId) => void }) {
  const cards = [
    { icon: '▸', title: 'Quick Start', desc: 'Install the CLI and run your first scan in under 5 minutes.', cta: 'Get started', to: 'getting-started' as const },
    { icon: '⌘', title: 'CLI Reference', desc: 'Every command, flag, and configuration option documented.', cta: 'View commands', to: 'cli' as const },
    { icon: '◈', title: 'Invariant Library', desc: '50+ security checks. Browse and filter by chain and severity.', cta: 'Browse invariants', href: '/library' },
    { icon: '⎇', title: 'CI/CD Integration', desc: 'Connect Truent to GitHub Actions or GitLab CI in one step.', cta: 'Set up pipeline', to: 'ci-cd' as const },
    { icon: '◆', title: 'AI Co-Auditor', desc: 'How the AI layer detects protocol-level logical vulnerabilities.', cta: 'Learn more', to: 'ai' as const, badge: true },
    { icon: '⟐', title: 'REST API', desc: 'Integrate Truent headlessly into your own tooling and workflows.', cta: 'API reference', to: 'api' as const },
  ]

  const inner = (c: (typeof cards)[number]) => (
    <>
      <div className="flex items-center justify-between">
        <span className="flex h-[38px] w-[38px] items-center justify-center rounded-[11px] border border-acc-text/20 bg-acc-text/[0.08] text-[16px]">
          {c.icon}
        </span>
        {c.badge && (
          <span className="rounded-[5px] border border-acc-text/25 bg-acc-text/10 px-2 py-[3px] font-mono text-[9.5px] tracking-[0.12em] text-acc-text">
            PRO
          </span>
        )}
      </div>
      <div>
        <div className="mb-1.5 text-[15.5px] font-medium text-text">{c.title}</div>
        <p className="m-0 text-[12.5px] leading-[1.6] text-sec">{c.desc}</p>
      </div>
      <span className="mt-auto text-[12px] font-semibold text-acc-text">{c.cta} →</span>
    </>
  )

  const cardClass =
    'flex flex-col gap-3 rounded-2xl border border-hair bg-white/[0.02] p-6 text-left transition-all duration-200 hover:-translate-y-[3px] hover:border-acc-text/[0.35]'

  return (
    <article>
      <span className="inline-block rounded-full border border-acc-text/20 bg-acc-text/[0.06] px-4 py-[7px] font-mono text-[11px] uppercase tracking-[0.18em] text-[#8fdcb2]">
        Documentation
      </span>
      <h1 className="m-0 mt-[22px] text-[clamp(34px,4.5vw,52px)] font-normal tracking-[-0.03em] text-[#f2f6f2]">
        Truent{' '}
        <span
          className="bg-clip-text text-transparent"
          style={{ backgroundImage: 'linear-gradient(100deg,#d7ffe9,#34d399)' }}
        >
          documentation
        </span>
      </h1>
      <Lede>
        Everything you need to audit, secure, and ship smart contracts with confidence. From first scan
        to CI/CD integration.
      </Lede>

      <div className="mt-10 grid gap-px overflow-hidden rounded-[14px] border border-hair bg-white/[0.07] sm:grid-cols-3">
        {[
          { label: 'Install', cmd: 'cargo install truent-cli' },
          { label: 'Scan', cmd: 'truent check ./contracts --chain evm' },
          { label: 'CI', cmd: 'uses: truent-dev/truent-action@v1' },
        ].map((s) => (
          <div key={s.label} className="bg-[#080c0a] px-[18px] py-4">
            <div className="mb-2 font-mono text-[9.5px] uppercase tracking-[0.16em] text-[#5c665f]">
              {s.label}
            </div>
            <code className="break-all font-mono text-[12px] text-acc-text">{s.cmd}</code>
          </div>
        ))}
      </div>

      <h2 className="mb-[22px] mt-14 text-[23px] font-normal tracking-[-0.02em] text-[#f2f6f2]">
        Explore the docs
      </h2>
      <div className="grid gap-3 sm:grid-cols-2">
        {cards.map((c) =>
          c.href ? (
            <Link key={c.title} href={c.href} className={cardClass}>
              {inner(c)}
            </Link>
          ) : (
            <button key={c.title} onClick={() => c.to && go(c.to)} className={cardClass}>
              {inner(c)}
            </button>
          ),
        )}
      </div>

      <div className="mt-11 rounded-2xl border border-acc-text/20 bg-acc-text/[0.04] p-[26px]">
        <h3 className="m-0 mb-[18px] text-[16px] font-medium text-text">Pro tips</h3>
        <div className="flex flex-col gap-3">
          {[
            <>Add a <Code>TRUENT.md</Code> in your repo root to give context to the AI Co-Auditor</>,
            <>Integrate Truent into CI/CD to scan on every pull request before merge</>,
            <>Use custom invariant libraries to define protocol-specific security rules</>,
            <>Share reports directly with your audit team via GitHub and GitLab integrations</>,
          ].map((tip, i) => (
            <div key={i} className="flex gap-3 text-[13px] leading-[1.6] text-sec">
              <span className="flex-shrink-0 font-mono text-acc-text">0{i + 1}</span>
              <span>{tip}</span>
            </div>
          ))}
        </div>
      </div>
    </article>
  )
}

function GettingStarted() {
  return (
    <article>
      <H1>Getting started</H1>
      <Lede>
        Install Truent and run your first security scan in minutes. This guide walks you through the
        essential steps.
      </Lede>

      <H2>Installation</H2>
      <P>
        Truent provides two installation methods: Rust CLI for local scanning and an npm package for
        JavaScript integration.
      </P>
      <H3>Rust CLI (recommended)</H3>
      <Cmd>cargo install truent-cli</Cmd>
      <p className="m-0 mt-2.5 text-[12px] text-[#748078]">
        Requires Rust 1.70 or later. Install Rust at{' '}
        <a href="https://rustup.rs" target="_blank" rel="noopener noreferrer" className="text-acc-text">
          rustup.rs
        </a>
        .
      </p>
      <H3>npm package</H3>
      <Cmd>npm install -g @dextonicx/cli</Cmd>

      <H2>Running your first scan</H2>
      <P>Once installed, run Truent on a smart contract directory:</P>
      <Cmd>truent check . --chain evm</Cmd>
      <p className="m-0 mt-4 text-[13.5px] leading-[1.75] text-sec">
        This scans all Solidity files in the current directory using the EVM analyzer. Replace{' '}
        <Code>evm</Code> with <Code>solana</Code> for Rust programs or <Code>move</Code> for Move
        modules.
      </p>

      <H3>Common options</H3>
      <Table
        cols="1fr 2fr"
        head={['Flag', 'Description']}
        rows={[
          [<span key="f" className="font-mono text-[12px] text-[#d7e2da]">--chain</span>, 'Blockchain: evm, solana, move'],
          [<span key="f" className="font-mono text-[12px] text-[#d7e2da]">--format</span>, 'Output format: json, html, pdf (default: pdf)'],
          [<span key="f" className="font-mono text-[12px] text-[#d7e2da]">--output</span>, 'Output file path'],
          [<span key="f" className="font-mono text-[12px] text-[#d7e2da]">--seed</span>, 'Random seed for reproducible results'],
        ]}
      />

      <H2>Understanding the output</H2>
      <P>
        Truent reports findings using a standard severity classification system. Each finding is
        categorized by impact level:
      </P>
      <SeverityTable />

      <H2>Next steps</H2>
      <div className="flex flex-col gap-3">
        {[
          ['CLI Reference', 'Deep dive into all available commands and flags'],
          ['Invariant Library', 'Browse the built-in security checks'],
          ['Audit Report Guide', 'Learn how to interpret and share reports'],
          ['CI/CD Integration', 'Automate security scanning in your pipeline'],
        ].map(([title, desc]) => (
          <div key={title} className="flex gap-3 text-[13.5px] leading-[1.65] text-sec">
            <span className="text-acc-text">→</span>
            <span>
              <strong className="font-medium text-text">{title}</strong> — {desc}
            </span>
          </div>
        ))}
      </div>
    </article>
  )
}

function CliReference() {
  return (
    <article>
      <H1>CLI reference</H1>
      <Lede>
        Complete documentation for the Truent command-line interface and all available commands.
      </Lede>

      <H2 mono>truent check</H2>
      <P>Scan a smart contract directory or file for security vulnerabilities and invariant violations.</P>
      <Cmd>truent check [PATH] [OPTIONS]</Cmd>

      <H3>Options</H3>
      <Table
        cols="1fr 2fr 0.8fr"
        head={['Flag', 'Description', 'Default']}
        rows={cliFlags.map((f) => [
          <span key="f" className="font-mono text-[12px] text-[#d7e2da]">{f.flag}</span>,
          f.desc,
          <span key="d" className="font-mono text-[11.5px] text-[#748078]">{f.def}</span>,
        ])}
      />

      <H2>Exit codes</H2>
      <P>
        Truent uses exit codes to indicate scan results. Use these in CI/CD pipelines to gate
        deployments.
      </P>
      <Table
        cols="0.4fr 3fr"
        rows={exitCodes.map((c) => [
          <span key="c" className="font-mono text-[13px] font-semibold" style={{ color: c.color }}>{c.code}</span>,
          c.meaning,
        ])}
      />

      <H2>Examples</H2>
      <H3>Scan current directory for EVM contracts</H3>
      <Cmd>truent check . --chain evm</Cmd>
      <H3>Generate JSON report to a specific file</H3>
      <Cmd>truent check ./contracts --chain evm --format json --output security-report.json</Cmd>
      <H3>Scan Solana project with reproducible seed</H3>
      <Cmd>truent check . --chain solana --seed 42</Cmd>
      <H3>CI/CD gating example</H3>
      <Block>
        <V>truent check . --chain evm --format json --output report.json</V>
        {`\nif [ $? -ge 2 ]; then\n  echo "Critical or high findings detected"\n  exit 1\nfi`}
      </Block>
    </article>
  )
}

function AiCoAuditor() {
  const stages = [
    { icon: '◈', label: '1. LIBRARY', desc: 'Scanning global invariant patterns and protocol specs.' },
    { icon: '◎', label: '2. HITS', desc: 'Identifying potential violations using symbolic execution.' },
    { icon: '⚙', label: '3. ENRICHMENT', desc: 'The model filters noise and constructs attack vectors.', featured: true },
  ]

  return (
    <article>
      <H1>AI Co-Auditor</H1>
      <Lede>
        How the AI layer reasons across millions of code pathways to detect protocol-level logical flaws
        — and why every lead is still put to the engine.
      </Lede>

      <div className="mt-10 grid items-center gap-4 rounded-[18px] border border-hair bg-white/[0.02] p-8 md:grid-cols-[1fr_auto_1fr_auto_1fr]">
        {stages.map((s, i) => (
          <div key={s.label} className="contents">
            <div
              className={`rounded-[14px] border p-6 text-center ${
                s.featured ? 'border-acc-text/25 bg-acc-text/[0.05]' : 'border-hair bg-[rgba(6,10,8,0.7)]'
              }`}
            >
              <div className="mb-3 text-[22px]">{s.icon}</div>
              <div
                className={`mb-2 font-mono text-[10px] tracking-[0.16em] ${
                  s.featured ? 'text-acc-text' : 'text-[#8fdcb2]'
                }`}
              >
                {s.label}
              </div>
              <p className="m-0 text-[12px] leading-[1.6] text-sec">{s.desc}</p>
            </div>
            {i < stages.length - 1 && (
              <div className="hidden text-[18px] text-acc-text md:block">→</div>
            )}
          </div>
        ))}
      </div>

      <div className="mt-[26px] rounded-[14px] border border-acc-text/20 bg-acc-text/[0.04] px-6 py-[22px]">
        <div className="mb-2.5 font-mono text-[10px] tracking-[0.16em] text-acc-text">PRO TIP</div>
        <p className="m-0 text-[13.5px] leading-[1.7] text-sec">
          The AI Co-Auditor works best when project documentation is provided via a{' '}
          <Code>TRUENT.md</Code> file in the root directory.
        </p>
      </div>

      <H2>Comparison</H2>
      <Table
        cols="1fr 1fr 1fr"
        head={['Feature', 'Without AI', 'With Co-Auditor']}
        rows={[
          ['Logic flaw detection', 'Pattern-based only', <span key="a" className="font-medium text-acc-text">Reasoning-based discovery</span>],
          ['False positive rate', 'High (manual triage)', <span key="b" className="font-medium text-acc-text">Low (80% noise reduction)</span>],
          ['Inferred invariants', 'None', <span key="c" className="font-medium text-acc-text">Auto-generated .sinv rules</span>],
        ]}
      />

      <H2>Configuration</H2>
      <Block>
        <K>ai</K>:{'\n  '}
        <K>enabled</K>: <V>true</V>{'\n  '}
        <K>model</K>: <V>claude-sonnet</V>{'\n  '}
        <K>context_window</K>: <V>full</V>{'\n  '}
        <K>remediation_prompts</K>: <V>true</V>{'\n  '}
        <K>chat_enabled</K>: <V>true</V>
      </Block>

      <H2>How it works</H2>
      <div className="flex flex-col gap-6">
        <div>
          <h3 className="m-0 mb-2 text-[15px] font-medium text-[#d7e2da]">1. Symbolic execution</h3>
          <p className="m-0 text-[13.5px] leading-[1.75] text-sec">
            The invariant library is scanned against your contract code paths, creating a set of
            potential violations. Each violation is tracked with its execution context.
          </p>
        </div>
        <div>
          <h3 className="m-0 mb-2 text-[15px] font-medium text-[#d7e2da]">2. AI enrichment</h3>
          <p className="m-0 mb-2.5 text-[13.5px] leading-[1.75] text-sec">
            The model receives the top candidates along with your protocol documentation, then performs
            deep reasoning to:
          </p>
          <div className="flex flex-col gap-[7px] pl-1">
            {[
              'Filter false positives by understanding business logic intent',
              'Construct realistic attack vectors',
              'Generate specialized test cases for edge conditions',
              'Suggest remediation code snippets',
            ].map((s) => (
              <span key={s} className="text-[13px] text-sec">• {s}</span>
            ))}
          </div>
        </div>
        <div>
          <h3 className="m-0 mb-2 text-[15px] font-medium text-[#d7e2da]">3. Actionable reports</h3>
          <p className="m-0 text-[13.5px] leading-[1.75] text-sec">
            The output is a prioritized list of confirmed vulnerabilities with clear remediation
            guidance and formal verification proofs.
          </p>
        </div>
      </div>
    </article>
  )
}

function RestApi() {
  const Verb = ({ method, path }: { method: 'POST' | 'GET'; path: string }) => (
    <h2 className="mb-3.5 mt-[52px] text-[22px] font-normal tracking-[-0.02em] text-[#f2f6f2]">
      <span
        className={`align-middle rounded-[5px] border px-[9px] py-1 font-mono text-[12px] ${
          method === 'POST'
            ? 'border-acc-text/25 bg-acc-text/10 text-acc-text'
            : 'border-[#8fdcb2]/25 bg-[#8fdcb2]/10 text-[#8fdcb2]'
        }`}
      >
        {method}
      </span>
      <span className="ml-3 font-mono text-[19px]">{path}</span>
    </h2>
  )

  return (
    <article>
      <H1>REST API reference</H1>
      <Lede>Integrate Truent programmatically for headless security scanning and report generation.</Lede>

      <H2>Authentication</H2>
      <P>
        All API requests require an API key in the <Code>Authorization</Code> header. Generate keys in
        your dashboard under Settings → API Keys.
      </P>
      <div className="overflow-x-auto rounded-xl border border-white/[0.08] bg-[#080c0a] px-[18px] py-4 font-mono text-[12.5px] text-acc-text">
        Authorization: Bearer YOUR_API_KEY
      </div>

      <Verb method="POST" path="/v1/scans" />
      <P>Submit a smart contract for security scanning.</P>
      <Block>
        {'{\n  '}
        <K>&quot;name&quot;</K>: <V>&quot;My Protocol&quot;</V>,{'\n  '}
        <K>&quot;chain&quot;</K>: <V>&quot;evm&quot;</V>,{'\n  '}
        <K>&quot;contract_code&quot;</K>: <V>&quot;pragma solidity ^0.8.0;…&quot;</V>,{'\n  '}
        <K>&quot;format&quot;</K>: <V>&quot;json&quot;</V>
        {'\n}'}
      </Block>
      <H3>Response — 202 Accepted</H3>
      <Block>
        {'{\n  '}
        <K>&quot;scan_id&quot;</K>: <V>&quot;scan_1a2b3c4d5e6f7g8h&quot;</V>,{'\n  '}
        <K>&quot;status&quot;</K>: <V>&quot;pending&quot;</V>,{'\n  '}
        <K>&quot;polling_url&quot;</K>: <V>&quot;/v1/scans/scan_1a2b3c4d5e6f7g8h&quot;</V>
        {'\n}'}
      </Block>

      <Verb method="GET" path="/v1/scans/:id" />
      <P>Poll for scan results. Use this to check if the scan is complete and retrieve the report.</P>
      <Block>
        {'{\n  '}
        <K>&quot;scan_id&quot;</K>: <V>&quot;scan_1a2b3c4d5e6f7g8h&quot;</V>,{'\n  '}
        <K>&quot;status&quot;</K>: <V>&quot;complete&quot;</V>,{'\n  '}
        <K>&quot;findings&quot;</K>: {'{ '}
        <K>&quot;critical&quot;</K>: <span className="text-[#ef4444]">2</span>,{' '}
        <K>&quot;high&quot;</K>: <span className="text-[#fbbf24]">5</span>,{' '}
        <K>&quot;medium&quot;</K>: <span className="text-[#818cf8]">3</span>,{' '}
        <K>&quot;low&quot;</K>: <V>1</V>
        {' },\n  '}
        <K>&quot;report_url&quot;</K>: <V>&quot;/v1/scans/scan_1a2b3c…/report&quot;</V>
        {'\n}'}
      </Block>
      <H3>Polling recommendations</H3>
      <div className="flex flex-col gap-[7px]">
        {[
          'Poll every 2–3 seconds for typical scans (5–15 minutes)',
          'Implement exponential backoff after 10 failed polls',
          'Set a maximum timeout of 30 minutes per scan',
          'Store scan_id for auditing and historical analysis',
        ].map((s) => (
          <span key={s} className="text-[13px] text-sec">• {s}</span>
        ))}
      </div>

      <H2>Rate limits</H2>
      <Table
        cols="1fr 1fr 1fr"
        head={['Plan', 'Requests/hour', 'Concurrent scans']}
        rows={[
          [<span key="p" className="text-[#d7e2da]">Starter</span>, '10', '1'],
          [<span key="p" className="text-[#d7e2da]">Professional</span>, '100', '5'],
          [<span key="p" className="text-[#d7e2da]">Enterprise</span>, 'Unlimited', 'Unlimited'],
        ]}
      />
      <H3>Rate limit headers</H3>
      <Block>{'X-RateLimit-Limit: 100\nX-RateLimit-Remaining: 87\nX-RateLimit-Reset: 1623771600'}</Block>
    </article>
  )
}

function CiCd() {
  return (
    <article>
      <H1>CI/CD integration</H1>
      <Lede>
        Integrate Truent security scanning into your deployment pipeline to catch vulnerabilities early.
      </Lede>

      <H2>GitHub Actions</H2>
      <Block>
        <K>name</K>: Truent Security{'\n'}
        <K>on</K>: [push, pull_request]{'\n'}
        <K>jobs</K>:{'\n  '}
        <K>security</K>:{'\n    '}
        <K>runs-on</K>: ubuntu-latest{'\n    '}
        <K>steps</K>:{'\n      - '}
        <K>uses</K>: actions/checkout@v3{'\n      - '}
        <K>run</K>: <V>cargo install truent-cli</V>
        {'\n      - '}
        <K>run</K>: <V>truent check . --chain evm --format json</V>
      </Block>

      <H2>GitLab CI</H2>
      <Block>
        <K>security-scan</K>:{'\n  '}
        <K>image</K>: rust:latest{'\n  '}
        <K>script</K>:{'\n    - '}
        <V>cargo install truent-cli</V>
        {'\n    - '}
        <V>truent check . --chain evm --format json</V>
      </Block>

      <H2>Failing on findings</H2>
      <P>Use exit codes to gate deployments:</P>
      <Table
        cols="0.4fr 3fr"
        rows={[
          [<span key="a" className="font-mono text-[13px] font-semibold text-acc-text">0</span>, 'Success — pipeline proceeds'],
          [<span key="b" className="font-mono text-[13px] font-semibold text-[#fbbf24]">2</span>, 'High findings — fail the build'],
          [<span key="c" className="font-mono text-[13px] font-semibold text-[#ef4444]">3</span>, 'Critical — block the deploy'],
        ]}
      />

      <H2>Uploading artifacts</H2>
      <P>Store reports for audit trails and compliance:</P>
      <Block>
        {'- '}
        <K>uses</K>: actions/upload-artifact@v3{'\n  '}
        <K>with</K>:{'\n    '}
        <K>name</K>: <V>truent-report</V>
        {'\n    '}
        <K>path</K>: <V>report.json</V>
        {'\n    '}
        <K>retention-days</K>: <V>90</V>
      </Block>
    </article>
  )
}

function Reports() {
  const fields = [
    ['Title', 'Brief name of the vulnerability'],
    ['Severity', 'Critical → Info'],
    ['File', 'Source file path'],
    ['Lines', 'Where the issue occurs'],
    ['Invariant', 'Which invariant was violated'],
    ['Impact', 'Consequences if unfixed'],
    ['PoC', 'Runnable reproduction trace'],
    ['Remediation', 'Specific steps to fix'],
  ]

  return (
    <article>
      <H1>Audit report guide</H1>
      <Lede>
        Learn how to read, interpret, and share Truent security audit reports with your team.
      </Lede>

      <H2>Report structure</H2>
      <div className="grid gap-3 md:grid-cols-3">
        {[
          { n: '01', title: 'Executive summary', desc: 'High-level overview for non-technical stakeholders: finding counts by severity, scan date, key risk assessment.' },
          { n: '02', title: 'Findings section', desc: 'Detailed technical findings by severity, each with file location, line numbers, invariant violated, and remediation.' },
          { n: '03', title: 'Coverage appendix', desc: 'Complete list of every invariant checked during the scan, with pass/fail status and coverage statistics.' },
        ].map((s) => (
          <div key={s.n} className="rounded-2xl border border-hair bg-white/[0.02] p-6">
            <div className="mb-3 font-mono text-[22px] text-[#8fdcb2]/40">{s.n}</div>
            <h3 className="m-0 mb-2 text-[15px] font-medium text-text">{s.title}</h3>
            <p className="m-0 text-[12.5px] leading-[1.65] text-sec">{s.desc}</p>
          </div>
        ))}
      </div>

      <H2>Severity definitions</H2>
      <SeverityTable />

      <H2>Reading a finding</H2>
      <P>Each finding contains standardized fields to help your team understand the vulnerability:</P>
      <div className="grid gap-2.5 sm:grid-cols-2">
        {fields.map(([k, v]) => (
          <div
            key={k}
            className="flex gap-3 rounded-[10px] border border-white/[0.06] bg-white/[0.02] px-4 py-3 text-[13px] leading-[1.65] text-sec"
          >
            <strong className="min-w-[96px] font-medium text-text">{k}</strong>
            {v}
          </div>
        ))}
      </div>

      <H2>Exporting reports</H2>
      <div className="grid gap-3 md:grid-cols-3">
        {[
          { t: 'PDF', d: 'Professional, printable report for auditors and stakeholders. Default output format.' },
          { t: 'JSON', d: 'Machine-readable for CI/CD, aggregation, and custom tooling. Use it to gate deployments.' },
          { t: 'HTML', d: 'Interactive report viewable in any browser, with searchable findings and collapsible sections.' },
        ].map((f) => (
          <div key={f.t} className="rounded-2xl border border-hair bg-white/[0.02] p-6">
            <h3 className="m-0 mb-2 text-[15px] font-medium text-text">{f.t}</h3>
            <p className="m-0 text-[12.5px] leading-[1.65] text-sec">{f.d}</p>
          </div>
        ))}
      </div>
    </article>
  )
}

// ─────────────────────────────────────────────────────────────────────────────
// Page
// ─────────────────────────────────────────────────────────────────────────────

export default function DocsPage() {
  const [page, setPage] = useState<PageId>('overview')

  // Deep links (/docs#cli) are honoured on mount and on back/forward.
  useEffect(() => {
    const apply = () => {
      const h = window.location.hash.replace('#', '')
      if (PAGES.some((p) => p.id === h)) setPage(h as PageId)
    }
    apply()
    window.addEventListener('hashchange', apply)
    return () => window.removeEventListener('hashchange', apply)
  }, [])

  const go = (id: PageId) => {
    setPage(id)
    window.history.replaceState(null, '', `#${id}`)
    window.scrollTo({ top: 0, behavior: 'smooth' })
  }

  return (
    <PageShell glow="radial-gradient(1100px 500px at 50% -120px, rgba(52,211,153,0.12), rgba(6,9,8,0) 60%)">
      <MarketingNav />

      <div className="mx-auto grid max-w-[1200px] items-start gap-12 px-6 pt-14 md:grid-cols-[210px_1fr]">
        {/* ─── Sidebar ─── */}
        <aside className="md:sticky md:top-24">
          <div className="mb-[18px] hidden overflow-hidden opacity-30 md:block">
            <AsciiLogo className="text-[3.2px] !leading-[1.08]" />
          </div>
          <div className="mb-3.5 font-mono text-[10px] uppercase tracking-[0.18em] text-[#5c665f]">
            Documentation
          </div>
          <nav className="flex flex-row flex-wrap gap-0.5 md:flex-col">
            {PAGES.map((p) => (
              <button
                key={p.id}
                onClick={() => go(p.id)}
                aria-current={page === p.id ? 'page' : undefined}
                className={`rounded-[9px] border-l-2 px-3 py-[9px] text-left text-[13px] transition-colors ${
                  page === p.id
                    ? 'border-acc-text bg-acc-text/[0.09] font-medium text-text'
                    : 'border-transparent text-[#8a948d] hover:text-text'
                }`}
              >
                {p.label}
              </button>
            ))}
          </nav>
          <div className="mt-7 flex flex-col gap-2.5 border-t border-white/[0.06] pt-5">
            <Link href="/library" className="text-[12.5px] text-sec transition-colors hover:text-text">
              Invariant Library ↗
            </Link>
            <a
              href="https://github.com/geekstrancend/Truent"
              target="_blank"
              rel="noopener noreferrer"
              className="text-[12.5px] text-sec transition-colors hover:text-text"
            >
              GitHub repo ↗
            </a>
          </div>
        </aside>

        {/* ─── Content ─── */}
        <main className="min-h-[70vh] pb-[90px]">
          {page === 'overview' && <Overview go={go} />}
          {page === 'getting-started' && <GettingStarted />}
          {page === 'cli' && <CliReference />}
          {page === 'ai' && <AiCoAuditor />}
          {page === 'api' && <RestApi />}
          {page === 'ci-cd' && <CiCd />}
          {page === 'reports' && <Reports />}

          <div className="mt-[70px] flex flex-wrap items-center justify-between gap-3.5 border-t border-white/[0.06] pt-7">
            <Link href="/library" className="text-[13px] text-sec transition-colors hover:text-text">
              ← Browse the invariant library
            </Link>
            <Link href="/contact" className="text-[13px] text-acc-text">
              Need help? Talk to us →
            </Link>
          </div>
        </main>
      </div>

      <SlimFooter omit={['Docs']} />
    </PageShell>
  )
}
