import { randomBytes, createHash } from 'crypto'
import { NextRequest, NextResponse } from 'next/server'
import { z } from 'zod'
import prisma from '@/lib/prisma'

const schema = z.object({ address: z.string().regex(/^0x[a-fA-F0-9]{40}$/) })

export async function POST(request: NextRequest) {
  try {
    const { address: rawAddress } = schema.parse(await request.json())
    const address = rawAddress.toLowerCase()
    const nonce = randomBytes(24).toString('hex')
    const message = `Truent sign-in\nAddress: ${address}\nNonce: ${nonce}`
    await prisma.authNonce.upsert({
      where: { address },
      create: {
        address,
        nonceHash: createHash('sha256').update(nonce).digest('hex'),
        expiresAt: new Date(Date.now() + 5 * 60_000),
      },
      update: {
        nonceHash: createHash('sha256').update(nonce).digest('hex'),
        expiresAt: new Date(Date.now() + 5 * 60_000),
      },
    })
    return NextResponse.json({ message, expiresIn: 300 })
  } catch (error) {
    if (error instanceof z.ZodError) {
      return NextResponse.json({ error: 'Invalid wallet address' }, { status: 400 })
    }
    console.error('Wallet nonce creation failed', error)
    return NextResponse.json({ error: 'Unable to create nonce' }, { status: 500 })
  }
}
