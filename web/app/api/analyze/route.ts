import { NextRequest, NextResponse } from 'next/server'
import { z } from 'zod'
import { createHash } from 'crypto'
import prisma from '@/lib/prisma'
import { getCurrentUser } from '@/lib/current-user'
import { monthlyScanLimit } from '@/lib/plans'

const analyzeSchema = z
  .object({
    code: z.string().max(100000, 'Code exceeds maximum size (100KB)').optional(),
    language: z.enum(['solidity', 'rust', 'move']).optional().default('solidity'),
    githubUrl: z.string().url('Invalid GitHub URL').optional(),
    projectName: z.string().trim().max(120).optional(),
  })
  .refine((data) => !!data.code || !!data.githubUrl, {
    message: 'Code or GitHub URL required',
  })

interface Finding {
  severity: 'critical' | 'high' | 'medium' | 'low'
  title: string
  description: string
  location?: string
  line?: number
  recommendation: string
}

const findingSchema = z.object({
  severity: z.enum(['critical', 'high', 'medium', 'low']),
  title: z.string().min(1).max(240),
  description: z.string().min(1).max(10000),
  location: z.string().max(500).optional(),
  line: z.number().int().positive().optional(),
  recommendation: z.string().min(1).max(10000),
})

/**
 * Pattern-based detection using invariant library
 */
