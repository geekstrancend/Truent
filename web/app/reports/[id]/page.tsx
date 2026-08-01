'use client'

import { useState } from 'react'
import Link from 'next/link'
import { Download, Share2, ChevronDown, ChevronUp, ArrowLeft, CheckCircle2, Circle, AlertTriangle } from 'lucide-react'
import { AppShell } from '@/components/layout/AppShell'
import { Button } from '@/components/ui/Button'
import { SeverityBadge } from '@/components/ui/SeverityBadge'
import clsx from 'clsx'

type Severity = 'critical' | 'high' | 'medium' | 'low'
type FindingStatus = 'open' | 'acknowledged' | 'resolved'

interface Finding {
  id: string
  severity: Severity
  title: string
  description: string
  location: string
  impact: string
  recommendation: string
  status: FindingStatus
  codeSnippet?: string
}

const FINDINGS: Finding[] = [
  {
    id: 'TRU-001',
    severity: 'critical',
    title: 'Reentrancy vulnerability in withdrawAll()',
    description: 'The withdrawAll() function transfers Ether to msg.sender before updating the internal balance mapping. An attacker can deploy a contract that calls back into withdrawAll() on receipt, draining the vault before the balance is zeroed.',
    location: 'contracts/Vault.sol · Line 42',
    impact: 'Complete drain of the Vault contract ETH balance in a single transaction.',
    recommendation: 'Apply the Checks-Effects-Interactions pattern: update balances[msg.sender] = 0 before the external call. Alternatively, add OpenZeppelin\'s ReentrancyGuard nonReentrant modifier.',
    status: 'open',
    codeSnippet: `// ❌ Vulnerable
function withdrawAll() external {
    uint256 amount = balances[msg.sender];
    require(amount > 0, "Nothing to withdraw");
    (bool ok, ) = msg.sender.call{value: amount}("");  // external call first
    require(ok, "Transfer failed");
    balances[msg.sender] = 0;  // state updated after
}

// ✅ Fixed
function withdrawAll() external nonReentrant {
    uint256 amount = balances[msg.sender];
    require(amount > 0, "Nothing to withdraw");
    balances[msg.sender] = 0;  // state updated first
    (bool ok, ) = msg.sender.call{value: amount}("");
    require(ok, "Transfer failed");
}`,
  },
  {
    id: 'TRU-002',
    severity: 'high',
    title: 'Unchecked return value on ERC-20 transfer()',
    description: 'The transferFunds() function calls token.transfer() without checking the boolean return value. Some tokens (USDT, BNB) return false on failure instead of reverting, meaning silent failures can leave the contract in an inconsistent state.',
    location: 'contracts/Token.sol · Line 108',
    impact: 'Silent transfer failures can be exploited to manipulate accounting without actual token movement.',
    recommendation: 'Replace token.transfer() with SafeERC20.safeTransfer() from OpenZeppelin, which reverts on false returns and handles non-standard tokens.',
    status: 'acknowledged',
    codeSnippet: `// ❌ Vulnerable
token.transfer(recipient, amount);

// ✅ Fixed
using SafeERC20 for IERC20;
token.safeTransfer(recipient, amount);`,
  },
  {
    id: 'TRU-003',
    severity: 'high',
    title: 'Missing oracle price staleness check',
    description: 'The price feed from Chainlink is fetched without validating the updatedAt timestamp. If the oracle heartbeat fails or the feed becomes stale, the protocol will continue operating on outdated prices.',
    location: 'contracts/PriceOracle.sol · Line 67',
    impact: 'Stale prices could allow undercollateralised borrows or unfair liquidations.',
    recommendation: 'After calling latestRoundData(), verify that block.timestamp - updatedAt <= HEARTBEAT_INTERVAL where HEARTBEAT_INTERVAL is the documented feed heartbeat (e.g. 3600 for 1h feeds).',
    status: 'open',
    codeSnippet: `// ❌ Vulnerable
(, int256 price,,,) = priceFeed.latestRoundData();
return uint256(price);

// ✅ Fixed
(, int256 price,, uint256 updatedAt,) = priceFeed.latestRoundData();
require(block.timestamp - updatedAt <= HEARTBEAT, "Stale price");
require(price > 0, "Invalid price");
return uint256(price);`,
  },
  {
    id: 'TRU-004',
    severity: 'medium',
    title: 'setFee() accepts unbounded values',
    description: 'The setFee() function allows the owner to set the protocol fee to any value including 100% or more. There is no upper bound check.',
    location: 'contracts/Pool.sol · Line 55',
    impact: 'A compromised or malicious owner can extract all user funds via 100% fees.',
    recommendation: 'Add require(newFee <= MAX_FEE, "Fee too high") where MAX_FEE is a reasonable constant like 1000 (10% in basis points).',
    status: 'resolved',
    codeSnippet: `// ❌ Vulnerable
function setFee(uint256 newFee) external onlyOwner {
    fee = newFee;
}

// ✅ Fixed
uint256 public constant MAX_FEE = 1000; // 10%
function setFee(uint256 newFee) external onlyOwner {
    require(newFee <= MAX_FEE, "Fee exceeds maximum");
    emit FeeUpdated(fee, newFee);
    fee = newFee;
}`,
  },
  {
    id: 'TRU-005',
    severity: 'low',
    title: 'Missing event emission in setFee()',
    description: 'State-changing governance functions should emit events so that off-chain monitors and users can track changes.',
    location: 'contracts/Pool.sol · Line 55',
    impact: 'Silent fee changes reduce protocol transparency and auditability.',
    recommendation: 'Add event FeeUpdated(uint256 oldFee, uint256 newFee) and emit it in setFee().',
    status: 'open',
  },
]

