import { readdirSync, readFileSync } from 'node:fs'
import { join } from 'node:path'
import { parse } from 'vue/compiler-sfc'
import { describe, expect, it } from 'vitest'

function vueFiles(dir: string): string[] {
  return readdirSync(join(__dirname, '../..', dir), { withFileTypes: true })
    .flatMap((entry) =>
      entry.isDirectory() ? vueFiles(`${dir}/${entry.name}`) : entry.name.endsWith('.vue') ? [`${dir}/${entry.name}`] : [],
    )
    .sort()
}

const FILES = vueFiles('src')

const CONTROL_TAGS = new Set(['button', 'input', 'textarea', 'summary'])
const SHAPE_PROPERTY = /^(height|min-height|border(-[a-z-]+)?|outline(-[a-z-]+)?)$/
const STATE_PSEUDO = /:(hover|focus|focus-visible|focus-within|active|disabled)\b/

const SHAPE_EXCEPTIONS: Record<string, string> = {
  'src/GlossaryManageOverlay.vue .gm-sr-only': 'visually hidden native radio/checkbox; its wrapping label carries the ui-* shape',
  'src/GlossaryQueueOverlay.vue .gq-sr-only': 'visually hidden native radio; its wrapping label carries the ui-* shape',
  'src/TmManageOverlay.vue .tm-sr-only': 'visually hidden native radio; its wrapping label carries the ui-* shape',
  'src/PromptLibraryOverlay.vue .pl-sr-only': 'visually hidden native radio; its wrapping label carries the ui-* shape',
  'src/AlignmentOverlay.vue .al-sr-only': 'visually hidden native radio; its wrapping label carries the row hit area',
  'src/modes/LibraryMode.vue .lib-sr-only': 'visually hidden native radio; its wrapping label carries the ui-* shape',
}

const NOT_A_CONTROL: Record<string, string> = {
  'src/PromptLibraryOverlay.vue .pl-row': 'selectable two-line list row with a left-edge mark (mockup prompt-library.html .it); DESIGN §Điều khiển has no row control and ui-menu-item is a fixed 28px nowrap line',
  'src/ProofreadMemoryOverlay.vue .pfmem-row': 'selectable wrapping list row with a left-edge mark; ui-menu-item is a fixed 28px nowrap line and would clip the wrapped text',
  'src/modes/ReadingMode.vue .toc-row': 'selectable Table-of-contents row whose current mark sits on its li; a long Chapter title wraps, precedent pfmem-row, ui-menu-item is a fixed 28px nowrap line and would clip it',
  'src/ProofreadScopeOverlay.vue .pfscope-cell': 'two-line scope card (mockup proofreader.html .sc); DESIGN §Điều khiển has no card control',
  'src/ImportPreviewOverlay.vue .ip-candidate': 'encoding candidate card keeping the mockup web-import.html .candrow shape; DESIGN §Điều khiển has no card control, the label only adds ui-check so the radio is token-drawn',
  'src/BilingualImportPreviewOverlay.vue .bip-candidate': 'encoding candidate card keeping the mockup web-import.html .candrow shape; DESIGN §Điều khiển has no card control, the label only adds ui-check so the radio is token-drawn',
  'src/SettingsOverlay.vue .set-nav-item': 'section navigation row with a left-edge mark (mockup settings.html .nav a); DESIGN §Điều khiển has no navigation row control',
  'src/SettingsReadingSection.vue .sr-range': 'native range slider; DESIGN §Điều khiển has no slider control',
  'src/GlossaryConfirmStrip.vue .gcs-option': 'multi-part suggestion row with key hint, source label and a left-edge mark; precedent pfmem-row and pl-row, ui-btn is nowrap and would clip the text',
  'src/panels/LookupPanel.vue .lookup-char': 'inline link inside a sentence; DESIGN §Điều khiển has no link control',
  'src/panels/LookupPanel.vue .lookup-concordance-pointer': 'inline link inside a sentence; DESIGN §Điều khiển has no link control',
  'src/modes/ReadingLookupPopup.vue .rl-char': 'inline link inside a sentence; DESIGN §Điều khiển has no link control',
  'src/modes/ReadingLookupPopup.vue .rl-concordance': 'inline link inside a sentence; DESIGN §Điều khiển has no link control',
  'src/BilingualImportPreviewOverlay.vue .bip-cut-target': 'cut mark inside the target sentence, sized by Sàn chuột DESIGN §Layout & Spacing; DESIGN §Điều khiển has no inline control',
  'src/App.vue .mode-tab': 'mode tab on the title bar; DESIGN §Điều khiển has no mode-tab control and mockup workspace-tinh-chinh.html draws it unlike ui-tab',
}

