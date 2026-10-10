import type { AiTranslateUsageWire } from './config/aitranslate'

export type AiUsageLine = { key: string; params: Record<string, string> }

/** Decimal comma by integer arithmetic, no locale API (NFR16). */
export function formatUsdParam(costUsd: number): string {
  const rounded = Math.round(costUsd * 10000) / 10000
  return rounded.toFixed(4).replace('.', ',')
}

export function aiUsageLine(usage: AiTranslateUsageWire | null): AiUsageLine {
  if (usage === null) {
    return { key: 'ai.translate.usage_unavailable', params: {} }
  }
  if (usage.cost_usd === null) {
    return { key: 'ai.translate.usage_no_price', params: { token_count: String(usage.total_tokens) } }
  }
  return {
    key: 'ai.translate.usage_with_cost',
    params: { token_count: String(usage.total_tokens), cost_usd: formatUsdParam(usage.cost_usd) },
  }
}