const STATUS_CONFIG: Record<FindingStatus, { label: string; cls: string; icon: React.ReactNode }> = {
  open:         { label: 'Open',         cls: 'text-critical bg-critical/10 border-critical/20', icon: <Circle size={12} /> },
  acknowledged: { label: 'Acknowledged', cls: 'text-high bg-high/10 border-high/20',             icon: <AlertTriangle size={12} /> },
  resolved:     { label: 'Resolved',     cls: 'text-low bg-low/10 border-low/20',               icon: <CheckCircle2 size={12} /> },
}

function FindingCard({ finding }: { finding: Finding }) {
  const [expanded, setExpanded] = useState(finding.severity === 'critical' || finding.severity === 'high')
  const status = STATUS_CONFIG[finding.status]

  return (
    <div className={clsx(
      'bg-panel border rounded-card overflow-hidden transition-all',
      finding.status === 'resolved' ? 'border-hair opacity-70' : 'border-hair',
    )}>
      {/* Severity bar */}
      <div className={clsx('h-0.5', {
        'bg-critical': finding.severity === 'critical',
        'bg-high': finding.severity === 'high',
        'bg-medium': finding.severity === 'medium',
        'bg-low': finding.severity === 'low',
      })} />

      {/* Header */}
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full flex items-center justify-between px-6 py-4 text-left hover:bg-panel/40 transition-colors"
      >
        <div className="flex items-center gap-4 flex-1 min-w-0">
          <SeverityBadge level={finding.severity} />
          <div className="flex-1 min-w-0">
            <p className="text-body-md font-[600] text-text">{finding.title}</p>
            <p className="text-xs text-sec mt-0.5 font-mono">{finding.location}</p>
          </div>
        </div>
        <div className="flex items-center gap-3 flex-shrink-0 ml-4">
          <span className={clsx('inline-flex items-center gap-1 text-xs font-[600] px-2 py-0.5 rounded border', status.cls)}>
            {status.icon} {status.label}
          </span>
          <code className="text-xs text-sec font-mono">{finding.id}</code>
          {expanded ? <ChevronUp size={16} className="text-sec" /> : <ChevronDown size={16} className="text-sec" />}
        </div>
      </button>

      {/* Expanded content */}
      {expanded && (
        <div className="px-6 pb-6 border-t border-hair/50 pt-5 space-y-5">
          <div>
            <p className="text-label-sm text-sec mb-2">DESCRIPTION</p>
            <p className="text-body-md text-sec leading-6">{finding.description}</p>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
            <div>
              <p className="text-label-sm text-sec mb-2">IMPACT</p>
              <p className="text-body-md text-sec leading-6">{finding.impact}</p>
            </div>
            <div>
              <p className="text-label-sm text-sec mb-2">RECOMMENDATION</p>
              <p className="text-body-md text-sec leading-6">{finding.recommendation}</p>
            </div>
          </div>
          {finding.codeSnippet && (
            <div>
              <p className="text-label-sm text-sec mb-2">CODE</p>
              <pre className="bg-surface-2 border border-hair rounded-lg p-4 text-xs font-mono text-sec overflow-x-auto leading-5 whitespace-pre">
                {finding.codeSnippet}
              </pre>
            </div>
          )}
        </div>
      )}
    </div>
  )
}

