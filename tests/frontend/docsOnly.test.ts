import { describe, it, expect } from 'vitest'
import { isDocsOnly } from '../../scripts/docs-only.mjs'

describe('isDocsOnly', () => {
  it('accepts markdown anywhere and anything under _bmad-output/', () => {
    expect(isDocsOnly(['INSTALL.md', 'src/AGENTS.md', '_bmad-output/x/tickets.toml', ''])).toBe(true)
  })

  it('refuses the root AGENTS.md because naming_boundary.rs reads it', () => {
    expect(isDocsOnly(['AGENTS.md'])).toBe(false)
  })

  it('refuses a *-ban-do/ folder because Rust tests read it, even as markdown', () => {
    expect(isDocsOnly(['_bmad-output/implementation-artifacts/6-18-ban-do/REPORT.md'])).toBe(false)
  })

  it('refuses a code change pushed together with documentation', () => {
    expect(isDocsOnly(['README.md', '.github/workflows/ci.yml'])).toBe(false)
  })

  it('refuses an empty diff', () => {
    expect(isDocsOnly([])).toBe(false)
    expect(isDocsOnly([''])).toBe(false)
  })
})
