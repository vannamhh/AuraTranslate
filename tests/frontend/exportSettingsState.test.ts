import { beforeEach, describe, expect, it, vi } from 'vitest'

const getMock = vi.fn()
const saveMock = vi.fn()
vi.mock('../../src/config/attribution', () => ({
  translatorNameGet: (...a: unknown[]) => getMock(...a),
  translatorNameSave: (...a: unknown[]) => saveMock(...a),
}))

const ipcError = { code: 'ipc.unknown', message_key: 'err.unknown', params: {}, retryable: false }

async function fresh() {
  vi.resetModules()
  getMock.mockReset()
  saveMock.mockReset()
  return import('../../src/exportSettingsState')
}

beforeEach(() => {
  document.body.innerHTML = ''
})

describe('export settings state', () => {
  it('load fills the input from the stored name', async () => {
    const s = await fresh()
    getMock.mockResolvedValue({ name: 'Ice', error: null })
    await s.loadExportSettingsForm()
    expect(s.exportSettingsNameInput.value).toBe('Ice')
    expect(s.exportSettingsSaveError.value).toBeNull()
  })

  it('a load error surfaces', async () => {
    const s = await fresh()
    getMock.mockResolvedValue({ name: null, error: ipcError })
    await s.loadExportSettingsForm()
    expect(s.exportSettingsSaveError.value).toEqual(ipcError)
  })

  it('save sends the input verbatim and marks saved', async () => {
    const s = await fresh()
    saveMock.mockResolvedValue({ ok: true, error: null })
    s.exportSettingsNameInput.value = '  Ice  '
    await s.saveExportSettings()
    expect(saveMock).toHaveBeenCalledWith('  Ice  ')
    expect(s.exportSettingsSaved.value).toBe(true)
  })

  it('a save error surfaces and saved stays false', async () => {
    const s = await fresh()
    saveMock.mockResolvedValue({ ok: false, error: ipcError })
    s.exportSettingsNameInput.value = 'Ice'
    await s.saveExportSettings()
    expect(s.exportSettingsSaveError.value).toEqual(ipcError)
    expect(s.exportSettingsSaved.value).toBe(false)
  })
})