export default function ReportDetailPage({ params }: { params: { id: string } }) {
  const reportId = params.id
  const [filter, setFilter] = useState<'all' | Severity>('all')

  const stats = {
    critical: FINDINGS.filter(f => f.severity === 'critical').length,
    high:     FINDINGS.filter(f => f.severity === 'high').length,
    medium:   FINDINGS.filter(f => f.severity === 'medium').length,
    low:      FINDINGS.filter(f => f.severity === 'low').length,
  }

  const open      = FINDINGS.filter(f => f.status === 'open').length
  const resolved  = FINDINGS.filter(f => f.status === 'resolved').length

  const visible = filter === 'all' ? FINDINGS : FINDINGS.filter(f => f.severity === filter)

  return (
    <AppShell currentPage="audits">
      <div className="max-w-4xl mx-auto p-6 lg:p-8 space-y-8">

        {/* Back link */}
        <Link href="/dashboard" className="inline-flex items-center gap-2 text-sec hover:text-text text-body-md transition-colors">
          <ArrowLeft size={16} /> Back to Dashboard
        </Link>

        {/* Header */}
        <div className="flex flex-col md:flex-row md:items-start justify-between gap-4">
          <div>
            <div className="flex items-center gap-3 mb-2">
              <span className="text-label-sm text-sec bg-panel border border-hair px-2 py-0.5 rounded font-mono">{reportId}</span>
              <span className="text-xs text-low bg-low/10 border border-low/20 px-2 py-0.5 rounded font-mono">COMPLETE</span>
            </div>
            <h1 className="font-display text-3xl font-[600] text-text mb-2">Security Audit Report</h1>
            <p className="text-body-md text-sec">Generated {new Date().toLocaleDateString('en-US', { month: 'long', day: 'numeric', year: 'numeric' })}</p>
          </div>
          <div className="flex gap-2 flex-shrink-0">
            <Button variant="secondary" size="sm" icon={<Share2 size={14} />} disabled title="Coming soon">Share</Button>
            <Button variant="primary" size="sm" icon={<Download size={14} />} disabled title="Coming soon">Export PDF</Button>
          </div>
        </div>

        {/* Summary stats */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-px bg-hair rounded-card overflow-hidden">
          {[
            { label: 'CRITICAL', count: stats.critical, color: 'text-critical', bg: 'bg-critical/5' },
            { label: 'HIGH',     count: stats.high,     color: 'text-high',     bg: 'bg-high/5' },
            { label: 'MEDIUM',   count: stats.medium,   color: 'text-medium',   bg: 'bg-medium/5' },
            { label: 'LOW',      count: stats.low,      color: 'text-low',      bg: 'bg-low/5' },
          ].map((item) => (
            <button
              key={item.label}
              onClick={() => setFilter(filter === item.label.toLowerCase() as Severity ? 'all' : item.label.toLowerCase() as Severity)}
              className={clsx(
                'p-5 text-center transition-colors',
                item.bg,
                filter === item.label.toLowerCase() ? 'ring-1 ring-inset ring-outline' : 'hover:bg-panel',
              )}
            >
              <div className={clsx('font-display text-4xl font-[700] mb-1', item.color)}>{item.count}</div>
              <div className="text-label-sm text-sec">{item.label}</div>
            </button>
          ))}
        </div>

        {/* Status bar */}
        <div className="bg-panel border border-hair rounded-card p-5">
          <div className="flex items-center justify-between mb-3">
            <span className="text-body-md text-sec">Remediation Progress</span>
            <span className="text-body-md text-text font-[600]">{resolved}/{FINDINGS.length} resolved</span>
          </div>
          <div className="w-full h-2 bg-panel rounded-full overflow-hidden">
            <div
              className="h-full bg-gradient-to-r from-medium to-low rounded-full transition-all duration-700"
              style={{ width: `${(resolved / FINDINGS.length) * 100}%` }}
            />
          </div>
          <div className="flex items-center gap-4 mt-3 text-xs">
            <span className="text-critical font-[600]">{open} open</span>
            <span className="text-low font-[600]">{resolved} resolved</span>
            <span className="text-high font-[600]">{FINDINGS.filter(f => f.status === 'acknowledged').length} acknowledged</span>
          </div>
        </div>

        {/* Filter tabs */}
        <div className="flex gap-1">
          {(['all', 'critical', 'high', 'medium', 'low'] as const).map((f) => (
            <button
              key={f}
              onClick={() => setFilter(f)}
              className={clsx(
                'px-4 py-1.5 rounded-lg text-sm font-[600] transition-colors capitalize',
                filter === f ? 'bg-panel text-text' : 'text-sec hover:text-text',
              )}
            >
              {f === 'all' ? `All (${FINDINGS.length})` : `${f.charAt(0).toUpperCase() + f.slice(1)} (${stats[f]})`}
            </button>
          ))}
        </div>

        {/* Findings list */}
        <div className="space-y-3">
          {visible.map((finding) => (
            <FindingCard key={finding.id} finding={finding} />
          ))}
        </div>
      </div>
    </AppShell>
  )
}
