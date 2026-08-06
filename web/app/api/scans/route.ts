import { NextRequest, NextResponse } from 'next/server'
import { getCurrentUser } from '@/lib/current-user'
import prisma from '@/lib/prisma'

export async function GET(request: NextRequest) {
  const user = await getCurrentUser()
  if (!user) return NextResponse.json({ error: 'Unauthorized' }, { status: 401 })

  const requestedLimit = Number(request.nextUrl.searchParams.get('limit') || 20)
  const limit = Number.isFinite(requestedLimit) ? Math.min(Math.max(requestedLimit, 1), 100) : 20
  const scans = await prisma.scan.findMany({
    where: { userId: user.id },
    orderBy: { createdAt: 'desc' },
    take: limit,
    include: { findings: { select: { severity: true, status: true } } },
  })

  return NextResponse.json({ scans })
}
