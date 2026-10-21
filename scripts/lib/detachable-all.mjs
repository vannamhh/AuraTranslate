/**
 * Reads the `code` of every layer in `DETACHABLE_ALL` from `tools/dict-build/src/sources_meta.rs`.
 * Returns `{ codes, problems }`; a non-empty `problems` means the codes cannot be trusted.
 *
 * @param {string} sourcesMetaText
 * @returns {{ codes: string[], problems: string[] }}
 */
export function readDetachableAllCodes(sourcesMetaText) {
  /** @type {string[]} */
  const codes = []
  /** @type {string[]} */
  const problems = []
  const arrayMatch = sourcesMetaText.match(/DETACHABLE_ALL\s*:\s*\[[^\]]*\]\s*=\s*\[([^\]]*)\]/)
  if (!arrayMatch) {
    problems.push("không tìm thấy khai báo 'DETACHABLE_ALL' trong sources_meta.rs")
    return { codes, problems }
  }
  const constNames = arrayMatch[1]
    .split(',')
    .map((s) => s.trim().replace(/^&/, ''))
    .filter(Boolean)
  for (const name of constNames) {
    const constRe = new RegExp(`pub const ${name}\\s*:\\s*SourceMeta\\s*=\\s*SourceMeta\\s*\\{([\\s\\S]*?)\\};`)
    const constMatch = sourcesMetaText.match(constRe)
    if (!constMatch) {
      problems.push(`DETACHABLE_ALL khai '${name}' nhưng không tìm thấy khối 'pub const ${name}: SourceMeta { ... }'`)
      continue
    }
    const codeMatch = constMatch[1].match(/code\s*:\s*"([^"]*)"/)
    if (!codeMatch) {
      problems.push(`khối '${name}' không có trường 'code: "..."'`)
      continue
    }
    codes.push(codeMatch[1])
  }
  return { codes, problems }
}

/**
 * Validates the `[[removed]]` rows and the `[[detachable]]` rows against `DETACHABLE_ALL`.
 *
 * @param {{ codes: string[], detachable: Record<string,string>[], removed: Record<string,string>[] }} input
 * @returns {string[]} problems
 */
export function checkLayerPartition({ codes, detachable, removed }) {
  /** @type {string[]} */
  const problems = []
  const known = new Set(codes)
  const detNames = detachable.map((o) => o.name).filter((n) => typeof n === 'string' && n !== '')
  const remNames = removed.map((o) => o.name).filter((n) => typeof n === 'string' && n !== '')

  removed.forEach((row, idx) => {
    const label = `[[removed]] #${idx + 1}`
    for (const field of ['name', 'reason', 'date']) {
      if (typeof row[field] !== 'string' || row[field] === '') problems.push(`${label}: thiếu hoặc rỗng '${field}'`)
    }
    for (const field of ['url', 'sha256']) {
      if (field in row) problems.push(`${label}: không được mang '${field}' (lớp đã gỡ không còn tải)`)
    }
    if (typeof row.name === 'string' && row.name !== '' && !known.has(row.name)) {
      problems.push(`${label}: name '${row.name}' không thuộc DETACHABLE_ALL ${JSON.stringify(codes)}`)
    }
  })

  const dupDet = detNames.filter((n, i) => detNames.indexOf(n) !== i)
  const dupRem = remNames.filter((n, i) => remNames.indexOf(n) !== i)
  if (dupDet.length) problems.push(`[[detachable]] có 'name' trùng lặp: ${JSON.stringify(dupDet)}`)
  if (dupRem.length) problems.push(`[[removed]] có 'name' trùng lặp: ${JSON.stringify(dupRem)}`)
  const both = detNames.filter((n) => remNames.includes(n))
  if (both.length) problems.push(`'name' ghi ở cả [[detachable]] lẫn [[removed]]: ${JSON.stringify([...new Set(both)])}`)

  const covered = new Set([...detNames, ...remNames])
  const missing = codes.filter((c) => !covered.has(c))
  const extra = detNames.filter((n) => !known.has(n))
  if (missing.length) {
    problems.push(`DETACHABLE_ALL có lớp ${JSON.stringify(missing)} không ở [[detachable]] cũng không ở [[removed]] — lớp bị rơi mất? Gỡ có chủ ý thì chuyển khối sang [[removed]]`)
  }
  if (extra.length) problems.push(`[[detachable]] có 'name' không thuộc DETACHABLE_ALL: ${JSON.stringify(extra)}`)
  return problems
}
