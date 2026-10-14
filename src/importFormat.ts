const NBSP = ' '

export function groupThousands(value: number): string {
  return String(value).replace(/\B(?=(\d{3})+(?!\d))/g, NBSP)
}

export function formatByteSize(bytes: number): string {
  if (bytes < 1024) return `${groupThousands(bytes)}${NBSP}B`
  const units = ['KB', 'MB', 'GB']
  let value = bytes / 1024
  let unit = 0
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024
    unit += 1
  }
  return `${value.toFixed(1).replace('.', ',')}${NBSP}${units[unit]}`
}
