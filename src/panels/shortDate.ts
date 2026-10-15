function pad2(n: number): string {
  return n < 10 ? `0${n}` : String(n)
}

/** ISO timestamp -> `dd-MM` in local time, or `null` when the input is not a date. */
export function shortDayMonth(iso: string): string | null {
  const at = new Date(iso)
  if (Number.isNaN(at.getTime())) return null
  return `${pad2(at.getDate())}-${pad2(at.getMonth() + 1)}`
}
