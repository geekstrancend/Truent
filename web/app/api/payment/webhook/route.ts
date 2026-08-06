import { NextRequest, NextResponse } from 'next/server'
import Stripe from 'stripe'
import prisma from '@/lib/prisma'
import { isPlanId } from '@/lib/plans'

export const runtime = 'nodejs'

function stripeClient() {
  if (!process.env.STRIPE_SECRET_KEY) throw new Error('STRIPE_SECRET_KEY is not configured')
  return new Stripe(process.env.STRIPE_SECRET_KEY)
}

export async function POST(request: NextRequest) {
  const signature = request.headers.get('stripe-signature')
  const secret = process.env.STRIPE_WEBHOOK_SECRET
  if (!signature || !secret) {
    return NextResponse.json({ error: 'Webhook is not configured' }, { status: 503 })
  }

  let event: Stripe.Event
  try {
    const stripe = stripeClient()
    event = stripe.webhooks.constructEvent(await request.text(), signature, secret)
  } catch (error) {
    console.error('Invalid Stripe webhook', error)
    return NextResponse.json({ error: 'Invalid signature' }, { status: 400 })
  }

  if (event.type === 'checkout.session.completed') {
    const checkout = event.data.object
    const userId = checkout.metadata?.userId
    const plan = checkout.metadata?.planId
    if (userId && plan && isPlanId(plan) && checkout.subscription) {
      await prisma.subscription.upsert({
        where: { userId },
        create: {
          userId,
          provider: 'stripe',
          providerCustomerId: String(checkout.customer || ''),
          providerSubscriptionId: String(checkout.subscription),
          plan,
          status: 'active',
        },
        update: {
          providerCustomerId: String(checkout.customer || ''),
          providerSubscriptionId: String(checkout.subscription),
          plan,
          status: 'active',
        },
      })
    }
  }

  if (event.type === 'customer.subscription.updated' || event.type === 'customer.subscription.deleted') {
    const subscription = event.data.object
    await prisma.subscription.updateMany({
      where: { providerSubscriptionId: subscription.id },
      data: {
        status: subscription.status,
        cancelAtPeriodEnd: subscription.cancel_at_period_end,
        currentPeriodEnd: new Date(subscription.items.data[0]?.current_period_end * 1000),
      },
    })
  }

  return NextResponse.json({ received: true })
}
