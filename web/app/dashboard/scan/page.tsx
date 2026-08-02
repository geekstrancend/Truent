'use client'

import { useState } from 'react'
import { Upload, Github, Code, Zap, Check, AlertCircle } from 'lucide-react'
import { AppShell } from '@/components/layout/AppShell'
import { Button } from '@/components/ui/Button'
import { SeverityBadge } from '@/components/ui/SeverityBadge'

type SubmissionMethod = 'code' | 'file' | 'github'

interface ScanResult {
  vulnerabilities: {
    critical: number
    high: number
    medium: number
    low: number
  }
  findings: Array<{
    severity: string
    title: string
    description: string
    line?: number
  }>
  isScanning: boolean
}

export default function ScanPage() {
  const [method, setMethod] = useState<SubmissionMethod>('code')
  const [code, setCode] = useState('')
  const [githubUrl, setGithubUrl] = useState('')
  const [fileName, setFileName] = useState<string | null>(null)
  const [language, setLanguage] = useState('solidity')
  const [isScanning, setIsScanning] = useState(false)
  const [scanResult, setScanResult] = useState<ScanResult | null>(null)
  const [formError, setFormError] = useState('')

  const resetForm = () => {
    setCode('')
    setGithubUrl('')
    setFileName(null)
    setScanResult(null)
    setFormError('')
  }

  const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (file) {
      setFileName(file.name)
      const reader = new FileReader()
      reader.onload = (event) => {
        setCode(event.target?.result as string)
      }
      reader.readAsText(file)
    }
  }

  const handleScan = async () => {
    setFormError('')
    if (!code && method === 'code') {
      setFormError('Please enter code to scan')
      return
    }
    if (!githubUrl && method === 'github') {
      setFormError('Please enter a GitHub URL')
      return
    }

    setIsScanning(true)
    try {
      const response = await fetch('/api/analyze', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          code: method === 'code' || method === 'file' ? code : undefined,
          githubUrl: method === 'github' ? githubUrl : undefined,
          language,
        }),
      })

      if (!response.ok) {
        throw new Error('Scan failed')
      }

      const data = await response.json()
      setScanResult({
        vulnerabilities: {
          critical: data.vulnerabilities?.filter((v: any) => v.severity === 'critical').length || 0,
          high: data.vulnerabilities?.filter((v: any) => v.severity === 'high').length || 0,
          medium: data.vulnerabilities?.filter((v: any) => v.severity === 'medium').length || 0,
          low: data.vulnerabilities?.filter((v: any) => v.severity === 'low').length || 0,
        },
        findings: data.vulnerabilities || [],
        isScanning: false,
      })
    } catch (error) {
      setFormError('Error scanning code. Please try again.')
      console.error('Scan error:', error)
    } finally {
      setIsScanning(false)
    }
  }

  return (
    <AppShell currentPage="dashboard" onNewScan={resetForm}>
      <div className="max-w-6xl mx-auto p-6 space-y-8">
        {/* Header */}
        <div className="space-y-3">
          <h1 className="text-4xl font-[700] text-text font-display">
            Smart Contract Analyzer
          </h1>
          <p className="text-body-lg text-sec">
            Submit your code for comprehensive security analysis powered by AI and our invariant library.
          </p>
        </div>

        {/* Method Selection */}
        <div className="bg-panel border border-hair rounded-lg p-6">
          <h2 className="text-lg font-[600] text-text mb-4">Choose Submission Method</h2>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            {/* Direct Code Input */}
            <button
              onClick={() => setMethod('code')}
              className={`p-4 rounded-lg border-2 transition ${
                method === 'code'
                  ? 'border-brand bg-brand/10'
                  : 'border-hair hover:border-brand'
              }`}
            >
              <Code className="w-8 h-8 text-acc-text mb-2" />
              <h3 className="font-[600] text-text mb-1">Direct Input</h3>
              <p className="text-sm text-sec">Paste code directly</p>
            </button>

            {/* File Upload */}
            <button
              onClick={() => setMethod('file')}
              className={`p-4 rounded-lg border-2 transition ${
                method === 'file'
                  ? 'border-brand bg-brand/10'
                  : 'border-hair hover:border-brand'
              }`}
            >
              <Upload className="w-8 h-8 text-acc-text mb-2" />
              <h3 className="font-[600] text-text mb-1">Upload File</h3>
              <p className="text-sm text-sec">Upload contract files</p>
            </button>

            {/* GitHub Integration */}
            <button
              onClick={() => setMethod('github')}
              className={`p-4 rounded-lg border-2 transition ${
                method === 'github'
                  ? 'border-brand bg-brand/10'
                  : 'border-hair hover:border-brand'
              }`}
            >
              <Github className="w-8 h-8 text-acc-text mb-2" />
              <h3 className="font-[600] text-text mb-1">GitHub Link</h3>
              <p className="text-sm text-sec">Connect GitHub repo</p>
            </button>
          </div>
        </div>

        {/* Input Area */}
        <div className="bg-panel border border-hair rounded-lg p-6 space-y-4">
          {formError && (
            <div className="flex items-center gap-2 p-3 bg-critical-bg border border-critical-border rounded-lg">
              <AlertCircle size={16} className="text-critical flex-shrink-0" />
              <span className="text-sm text-critical">{formError}</span>
            </div>
          )}
          {method === 'code' && (
            <>
              <div>
                <label htmlFor="scan-language-code" className="block text-sm font-medium text-text mb-2">
                  Language
                </label>
                <select
                  id="scan-language-code"
                  value={language}
                  onChange={(e) => setLanguage(e.target.value)}
                  className="w-full px-4 py-2 bg-surface-2 text-text rounded-lg border border-hair focus:outline-none focus:border-brand"
                >
                  <option value="solidity">Solidity</option>
                  <option value="rust">Rust (Move, Anchor)</option>
                  <option value="cairo">Cairo</option>
                </select>
              </div>
              <div>
                <label htmlFor="scan-code-input" className="block text-sm font-medium text-text mb-2">
                  Smart Contract Code
                </label>
                <textarea
                  id="scan-code-input"
                  value={code}
                  onChange={(e) => setCode(e.target.value)}
                  placeholder="Paste your smart contract code here..."
                  className="w-full h-96 px-4 py-3 bg-surface-2 text-text placeholder-on-surface-variant rounded-lg border border-hair focus:outline-none focus:border-brand font-mono text-sm resize-none"
                  maxLength={100000}
                />
                <p className="text-xs text-sec mt-1">
                  {code.length.toLocaleString()} / 100,000 characters
                </p>
              </div>
            </>
          )}

          {method === 'file' && (
            <>
              <div>
                <label htmlFor="scan-language-file" className="block text-sm font-medium text-text mb-2">
                  Language
                </label>
                <select
                  id="scan-language-file"
                  value={language}
                  onChange={(e) => setLanguage(e.target.value)}
                  className="w-full px-4 py-2 bg-surface-2 text-text rounded-lg border border-hair focus:outline-none focus:border-brand"
                >
                  <option value="solidity">Solidity</option>
                  <option value="rust">Rust</option>
                  <option value="cairo">Cairo</option>
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium text-text mb-2">
                  Upload File
                </label>
                <div className="border-2 border-dashed border-hair rounded-lg p-8 text-center hover:border-brand transition">
                  <input
                    type="file"
                    onChange={handleFileUpload}
                    className="hidden"
                    id="file-upload"
                    accept=".sol,.rs,.cairo,.txt"
                  />
                  <label htmlFor="file-upload" className="cursor-pointer">
                    <Upload className="w-8 h-8 text-acc-text mx-auto mb-2" />
                    <p className="text-text font-medium">
                      {fileName || 'Click to upload or drag and drop'}
                    </p>
                    <p className="text-sm text-sec mt-1">
                      .sol, .rs, .cairo, or .txt files up to 10 MB
                    </p>
                  </label>
                </div>
              </div>
            </>
          )}

          {method === 'github' && (
            <>
              <div>
                <label htmlFor="scan-github-url" className="block text-sm font-medium text-text mb-2">
                  GitHub Repository URL
                </label>
                <input
                  id="scan-github-url"
                  type="text"
                  value={githubUrl}
                  onChange={(e) => setGithubUrl(e.target.value)}
                  placeholder="https://github.com/username/repository"
                  className="w-full px-4 py-2 bg-surface-2 text-text placeholder-on-surface-variant rounded-lg border border-hair focus:outline-none focus:border-brand"
                />
              </div>
              <div>
                <label htmlFor="scan-github-branch" className="block text-sm font-medium text-text mb-2">
                  Target Branch or Tag
                </label>
                <input
                  id="scan-github-branch"
                  type="text"
                  placeholder="main (default: main)"
                  className="w-full px-4 py-2 bg-surface-2 text-text placeholder-on-surface-variant rounded-lg border border-hair focus:outline-none focus:border-brand"
                />
              </div>
            </>
          )}

          <Button
            className="w-full"
            size="lg"
            onClick={handleScan}
            disabled={isScanning || (method === 'code' && !code) || (method === 'github' && !githubUrl)}
          >
            {isScanning ? 'Scanning...' : 'Start Analysis'}
          </Button>
        </div>

        {/* Scan Results */}
        {scanResult && (
          <div className="bg-panel border border-hair rounded-lg p-6 space-y-6">
            <h2 className="text-2xl font-[700] text-text font-display">
              Scan Results
            </h2>

            {/* Severity Grid */}
            <div className="grid grid-cols-4 gap-4">
              {[
                { label: 'CRITICAL', count: scanResult.vulnerabilities.critical, color: 'bg-critical' },
                { label: 'HIGH', count: scanResult.vulnerabilities.high, color: 'bg-high' },
                { label: 'MEDIUM', count: scanResult.vulnerabilities.medium, color: 'bg-medium' },
                { label: 'LOW', count: scanResult.vulnerabilities.low, color: 'bg-low' },
              ].map((item) => (
                <div
                  key={item.label}
                  className="bg-panel border border-hair rounded-lg p-4 text-center"
                >
                  <div className={`text-4xl font-[700] mb-2 text-text`}>
                    {item.count}
                  </div>
                  <div className="text-sm text-sec">{item.label}</div>
                </div>
              ))}
            </div>

            {/* Findings List */}
            {scanResult.findings.length > 0 && (
              <div className="space-y-3">
                <h3 className="text-lg font-[600] text-text">Detailed Findings</h3>
                {scanResult.findings.map((finding, idx) => (
                  <div
                    key={idx}
                    className="bg-panel border border-hair rounded-lg p-4 flex gap-4 items-start"
                  >
                    <SeverityBadge level={finding.severity as any} />
                    <div className="flex-1">
                      <h4 className="font-[600] text-text">{finding.title}</h4>
                      <p className="text-sm text-sec mt-1">{finding.description}</p>
                      {finding.line && (
                        <p className="text-xs text-sec mt-2">Line {finding.line}</p>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}

            {scanResult.findings.length === 0 && (
              <div className="text-center py-8">
                <Check className="w-12 h-12 text-low mx-auto mb-3" />
                <p className="text-text font-[600]">No vulnerabilities detected!</p>
                <p className="text-sec text-sm mt-1">
                  Your contract passed all security checks.
                </p>
              </div>
            )}

            <Button variant="secondary" className="w-full" disabled title="Coming soon">
              ↓ Download Report
            </Button>
          </div>
        )}

        {/* Features */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {[
            {
              icon: <Zap size={20} className="text-acc-text" />,
              title: 'AI-Powered Analysis',
              description: 'Advanced AI model works with our library for accurate detection',
            },
            {
              icon: <Check size={20} className="text-acc-text" />,
              title: '50+ Invariants',
              description: 'Access our comprehensive invariant library for detailed checks',
            },
            {
              icon: <Code size={20} className="text-acc-text" />,
              title: 'Multiple Chains',
              description: 'Support for EVM, Solana, Move, and Cairo smart contracts',
            },
          ].map((feature, idx) => (
            <div
              key={idx}
              className="bg-panel border border-hair rounded-lg p-4"
            >
              {feature.icon}
              <h3 className="font-[600] text-text mt-3 mb-1">{feature.title}</h3>
              <p className="text-sm text-sec">{feature.description}</p>
            </div>
          ))}
        </div>
      </div>
    </AppShell>
  )
}
