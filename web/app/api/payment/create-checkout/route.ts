import Stripe from 'stripe'
import { NextRequest, NextResponse } from 'next/server'
import { getCurrentUser } from '@/lib/current-user'
import { isPlanId, PLANS } from '@/lib/plans'
import { z } from 'zod'

const checkoutSchema = z.object({
  planId: z.string().min(1, 'planId is required'),
})

function getStripeClient() {
  const key = process.env.STRIPE_SECRET_KEY
  if (!key) {
    throw new Error('STRIPE_SECRET_KEY is not configured')
  }
  return new Stripe(key)
}

export async function POST(request: NextRequest) {
  try {
    const stripe = getStripeClient()
    const user = await getCurrentUser()
    if (!user?.email) {
      return NextResponse.json(
        { error: 'Unauthorized' },
        { status: 401 }
      )
    }

    const { planId } = checkoutSchema.parse(await request.json())
    if (!isPlanId(planId)) {
      return NextResponse.json({ error: 'Unknown plan' }, { status: 400 })
    }
    const plan = PLANS[planId]

    // Create Stripe checkout session
    const checkoutSession = await stripe.checkout.sessions.create({
      payment_method_types: ['card'],
      customer_email: user.email,
      line_items: [
        {
          price_data: {
            currency: 'usd',
            product_data: {
              name: plan.name,
              description: `Truent ${plan.name} Plan`,
            },
            unit_amount: plan.monthlyAmount,
            recurring: {
              interval: 'month',
              interval_count: 1,
            },
          },
          quantity: 1,
        },
      ],
      mode: 'subscription',
      success_url: `${process.env.NEXTAUTH_URL}/dashboard?payment=success`,
      cancel_url: `${process.env.NEXTAUTH_URL}/pricing?payment=cancelled`,
      metadata: {
        planId,
        userId: user.id,
      },
    })

    return NextResponse.json(
      { sessionId: checkoutSession.id, url: checkoutSession.url },
      { status: 200 }
    )
  } catch (error) {
    if (error instanceof z.ZodError) {
      return NextResponse.json(
        { error: error.issues[0].message },
        { status: 400 }
      )
    }

    console.error('Stripe error:', error)
    return NextResponse.json(
      { error: 'Payment processing failed' },
      { status: 500 }
    )
  }
}
