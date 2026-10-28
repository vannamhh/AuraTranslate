import { readFileSync } from 'node:fs'
import { join } from 'node:path'
import { parse } from 'vue/compiler-sfc'
import { describe, expect, it } from 'vitest'

const FILES = [
  'src/DropdownMenu.vue',
  'src/SelectMenu.vue',
  'src/TitlebarToolsMenu.vue',
  'src/panels/LookupSourcesMenu.vue',
  'src/GlossaryManageOverlay.vue',
  'src/GlossaryQueueOverlay.vue',
  'src/GlossaryHarvestOverlay.vue',
  'src/GlossaryImportOverlay.vue',
  'src/GlossaryQuickAdd.vue',
  'src/TmManageOverlay.vue',
  'src/TmImportOverlay.vue',
]

const CONTROL_TAGS = new Set(['button', 'input', 'textarea', 'summary'])
const SHAPE_PROPERTY = /^(height|min-height|border(-[a-z-]+)?|outline(-[a-z-]+)?)$/
const STATE_PSEUDO = /:(hover|focus|focus-visible|focus-within|active|disabled)\b/

const SHAPE_EXCEPTIONS: Record<string, string> = {
  'src/GlossaryManageOverlay.vue .gm-sr-only': 'visually hidden native radio/checkbox; its wrapping label carries the ui-* shape',
  'src/GlossaryQueueOverlay.vue .gq-sr-only': 'visually hidden native radio; its wrapping label carries the ui-* shape',
  'src/TmManageOverlay.vue .tm-sr-only': 'visually hidden native radio; its wrapping label carries the ui-* shape',
}

const LABEL_EXEMPT: Record<string, string> = {
  'src/GlossaryQueueOverlay.vue input[name="gq-row-cursor"]':
    'the ancestor label .gq-row-label is the whole row (display: contents); the radio is hidden and the row is the hit area',
}

interface Node {
  type: number
  tag?: string
  props?: Prop[]
  children?: Node[]
  loc: { start: { line: number } }
}
interface Prop {
  type: number
  name: string
  value?: { content: string }
  arg?: { content?: string }
  exp?: { content?: string }
}

function classText(node: Node): string {
  return (node.props ?? [])
    .map((prop) => {
      if (prop.type === 6 && prop.name === 'class') return prop.value?.content ?? ''
      if (prop.type === 7 && prop.name === 'bind' && prop.arg?.content === 'class') return prop.exp?.content ?? ''
      return ''
    })
    .join(' ')
}

function attr(node: Node, name: string): string | undefined {
  return (node.props ?? []).find((prop) => prop.type === 6 && prop.name === name)?.value?.content
}

function walk(node: Node, ancestors: Node[], visit: (node: Node, ancestors: Node[]) => void): void {
  if (node.type === 1) visit(node, ancestors)
  for (const child of node.children ?? []) walk(child, node.type === 1 ? [...ancestors, node] : ancestors, visit)
}

const hasUi = (text: string): boolean => /(^|[\s'"])ui-[a-z]/.test(text)

function inspect(file: string) {
  const { descriptor } = parse(readFileSync(join(__dirname, '../..', file), 'utf8'), { filename: file })
  const controls: { node: Node; ancestors: Node[] }[] = []
  const shaped: Node[] = []
  walk(descriptor.template!.ast as unknown as Node, [], (node, ancestors) => {
    if (hasUi(classText(node))) shaped.push(node)
    if (!CONTROL_TAGS.has(node.tag ?? '')) return
    if (node.tag === 'input' && attr(node, 'type') === 'hidden') return
    controls.push({ node, ancestors })
  })
  return { descriptor, controls, shaped }
}

describe('controls adopt the ui-* layer', () => {
  for (const file of FILES) {
    const { descriptor, controls, shaped } = inspect(file)

    it(`${file}: every control carries ui-* or sits in a label that does`, () => {
      expect(controls.length, `${file} has no control to guard`).toBeGreaterThan(0)
      const bare = controls
        .filter(({ node, ancestors }) => {
          if (hasUi(classText(node))) return false
          if (Object.hasOwn(LABEL_EXEMPT, `${file} ${node.tag}[name="${attr(node, 'name')}"]`)) return false
          const label = [...ancestors].reverse().find((a) => a.tag === 'label')
          return label === undefined || !hasUi(classText(label))
        })
        .map(({ node }) => `${file}:${node.loc.start.line} <${node.tag}>`)
      expect(bare).toEqual([])
    })

    it(`${file}: scoped rules on a control's or a ui-* element's classes leave shape and state to ui-*`, () => {
      const onControls = new Set<string>()
      for (const node of [...controls.map((c) => c.node), ...shaped]) {
        for (const match of classText(node).matchAll(/(?:^|[\s'"])([a-z][\w-]*)(?=$|[\s'"])/g)) {
          onControls.add(`.${match[1]}`)
        }
      }
      const offences: string[] = []
      for (const style of descriptor.styles.filter((s) => s.scoped)) {
        const css = style.content.replace(/\/\*[\s\S]*?\*\//g, (m) => m.replace(/[^\n]/g, ' '))
        for (const rule of css.matchAll(/([^{}]+)\{([^{}]*)\}/g)) {
          const line = style.loc.start.line + css.slice(0, rule.index! + rule[1]!.search(/\S/)).split('\n').length - 1
          for (const selector of rule[1]!.split(',').map((s) => s.trim())) {
            const classes = [...selector.matchAll(/\.[\w-]+/g)].map((m) => m[0])
            const hit = classes.find((cls) => onControls.has(cls))
            if (hit === undefined || Object.hasOwn(SHAPE_EXCEPTIONS, `${file} ${hit}`)) continue
            if (STATE_PSEUDO.test(selector)) offences.push(`${file}:${line} ${selector} styles a state`)
            for (const decl of rule[2]!.split(';')) {
              const prop = decl.split(':')[0]!.trim()
              if (SHAPE_PROPERTY.test(prop)) offences.push(`${file}:${line} ${selector} declares ${prop}`)
            }
          }
        }
      }
      expect(offences).toEqual([])
    })
  }

  it('reaches every listed file with at least one control, and the overlay with many', () => {
    const counts = FILES.map((file) => inspect(file).controls.length)
    expect(Math.min(...counts)).toBeGreaterThanOrEqual(1)
    expect(counts[FILES.indexOf('src/GlossaryManageOverlay.vue')]).toBeGreaterThanOrEqual(20)
  })
})