function detectPatterns(code: string, language: string): Finding[] {
  const findings: Finding[] = []
  
  // Solidity patterns
  if (language === 'solidity') {
    // Reentrancy pattern
    if (/\.call\{value:.*?\}|\.call\(""\)|msg\.sender\.call/i.test(code)) {
      // Check for state changes after external calls
      if (/\.call[\s\S]*?[\w\[\]]+\s*=|\.call[\s\S]*?delete\s+/i.test(code)) {
        findings.push({
          severity: 'critical',
          title: 'Reentrancy Vulnerability',
          description: 'Potential reentrancy vulnerability detected. State changes occur after external calls.',
          recommendation: 'Use checks-effects-interactions pattern or add mutex lock. Consider OpenZeppelin ReentrancyGuard.',
          location: 'Smart Contract',
        })
      }
    }

    // Unchecked return values
    if (/\.transfer\(|\.send\(|\.call\(\)/i.test(code) && !/require\(.*\.transfer|success/i.test(code)) {
      findings.push({
        severity: 'high',
        title: 'Unchecked Transfer Return Value',
        description: 'Transfer call return value is not checked.',
        recommendation: 'Use safeTransfer from OpenZeppelin or check return value with require().',
        location: 'Smart Contract',
      })
    }

    // Integer overflow/underflow risks
    if (/\+\+|--|[\+\-\*]\s*=|\+\s+|for\s*\(.*?;.*?\+\+/i.test(code)) {
      findings.push({
        severity: 'high',
        title: 'Integer Arithmetic Risk',
        description: 'Arithmetic operations without overflow/underflow protection.',
        recommendation: 'Use SafeMath library or Solidity 0.8+ checked arithmetic.',
        location: 'Smart Contract',
      })
    }

    // Missing access control
    if (/function\s+\w+\s*\([^)]*\)\s*(public|external)\s*[^{]*{[\s\S]*?(transfer|burn|mint|pause)/i.test(code)) {
      if (!/onlyOwner|onlyAdmin|require.*msg\.sender/i.test(code)) {
        findings.push({
          severity: 'critical',
          title: 'Missing Access Control',
          description: 'Critical function lacks proper access control.',
          recommendation: 'Add onlyOwner or appropriate role-based access control.',
          location: 'Smart Contract',
        })
      }
    }
  }

  return findings
}

/**
 * AI-powered analysis using Claude
 */
async function analyzeWithAI(code: string, language: string): Promise<Finding[]> {
  const findings: Finding[] = []

  const apiKey = process.env.ANTHROPIC_API_KEY
  if (!apiKey) {
    console.warn('ANTHROPIC_API_KEY not set, skipping AI analysis')
    return findings
  }

  try {
    const response = await fetch('https://api.anthropic.com/v1/messages', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'x-api-key': apiKey,
        'anthropic-version': '2023-06-01',
      },
      body: JSON.stringify({
        model: 'claude-haiku-4-5-20251001',
        max_tokens: 2048,
        system: `You are an expert smart contract security auditor. Analyze the provided code and identify vulnerabilities.
        
Return ONLY a valid JSON array of findings with this exact structure:
[
  {
    "severity": "critical|high|medium|low",
    "title": "Vulnerability title",
    "description": "Detailed description",
    "recommendation": "How to fix"
  }
]

Focus on:
- Reentrancy vulnerabilities
- Integer overflows/underflows
- Unchecked external calls
- Access control issues
- Dangerous patterns
- Missing input validation`,
        messages: [
          {
            role: 'user',
            content: `Analyze this ${language} code for security vulnerabilities:\n\n${code.substring(0, 8000)}`,
          },
        ],
      }),
      signal: AbortSignal.timeout(30_000),
    })

    if (!response.ok) {
      console.error('Claude API error:', response.status)
      return findings
    }

    const data = await response.json()
    const content = data.content[0]?.text || ''

    // Extract JSON array from response
    const jsonMatch = content.match(/\[[\s\S]*\]/)
    if (jsonMatch) {
      const parsed = JSON.parse(jsonMatch[0])
      if (Array.isArray(parsed)) {
        for (const candidate of parsed) {
          const validated = findingSchema.safeParse(candidate)
          if (validated.success) findings.push(validated.data)
        }
      }
    }
  } catch (error) {
    console.error('AI analysis error:', error)
  }

  return findings
}

export async function POST(request: NextRequest) {
  let scanId: string | undefined
  try {
    const user = await getCurrentUser()
    if (!user) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 })
    }

    const minuteAgo = new Date(Date.now() - 60_000)
    const recentRequests = await prisma.scan.count({
      where: { userId: user.id, createdAt: { gte: minuteAgo } },
    })
    if (recentRequests >= 5) {
      return NextResponse.json(
        { error: 'Rate limit exceeded. Please wait before submitting another scan.' },
        { status: 429, headers: { 'Retry-After': '60' } }
      )
    }

    const monthStart = new Date()
    monthStart.setUTCDate(1)
    monthStart.setUTCHours(0, 0, 0, 0)
    const scansThisMonth = await prisma.scan.count({
      where: { userId: user.id, createdAt: { gte: monthStart } },
    })
    const activePlan = user.subscription?.status === 'active' ? user.subscription.plan : null
    if (scansThisMonth >= monthlyScanLimit(activePlan)) {
      return NextResponse.json({ error: 'Monthly scan quota exceeded' }, { status: 403 })
    }

    const { code, language, githubUrl, projectName } = analyzeSchema.parse(await request.json())

    if (githubUrl) {
      return NextResponse.json(
        { error: 'GitHub repository scanning requires the GitHub App integration' },
        { status: 422 }
      )
    }

    const codeToAnalyze = code || ''

    const scan = await prisma.scan.create({
      data: {
        userId: user.id,
        projectName: projectName || 'Untitled scan',
        sourceType: 'code',
        language,
        status: 'processing',
      },
    })
    scanId = scan.id

    // Combine pattern-based and AI analysis
    const [patternFindings, aiFindings] = await Promise.all([
      Promise.resolve(detectPatterns(codeToAnalyze, language)),
      analyzeWithAI(codeToAnalyze, language),
    ])

    // Combine and deduplicate findings
    const allFindings = [...patternFindings, ...aiFindings]
    const uniqueFindings = allFindings.filter(
      (finding, index, self) =>
        index ===
        self.findIndex(
          (f) =>
            f.title === finding.title &&
            f.severity === finding.severity
        )
    )

    // Sort by severity
    const severityOrder = { critical: 0, high: 1, medium: 2, low: 3 }
    uniqueFindings.sort(
      (a, b) =>
        severityOrder[a.severity as keyof typeof severityOrder] -
        severityOrder[b.severity as keyof typeof severityOrder]
    )

    await prisma.$transaction([
      ...uniqueFindings.map((finding) =>
        prisma.finding.create({
          data: {
            scanId: scan.id,
            severity: finding.severity,
            title: finding.title.slice(0, 240),
            description: finding.description,
            location: finding.location,
            line: finding.line,
            recommendation: finding.recommendation,
            fingerprint: createHash('sha256')
              .update(`${finding.severity}\0${finding.title}\0${finding.location || ''}\0${finding.line || ''}`)
              .digest('hex'),
          },
        })
      ),
      prisma.scan.update({
        where: { id: scan.id },
        data: { status: 'complete', completedAt: new Date() },
      }),
    ])

    return NextResponse.json(
      {
        success: true,
        scanId: scan.id,
        vulnerabilities: uniqueFindings,
        summary: {
          total: uniqueFindings.length,
          critical: uniqueFindings.filter((f) => f.severity === 'critical').length,
          high: uniqueFindings.filter((f) => f.severity === 'high').length,
          medium: uniqueFindings.filter((f) => f.severity === 'medium').length,
          low: uniqueFindings.filter((f) => f.severity === 'low').length,
        },
      },
      { status: 200 }
    )
  } catch (error) {
    if (scanId) {
      await prisma.scan.update({
        where: { id: scanId },
        data: { status: 'failed', error: 'Analysis failed', completedAt: new Date() },
      }).catch(() => undefined)
    }
    if (error instanceof z.ZodError) {
      return NextResponse.json(
        { error: error.issues[0].message },
        { status: 400 }
      )
    }

    console.error('Analysis error:', error)
    return NextResponse.json(
      { error: 'Analysis failed' },
      { status: 500 }
    )
  }
}
