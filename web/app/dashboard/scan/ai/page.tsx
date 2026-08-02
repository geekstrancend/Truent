'use client'

import { useState } from 'react'
import { AppShell } from '@/components/layout/AppShell'
import { Button } from '@/components/ui/Button'
import { Brain, Send, Copy, Check, AlertCircle, Loader } from 'lucide-react'

interface AnalysisResult {
  vulnerabilities: string[]
  recommendations: string[]
  riskLevel: 'low' | 'medium' | 'high' | 'critical'
  summary: string
}

export default function AIScanPage() {
  const [contractCode, setContractCode] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const [analysis, setAnalysis] = useState<AnalysisResult | null>(null)
  const [copiedCode, setCopiedCode] = useState(false)

  const handleAnalyze = async () => {
    if (!contractCode.trim()) {
      setError('Please paste your smart contract code')
      return
    }

    setLoading(true)
    setError('')

    try {
      const response = await fetch('/api/analyze', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          code: contractCode,
        }),
      })

      if (!response.ok) {
        throw new Error(`API error: ${response.statusText}`)
      }

      const data = await response.json()
      setAnalysis(data)
    } catch (err) {
      setError(
        err instanceof Error
          ? err.message
          : 'Failed to analyze code. Please check your API configuration.',
      )
    } finally {
      setLoading(false)
    }
  }

  const handleCopyCode = () => {
    navigator.clipboard.writeText(contractCode)
    setCopiedCode(true)
    setTimeout(() => setCopiedCode(false), 2000)
  }

  const getRiskColor = (level: string) => {
    switch (level) {
      case 'critical':
        return 'text-critical'
      case 'high':
        return 'text-high'
      case 'medium':
        return 'text-medium'
      case 'low':
        return 'text-low'
      default:
        return 'text-text'
    }
  }

  const handleReset = () => {
    setContractCode('')
    setAnalysis(null)
    setError('')
  }

  return (
    <AppShell currentPage="audits" onNewScan={handleReset}>
      <div className="p-8 max-w-site mx-auto">
        <div className="mb-8">
          <div className="flex items-center gap-3 mb-4">
            <Brain className="w-8 h-8 text-brand" />
            <h1 className="font-display text-4xl font-[600] text-text">
              AI-Powered Code Analysis
            </h1>
          </div>
          <p className="text-body-lg text-sec max-w-2xl">
            Leverage Claude Haiku to analyze your smart contracts for vulnerabilities, best practices, and security improvements.
          </p>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
          {/* Code Input */}
          <div className="space-y-4">
            <div>
              <div className="flex justify-between items-center mb-2">
                <label className="text-sm font-[600] text-text">Smart Contract Code</label>
                <button
                  onClick={handleCopyCode}
                  className="text-xs text-sec hover:text-text transition flex items-center gap-1"
                >
                  {copiedCode ? (
                    <>
                      <Check size={14} />
                      Copied
                    </>
                  ) : (
                    <>
                      <Copy size={14} />
                      Copy
                    </>
                  )}
                </button>
              </div>
              <textarea
                value={contractCode}
                onChange={(e) => setContractCode(e.target.value)}
                placeholder="Paste your Solidity, Rust, or Move smart contract code here..."
                className="w-full h-[500px] px-4 py-3 bg-surface-2 border border-hair rounded-lg text-text font-mono text-sm placeholder-on-surface-variant focus:outline-none focus:border-brand transition resize-none"
              />
            </div>

            {error && (
              <div className="flex items-center gap-2 p-3 bg-critical-bg border border-critical-border rounded-lg">
                <AlertCircle className="w-5 h-5 text-critical flex-shrink-0" />
                <p className="text-sm text-critical">{error}</p>
              </div>
            )}

            <Button
              variant="primary"
              fullWidth
              onClick={handleAnalyze}
              disabled={loading || !contractCode.trim()}
              className="gap-2 justify-center"
            >
              {loading ? (
                <>
                  <Loader className="w-4 h-4 animate-spin" />
                  Analyzing...
                </>
              ) : (
                <>
                  <Send className="w-4 h-4" />
                  Analyze with Claude Haiku
                </>
              )}
            </Button>

            <div className="p-3 bg-brand/10 border border-brand rounded-lg text-xs text-sec">
              <p className="font-[600] text-text mb-1">Model: Claude Haiku 4.5</p>
              <p>Fast and efficient AI analysis powered by Anthropic&apos;s Claude Haiku model.</p>
            </div>
          </div>

          {/* Analysis Results */}
          <div className="space-y-4">
            {analysis ? (
              <div className="space-y-4">
                {/* Risk Badge */}
                <div className={`p-4 bg-panel rounded-lg border-l-4 ${
                  analysis.riskLevel === 'critical'
                    ? 'border-critical bg-critical/5'
                    : analysis.riskLevel === 'high'
                      ? 'border-high bg-high/5'
                      : analysis.riskLevel === 'medium'
                        ? 'border-medium bg-medium/5'
                        : 'border-low bg-low/5'
                }`}>
                  <p className="text-xs text-sec mb-1">RISK LEVEL</p>
                  <p className={`font-display text-2xl font-[700] capitalize ${getRiskColor(analysis.riskLevel)}`}>
                    {analysis.riskLevel}
                  </p>
                </div>

                {/* Summary */}
                <div className="bg-panel rounded-lg p-4">
                  <h3 className="text-sm font-[600] text-text mb-2">Analysis Summary</h3>
                  <p className="text-body-sm text-sec leading-6">
                    {analysis.summary}
                  </p>
                </div>

                {/* Vulnerabilities */}
                {analysis.vulnerabilities.length > 0 && (
                  <div className="bg-panel rounded-lg p-4">
                    <h3 className="text-sm font-[600] text-critical mb-3">
                      Potential Vulnerabilities ({analysis.vulnerabilities.length})
                    </h3>
                    <ul className="space-y-2">
                      {analysis.vulnerabilities.map((vuln, idx) => (
                        <li key={idx} className="flex gap-2 text-sm">
                          <span className="text-critical flex-shrink-0">•</span>
                          <span className="text-sec">{vuln}</span>
                        </li>
                      ))}
                    </ul>
                  </div>
                )}

                {/* Recommendations */}
                {analysis.recommendations.length > 0 && (
                  <div className="bg-panel rounded-lg p-4">
                    <h3 className="text-sm font-[600] text-medium mb-3">
                      Recommendations ({analysis.recommendations.length})
                    </h3>
                    <ul className="space-y-2">
                      {analysis.recommendations.map((rec, idx) => (
                        <li key={idx} className="flex gap-2 text-sm">
                          <span className="text-medium flex-shrink-0">✓</span>
                          <span className="text-sec">{rec}</span>
                        </li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>
            ) : (
              <div className="h-[500px] flex flex-col items-center justify-center bg-panel rounded-lg border border-dashed border-hair">
                <Brain className="w-12 h-12 text-sec mb-4 opacity-50" />
                <p className="text-center text-sec">
                  <span className="block font-[600] mb-1">Paste your code to get started</span>
                  <span className="text-sm">Claude Haiku will analyze your smart contract</span>
                </p>
              </div>
            )}
          </div>
        </div>

        {/* Info Cards */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mt-12 pt-8 border-t border-hair">
          <div className="bg-panel rounded-lg p-4">
            <h4 className="text-sm font-[600] text-text mb-2">Supported Languages</h4>
            <p className="text-xs text-sec">Solidity, Rust, Move, and Vyper</p>
          </div>
          <div className="bg-panel rounded-lg p-4">
            <h4 className="text-sm font-[600] text-text mb-2">AI Model</h4>
            <p className="text-xs text-sec">Claude Haiku 4.5 - Fast and efficient</p>
          </div>
          <div className="bg-panel rounded-lg p-4">
            <h4 className="text-sm font-[600] text-text mb-2">Privacy</h4>
            <p className="text-xs text-sec">Your code is processed securely and never stored</p>
          </div>
        </div>
      </div>
    </AppShell>
  )
}
