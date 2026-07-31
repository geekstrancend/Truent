'use client'

import { BookOpen, Target, Cpu } from 'lucide-react'
import { DocsShell } from '@/components/layout/DocsShell'
import { Callout } from '@/components/ui/Callout'
import { CodeBlock } from '@/components/ui/CodeBlock'

export default function AICoAuditorPage() {
  return (
    <DocsShell pageTitle="AI Co-Auditor">
      <article className="space-y-12">
        {/* Hero */}
        <div>
          <h1 className="font-display text-5xl font-[600] text-text mb-4">
            AI Co-Auditor
          </h1>
          <p className="text-body-lg text-sec leading-7">
            How the AI layer reasons across millions of code pathways to detect protocol-level logical flaws.
          </p>
        </div>

        {/* Three-Step Diagram */}
        <div className="bg-panel border border-hair rounded-lg p-8">
          <div className="grid grid-cols-3 gap-4 items-center">
            {/* Step 1 */}
            <div className="bg-surface-2 border border-hair rounded-lg p-6 text-center">
              <div className="flex justify-center mb-4">
                <BookOpen size={32} className="text-medium" />
              </div>
              <p className="text-label-sm text-text mb-2">1. LIBRARY</p>
              <p className="text-body-md text-sec text-sm">
                Scanning global invariant patterns and protocol specs.
              </p>
            </div>

            {/* Arrow */}
            <div className="text-center">
              <div className="text-headline-lg text-sec" aria-hidden="true">→</div>
            </div>

            {/* Step 2 */}
            <div className="bg-surface-2 border border-hair rounded-lg p-6 text-center">
              <div className="flex justify-center mb-4">
                <Target size={32} className="text-medium" />
              </div>
              <p className="text-label-sm text-text mb-2">2. HITS</p>
              <p className="text-body-md text-sec text-sm">
                Identifying potential violations using symbolic execution.
              </p>
            </div>

            {/* Arrow */}
            <div className="text-center">
              <div className="text-headline-lg text-sec" aria-hidden="true">→</div>
            </div>

            {/* Step 3 */}
            <div className="bg-surface-2 border border-hair rounded-lg p-6 text-center col-span-1">
              <div className="flex justify-center mb-4">
                <Cpu size={32} className="text-high" />
              </div>
              <p className="text-label-sm text-text mb-2">3. AI ENRICHMENT</p>
              <p className="text-body-md text-sec text-sm">
                Claude Sonnet filters noise and constructs attack vectors.
              </p>
            </div>
          </div>
        </div>

        {/* Pro Tip */}
        <Callout type="info" title="PRO TIP">
          The AI Co-Auditor works best when project documentation is provided via a{' '}
          <code className="bg-panel border border-hair text-acc-text px-1.5 py-0.5 rounded font-mono text-xs">
            TRUENT.md
          </code>
          {' '}file in the root directory.
        </Callout>

        {/* Comparison Table */}
        <div>
          <h2 className="font-display text-2xl font-[600] text-text mb-6">
            Comparison Table
          </h2>
          <div className="border border-hair rounded-lg overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="bg-panel border-b border-hair">
                  <th className="text-label-sm text-sec text-left px-6 py-3">Feature</th>
                  <th className="text-label-sm text-sec text-left px-6 py-3">Without AI</th>
                  <th className="text-label-sm text-sec text-left px-6 py-3">With AI Co-Auditor</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-hair">
                <tr className="hover:bg-panel transition-colors">
                  <td className="text-body-md text-text px-6 py-4">Logic Flaw Detection</td>
                  <td className="text-body-md text-sec px-6 py-4">Pattern-based only</td>
                  <td className="text-body-md text-low font-[600] px-6 py-4">Reasoning-based discovery</td>
                </tr>
                <tr className="hover:bg-panel transition-colors">
                  <td className="text-body-md text-text px-6 py-4">False Positive Rate</td>
                  <td className="text-body-md text-sec px-6 py-4">High (Manual triage req.)</td>
                  <td className="text-body-md text-low font-[600] px-6 py-4">Low (80% noise reduction)</td>
                </tr>
                <tr className="hover:bg-panel transition-colors">
                  <td className="text-body-md text-text px-6 py-4">Inferred Invariants</td>
                  <td className="text-body-md text-sec px-6 py-4">None</td>
                  <td className="text-body-md text-low font-[600] px-6 py-4">Auto-generated .sinv rules</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        {/* Configuration */}
        <div>
          <h2 className="font-display text-2xl font-[600] text-text mb-6">
            Configuration
          </h2>
          <CodeBlock
            language="yaml"
            code={`ai:
  enabled: true
  model: claude-sonnet
  context_window: full
  remediation_prompts: true
  chat_enabled: true`}
          />
        </div>

        {/* How It Works */}
        <div>
          <h2 className="font-display text-2xl font-[600] text-text mb-6">
            How It Works
          </h2>
          <div className="space-y-6">
            <div>
              <h3 className="font-display text-lg font-[500] text-text mb-2">
                1. Symbolic Execution
              </h3>
              <p className="text-body-md text-sec">
                The invariant library is scanned against your contract code paths, creating a set of potential violations. Each violation is tracked with its execution context.
              </p>
            </div>

            <div>
              <h3 className="font-display text-lg font-[500] text-text mb-2">
                2. AI Enrichment
              </h3>
              <p className="text-body-md text-sec">
                Claude Sonnet receives the top candidates along with your protocol documentation. It performs deep reasoning to:
              </p>
              <ul className="mt-3 space-y-2 ml-4">
                <li className="text-body-md text-sec">
                  • Filter false positives by understanding business logic intent
                </li>
                <li className="text-body-md text-sec">
                  • Construct realistic attack vectors
                </li>
                <li className="text-body-md text-sec">
                  • Generate specialized test cases for edge conditions
                </li>
                <li className="text-body-md text-sec">
                  • Suggest remediation code snippets
                </li>
              </ul>
            </div>

            <div>
              <h3 className="font-display text-lg font-[500] text-text mb-2">
                3. Actionable Reports
              </h3>
              <p className="text-body-md text-sec">
                The output is a prioritized list of confirmed vulnerabilities with clear remediation guidance and formal verification proofs.
              </p>
            </div>
          </div>
        </div>

        {/* Performance */}
        <Callout type="success" title="PERFORMANCE METRICS">
          The AI Co-Auditor reduces audit time by 40% while improving finding depth. Combines 1,400+ invariants with contextual reasoning for enterprise-grade detection.
        </Callout>
      </article>
    </DocsShell>
  )
}
