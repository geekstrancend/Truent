'use client'

import { useState } from 'react'
import { AppShell } from '@/components/layout/AppShell'
import { ScanModal } from '@/components/ui/ScanModal'

interface Scan {
  id: string
  project: string
  chain: 'EVM' | 'Solana' | 'Arbitrum' | 'Base'
  date: string
  findings: { critical: number; high: number; medium: number; low: number }
  status: 'complete' | 'scanning' | 'failed'
  duration: string
}

interface Activity {
  type: 'finding' | 'shared' | 'complete' | 'updated' | 'failed'
  title: string
  description: string
  time: string
}

const SCANS: Scan[] = [
  { id: 'TRU-2026-042', project: 'Dexalot Contracts', chain: 'EVM', date: 'Jul 7, 2026', findings: { critical: 3, high: 6, medium: 7, low: 5 }, status: 'complete', duration: '4m 12s' },
  { id: 'TRU-2026-041', project: 'Circle-Pay BCH', chain: 'Solana', date: 'Jul 6, 2026', findings: { critical: 5, high: 7, medium: 6, low: 4 }, status: 'complete', duration: '6m 55s' },
  { id: 'TRU-2026-040', project: 'Vault Core V2', chain: 'Arbitrum', date: 'Jul 5, 2026', findings: { critical: 0, high: 0, medium: 2, low: 12 }, status: 'complete', duration: '2m 08s' },
  { id: 'TRU-2026-043', project: 'Protocol X LendingPool', chain: 'Base', date: 'Jul 8, 2026', findings: { critical: 0, high: 0, medium: 0, low: 0 }, status: 'scanning', duration: '–' },
]

const ACTIVITY: Activity[] = [
  { type: 'finding', title: 'Critical confirmed', description: 'Reentrancy vulnerability in Dexalot signature logic', time: '2m ago' },
  { type: 'shared', title: 'Report shared', description: 'Security disclosure TRU-2026-041 sent to Circle-Pay team', time: '45m ago' },
  { type: 'complete', title: 'Scan complete', description: 'Circle-Pay BCH finished — 22 findings across 3 contracts', time: '3h ago' },
  { type: 'updated', title: 'Library updated', description: '47 new patterns synced from global exploit database', time: '5h ago' },
  { type: 'complete', title: 'Scan complete', description: 'Vault Core V2 — Clean result, 14 low-risk observations', time: '1d ago' },
]

const METRICS = [
  { label: 'Total scans', value: '43', delta: '+8 this month', icon: '▤' },
  { label: 'Critical findings', value: '8', delta: '-3 resolved', icon: '⚠' },
  { label: 'Protocols monitored', value: '12', delta: '+2 this month', icon: '⬡' },
  { label: 'Avg scan time', value: '4m 20s', delta: '↓ 18% faster', icon: '◔' },
]

const ACTIVITY_ICON: Record<Activity['type'], { cls: string; symbol: string }> = {
  finding: { cls: 'bg-[#ef4444]/20 text-[#ef4444]', symbol: '!' },
  shared: { cls: 'bg-[#818cf8]/20 text-[#818cf8]', symbol: '↗' },
  complete: { cls: 'bg-[#4ade80]/20 text-[#4ade80]', symbol: '✓' },
  updated: { cls: 'bg-[#fbbf24]/20 text-[#fbbf24]', symbol: '↺' },
  failed: { cls: 'bg-[#ef4444]/20 text-[#ef4444]', symbol: '✗' },
}

/** Coloured finding counts, or a dash while a scan is still running. */
function Findings({ scan }: { scan: Scan }) {
  if (scan.status === 'scanning') return <span className="text-[#5c665f]">—</span>
  const { critical, high, medium, low } = scan.findings
  const parts: Array<[number, string]> = [
    [critical, '#ef4444'],
    [high, '#fbbf24'],
    [medium, '#818cf8'],
    [low, '#4ade80'],
  ]
  return (
    <span className="flex gap-1.5">
      {parts.map(([n, c], i) => (
        <span key={i} style={{ color: n > 0 ? c : '#3d453f' }}>{n}</span>
      ))}
    </span>
  )
}

