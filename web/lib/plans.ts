export const PLANS = {
  professional: { name: 'Professional', monthlyAmount: 49900, scansPerMonth: 10000 },
} as const

export type PlanId = keyof typeof PLANS

export function isPlanId(value: string): value is PlanId {
  return Object.prototype.hasOwnProperty.call(PLANS, value)
}

export function monthlyScanLimit(plan?: string | null): number {
  if (plan && isPlanId(plan)) return PLANS[plan].scansPerMonth
  return 5
}
