'use client'

import { DocsShell } from '@/components/layout/DocsShell'
import { CodeBlock } from '@/components/ui/CodeBlock'

export default function CLIReferencePage() {
  const toc = [
    { label: 'truent check', href: '#truent-check' },
    { label: 'Exit Codes', href: '#exit-codes' },
    { label: 'Examples', href: '#examples' },
  ]

  return (
    <DocsShell pageTitle="CLI Reference" tableOfContents={toc}>
      <article className="space-y-12">
        <div>
          <h1 className="font-display text-5xl font-[600] text-text mb-4">
            CLI Reference
          </h1>
          <p className="text-body-lg text-sec max-w-2xl">
            Complete documentation for the Truent command-line interface and all available commands.
          </p>
        </div>

        {/* truent check */}
        <section id="truent-check">
          <h2 className="font-display text-2xl font-[600] text-text mt-12 mb-4 scroll-mt-24">
            truent check
          </h2>
          <p className="text-body-md text-sec mb-4 leading-relaxed">
            Scan a smart contract directory or file for security vulnerabilities and invariant violations.
          </p>

          <h3 className="font-display text-lg font-[600] text-text mt-6 mb-3">
            Usage
          </h3>
          <CodeBlock
            language="bash"
            code={`truent check [PATH] [OPTIONS]`}
          />

          <h3 className="font-display text-lg font-[600] text-text mt-6 mb-3">
            Arguments
          </h3>
          <div className="overflow-x-auto mt-4">
            <table className="w-full border border-hair rounded text-body-md">
              <thead>
                <tr className="border-b border-hair">
                  <th className="text-left p-3 bg-panel font-[600] text-text">Argument</th>
                  <th className="text-left p-3 bg-panel font-[600] text-text">Description</th>
                  <th className="text-left p-3 bg-panel font-[600] text-text">Default</th>
                </tr>
              </thead>
              <tbody>
                <tr className="border-b border-hair">
                  <td className="p-3"><code className="font-mono text-xs">PATH</code></td>
                  <td className="p-3 text-sec">Directory or file to scan</td>
                  <td className="p-3 text-sec">.</td>
                </tr>
              </tbody>
            </table>
          </div>

          <h3 className="font-display text-lg font-[600] text-text mt-6 mb-3">
            Options
          </h3>
          <div className="overflow-x-auto mt-4">
            <table className="w-full border border-hair rounded text-body-md">
              <thead>
                <tr className="border-b border-hair">
                  <th className="text-left p-3 bg-panel font-[600] text-text">Flag</th>
                  <th className="text-left p-3 bg-panel font-[600] text-text">Description</th>
                  <th className="text-left p-3 bg-panel font-[600] text-text">Default</th>
                </tr>
              </thead>
              <tbody>
                <tr className="border-b border-hair">
                  <td className="p-3"><code className="font-mono text-xs">--chain</code></td>
                  <td className="p-3 text-sec">Blockchain target: evm, solana, move</td>
                  <td className="p-3 text-sec">evm</td>
                </tr>
                <tr className="border-b border-hair">
                  <td className="p-3"><code className="font-mono text-xs">--format</code></td>
                  <td className="p-3 text-sec">Output format: json, html, pdf</td>
                  <td className="p-3 text-sec">pdf</td>
                </tr>
                <tr className="border-b border-hair">
                  <td className="p-3"><code className="font-mono text-xs">--output</code></td>
                  <td className="p-3 text-sec">Output file path</td>
                  <td className="p-3 text-sec">report.pdf</td>
                </tr>
                <tr className="border-b border-hair">
                  <td className="p-3"><code className="font-mono text-xs">--seed</code></td>
                  <td className="p-3 text-sec">Reproducible randomness seed</td>
                  <td className="p-3 text-sec">auto</td>
                </tr>
                <tr>
                  <td className="p-3"><code className="font-mono text-xs">--verbose</code></td>
                  <td className="p-3 text-sec">Enable verbose logging</td>
                  <td className="p-3 text-sec">false</td>
                </tr>
              </tbody>
            </table>
          </div>
        </section>

        {/* Exit Codes */}
        <section id="exit-codes">
          <h2 className="font-display text-2xl font-[600] text-text mt-12 mb-4 scroll-mt-24">
            Exit Codes
          </h2>
          <p className="text-body-md text-sec mb-4 leading-relaxed">
            Truent uses exit codes to indicate scan results. Use these in CI/CD pipelines to gate deployments.
          </p>

          <div className="overflow-x-auto mt-4">
            <table className="w-full border border-hair rounded text-body-md">
              <thead>
                <tr className="border-b border-hair">
                  <th className="text-left p-3 bg-panel font-[600] text-text">Code</th>
                  <th className="text-left p-3 bg-panel font-[600] text-text">Meaning</th>
                </tr>
              </thead>
              <tbody>
                <tr className="border-b border-hair">
                  <td className="p-3"><span className="text-low font-[600]">0</span></td>
                  <td className="p-3 text-sec">Scan completed successfully, no critical/high findings</td>
                </tr>
                <tr className="border-b border-hair">
                  <td className="p-3"><span className="text-medium font-[600]">1</span></td>
                  <td className="p-3 text-sec">Scan found medium or low severity findings</td>
                </tr>
                <tr className="border-b border-hair">
                  <td className="p-3"><span className="text-high font-[600]">2</span></td>
                  <td className="p-3 text-sec">Scan found high severity findings</td>
                </tr>
                <tr>
                  <td className="p-3"><span className="text-critical font-[600]">3</span></td>
                  <td className="p-3 text-sec">Scan found critical findings or scan failed</td>
                </tr>
              </tbody>
            </table>
          </div>
        </section>

        {/* Examples */}
        <section id="examples">
          <h2 className="font-display text-2xl font-[600] text-text mt-12 mb-4 scroll-mt-24">
            Examples
          </h2>

          <h3 className="font-display text-lg font-[600] text-text mt-6 mb-3">
            Scan current directory for EVM contracts
          </h3>
          <CodeBlock
            language="bash"
            code={`truent check . --chain evm`}
          />

          <h3 className="font-display text-lg font-[600] text-text mt-6 mb-3">
            Generate JSON report to specific file
          </h3>
          <CodeBlock
            language="bash"
            code={`truent check ./contracts --chain evm --format json --output security-report.json`}
          />

          <h3 className="font-display text-lg font-[600] text-text mt-6 mb-3">
            Scan Solana project with reproducible seed
          </h3>
          <CodeBlock
            language="bash"
            code={`truent check . --chain solana --seed 42`}
          />

          <h3 className="font-display text-lg font-[600] text-text mt-6 mb-3">
            CI/CD gating example
          </h3>
          <CodeBlock
            language="bash"
            code={`truent check . --chain evm --format json --output report.json
if [ $? -ge 2 ]; then
  echo "Critical or high findings detected"
  exit 1
fi`}
          />
        </section>
      </article>
    </DocsShell>
  )
}
