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

export type AiUsageSummary = {
  tokenCount: number
  reportedCount: number
  sentenceCount: number
  costUsd: number | null
}

export type AiUsageSummaryKeys = {
  total: string
  totalWithCost: string
  partial: string
  partialWithCost: string
}

/** `null` when no sentence reported usage, so no "0 token" line is drawn. */
export function aiUsageSummaryLine(summary: AiUsageSummary, keys: AiUsageSummaryKeys): AiUsageLine | null {
  if (summary.reportedCount === 0) return null
  const full = summary.reportedCount === summary.sentenceCount
  const params: Record<string, string> = {
    token_count: String(summary.tokenCount),
    sentence_count: String(summary.sentenceCount),
  }
  if (!full) params.reported_count = String(summary.reportedCount)
  if (summary.costUsd !== null) {
    params.cost_usd = formatUsdParam(summary.costUsd)
    return { key: full ? keys.totalWithCost : keys.partialWithCost, params }
  }
  return { key: full ? keys.total : keys.partial, params }
}