const LABEL_EXEMPT: Record<string, string> = {
  'src/GlossaryQueueOverlay.vue input[name="gq-row-cursor"]':
    'the ancestor label .gq-row-label is the whole row (display: contents); the radio is hidden and the row is the hit area',
  'src/AlignmentOverlay.vue input[name="al-row-cursor"]':
    'the ancestor label .al-row-label is the whole row; the radio is hidden and the row is the hit area',
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

const notAControl = (file: string, node: Node): boolean =>
  classText(node)
    .split(/[^\w-]+/)
    .some((cls) => cls !== '' && Object.hasOwn(NOT_A_CONTROL, `${file} .${cls}`))

const holdsControl = (node: Node): boolean =>
  (node.children ?? []).some((child) => child.type === 1 && (CONTROL_TAGS.has(child.tag ?? '') || holdsControl(child)))

const hasUi = (text: string): boolean => /(^|[\s'"])ui-[a-z]/.test(text)

function inspect(file: string) {
  const { descriptor } = parse(readFileSync(join(__dirname, '../..', file), 'utf8'), { filename: file })
  const controls: { node: Node; ancestors: Node[] }[] = []
  const shaped: Node[] = []
  if (descriptor.template) walk(descriptor.template.ast as unknown as Node, [], (node, ancestors) => {
    if (notAControl(file, node)) return
    if (hasUi(classText(node))) shaped.push(node)
    if (!CONTROL_TAGS.has(node.tag ?? '')) return
    if (node.tag === 'input' && attr(node, 'type') === 'hidden') return
    controls.push({ node, ancestors })
  })
  return { descriptor, controls, shaped }
}

function shapeOffences(file: string): { offences: string[]; exempted: Set<string> } {
  const { descriptor, controls, shaped } = inspect(file)
  const onControls = new Set<string>()
  for (const node of [...controls.map((c) => c.node), ...shaped]) {
    for (const match of classText(node).matchAll(/(?:^|[\s'"])([a-z][\w-]*)(?=$|[\s'"])/g)) {
      onControls.add(`.${match[1]}`)
    }
  }
  const offences: string[] = []
  const exempted = new Set<string>()
  for (const style of descriptor.styles.filter((s) => s.scoped)) {
    const css = style.content.replace(/\/\*[\s\S]*?\*\//g, (m) => m.replace(/[^\n]/g, ' '))
    for (const rule of css.matchAll(/([^{}]+)\{([^{}]*)\}/g)) {
      const line = style.loc.start.line + css.slice(0, rule.index! + rule[1]!.search(/\S/)).split('\n').length - 1
      for (const selector of rule[1]!.split(',').map((s) => s.trim())) {
        const classes = [...selector.matchAll(/\.[\w-]+/g)].map((m) => m[0])
        const hit = classes.find((cls) => onControls.has(cls))
        if (hit === undefined) continue
        if (Object.hasOwn(SHAPE_EXCEPTIONS, `${file} ${hit}`)) {
          exempted.add(`${file} ${hit}`)
          continue
        }
        if (STATE_PSEUDO.test(selector)) offences.push(`${file}:${line} ${selector} styles a state`)
        for (const decl of rule[2]!.split(';')) {
          const prop = decl.split(':')[0]!.trim()
          if (SHAPE_PROPERTY.test(prop)) offences.push(`${file}:${line} ${selector} declares ${prop}`)
        }
      }
    }
  }
  return { offences, exempted }
}

describe('controls adopt the ui-* layer', () => {
  for (const file of FILES) {
    const { controls } = inspect(file)

    it(`${file}: every control carries ui-* or sits in a label that does`, () => {
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
      expect(shapeOffences(file).offences).toEqual([])
    })
  }

  it('no template uses a native <select>: pickers go through SelectMenu', () => {
    const native: string[] = []
    for (const file of FILES) {
      const { descriptor } = parse(readFileSync(join(__dirname, '../..', file), 'utf8'), { filename: file })
      if (descriptor.template) walk(descriptor.template.ast as unknown as Node, [], (node) => {
        if (node.tag === 'select') native.push(`${file}:${node.loc.start.line}`)
      })
    }
    expect(native).toEqual([])
  })

  it('every SHAPE_EXCEPTIONS key still exempts a scoped rule in its file', () => {
    const live = new Set(FILES.flatMap((file) => [...shapeOffences(file).exempted]))
    expect(Object.keys(SHAPE_EXCEPTIONS).filter((key) => !live.has(key))).toEqual([])
  })

  it('every NOT_A_CONTROL key still names a control, or a label wrapping one, in its file', () => {
    const live = new Set<string>()
    for (const file of FILES) {
      const { descriptor } = parse(readFileSync(join(__dirname, '../..', file), 'utf8'), { filename: file })
      if (descriptor.template) walk(descriptor.template.ast as unknown as Node, [], (node) => {
        if ((CONTROL_TAGS.has(node.tag ?? '') || (node.tag === 'label' && holdsControl(node))) && notAControl(file, node)) {
          for (const key of Object.keys(NOT_A_CONTROL)) {
            if (key.startsWith(`${file} `) && classText(node).split(/[^\w-]+/).includes(key.slice(file.length + 2))) live.add(key)
          }
        }
      })
    }
    expect(Object.keys(NOT_A_CONTROL).filter((key) => !live.has(key))).toEqual([])
  })

  it('reads every .vue under src and reaches controls, the overlay with many', () => {
    const listed = readdirSync(join(__dirname, '../../src'), { recursive: true, encoding: 'utf8' })
      .filter((path) => path.endsWith('.vue'))
      .map((path) => `src/${path.split('\\').join('/')}`)
      .sort()
    expect(FILES).toEqual(listed)
    const counts = FILES.map((file) => inspect(file).controls.length)
    expect(counts.reduce((a, b) => a + b, 0)).toBeGreaterThan(0)
    expect(counts[FILES.indexOf('src/GlossaryManageOverlay.vue')]).toBeGreaterThanOrEqual(20)
  })
})
