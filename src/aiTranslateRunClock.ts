let stamp = 0

export function nextAiTranslateRunStamp(): number {
  stamp += 1
  return stamp
}