export default function DashboardPage() {
  const [showScanModal, setShowScanModal] = useState(false)

  return (
    <AppShell currentPage="dashboard" onNewScan={() => setShowScanModal(true)}>
      <div className="max-w-[1080px] px-6 pb-16 pt-8 lg:px-9">
        {/* ─── Header ─── */}
        <div className="mb-8 flex items-start justify-between gap-5">
          <div>
            <h1 className="m-0 text-[30px] font-normal tracking-[-0.02em] text-[#f2f6f2]">Dashboard</h1>
            <p className="m-0 mt-2 text-[13.5px] text-sec">
              Welcome back, Alex. Here&apos;s your security overview.
            </p>
          </div>
          <button
            onClick={() => setShowScanModal(true)}
            className="inline-flex flex-shrink-0 items-center gap-2.5 rounded-full bg-[#eef2ef] py-[5px] pl-[18px] pr-[5px] text-[13px] font-semibold text-[#0a0d0b] transition-colors hover:bg-white"
          >
            New scan
            <span className="flex h-7 w-7 items-center justify-center rounded-full bg-acc-text text-[14px] text-on-acc">
              +
            </span>
          </button>
        </div>

        {/* ─── Metrics: single hairline grid ─── */}
        <div className="relative mb-[34px]">
          <div
            className="pointer-events-none absolute left-[34%] top-[38%] h-[180px] w-[280px]"
            style={{ background: 'radial-gradient(closest-side,rgba(52,211,153,0.1),transparent)' }}
          />
          <div className="relative grid grid-cols-2 overflow-hidden rounded-[18px] border border-white/[0.06] lg:grid-cols-4">
            {METRICS.map((m, i) => (
              <div
                key={m.label}
                className={`p-5 ${i < 3 ? 'lg:border-r lg:border-white/[0.06]' : ''} ${
                  i % 2 === 0 ? 'border-r border-white/[0.06] lg:border-r' : ''
                } ${i < 2 ? 'border-b border-white/[0.06] lg:border-b-0' : ''}`}
              >
                <div className="mb-3 flex items-center justify-between">
                  <span className="text-[12px] text-[#748078]">{m.label}</span>
                  <span className="text-[14px]">{m.icon}</span>
                </div>
                <div className="text-[31px] font-medium tracking-[-0.02em] text-text">{m.value}</div>
                <div className="mt-1.5 font-mono text-[10.5px] text-acc-text">{m.delta}</div>
              </div>
            ))}
          </div>
        </div>

        <div className="grid items-start gap-3.5 lg:grid-cols-[1.9fr_1fr]">
          {/* ─── Recent scans ─── */}
          <div className="overflow-hidden rounded-[18px] border border-hair bg-white/[0.02]">
            <div className="flex items-center justify-between border-b border-white/[0.06] px-[22px] py-[18px]">
              <h2 className="m-0 text-[16px] font-medium text-text">Recent scans</h2>
              <span className="font-mono text-[10.5px] tracking-[0.1em] text-acc-text">
                {SCANS.length} TOTAL
              </span>
            </div>
            <div className="overflow-x-auto">
              <div className="min-w-[540px]">
                <div className="grid grid-cols-[1.6fr_0.7fr_1fr_0.9fr_0.9fr] border-b border-white/[0.05] px-[22px] py-[11px] font-mono text-[9.5px] uppercase tracking-[0.14em] text-[#4d564f]">
                  <span>Project</span>
                  <span>Chain</span>
                  <span>Findings</span>
                  <span>Date</span>
                  <span>Status</span>
                </div>
                {SCANS.map((scan) => (
                  <div
                    key={scan.id}
                    className="grid grid-cols-[1.6fr_0.7fr_1fr_0.9fr_0.9fr] items-center border-b border-white/[0.04] px-[22px] py-[15px] last:border-b-0"
                  >
                    <div>
                      <div className="text-[13px] font-medium text-text">{scan.project}</div>
                      <div className="mt-[3px] font-mono text-[10px] text-[#5c665f]">{scan.id}</div>
                    </div>
                    <span className="font-mono text-[10px] text-[#8fa398]">{scan.chain}</span>
                    <span className="font-mono text-[11px]">
                      <Findings scan={scan} />
                    </span>
                    <span className="whitespace-nowrap text-[12px] text-[#748078]">{scan.date}</span>
                    <span
                      className={`w-fit rounded-[5px] border px-2 py-[3px] font-mono text-[9.5px] tracking-[0.1em] ${
                        scan.status === 'complete'
                          ? 'border-acc-text/25 bg-acc-text/10 text-acc-text'
                          : scan.status === 'scanning'
                            ? 'border-[#fbbf24]/25 bg-[#fbbf24]/10 text-[#fbbf24]'
                            : 'border-[#ef4444]/25 bg-[#ef4444]/10 text-[#ef4444]'
                      }`}
                    >
                      {scan.status.toUpperCase()}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* ─── Activity ─── */}
          <div className="overflow-hidden rounded-[18px] border border-hair bg-white/[0.02]">
            <div className="border-b border-white/[0.06] px-5 py-[18px]">
              <h2 className="m-0 text-[16px] font-medium text-text">Activity</h2>
            </div>
            {ACTIVITY.map((a, i) => {
              const { cls, symbol } = ACTIVITY_ICON[a.type]
              return (
                <div key={i} className="flex gap-3 border-b border-white/[0.04] px-5 py-[15px] last:border-b-0">
                  <span
                    className={`flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full text-xs font-bold ${cls}`}
                  >
                    {symbol}
                  </span>
                  <div className="min-w-0">
                    <div className="text-[12.5px] font-medium text-text">{a.title}</div>
                    <p className="m-0 mt-[3px] text-[11.5px] leading-[1.55] text-[#748078]">
                      {a.description}
                    </p>
                    <div className="mt-[5px] font-mono text-[9.5px] text-[#4d564f]">{a.time}</div>
                  </div>
                </div>
              )
            })}
          </div>
        </div>
      </div>

      <ScanModal isOpen={showScanModal} onClose={() => setShowScanModal(false)} />
    </AppShell>
  )
}
