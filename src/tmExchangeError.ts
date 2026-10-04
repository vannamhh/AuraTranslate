import { t, tError } from './i18n'
import type { IpcError } from './i18n'

const TMX_ERROR_KEYS: Readonly<Partial<Record<string, string>>> = {
  'tm.tmx_malformed': 'tm.exchange.err_malformed',
  'tm.tmx_no_body': 'tm.exchange.err_no_body',
  'tm.tmx_not_utf8': 'tm.exchange.err_not_utf8',
  'tm.tmx_too_large': 'tm.exchange.err_too_large',
  'tm.tmx_no_usable_pair': 'tm.exchange.err_no_usable_pair',
  'tm.tmx_read_failed': 'tm.exchange.err_read_failed',
  'tm.tmx_write_failed': 'tm.exchange.err_write_failed',
  'tm.no_pending_import': 'tm.exchange.err_no_pending',
  'tm.dialog_path_invalid': 'tm.exchange.err_dialog_path',
  'work.none_open': 'tm.exchange.err_work_none_open',
}

/** The TMX exchange commands carry no `message_key`; the webview maps their codes. */
export function tmExchangeErrorText(err: IpcError): string {
  const lang = err.code === 'tm.tmx_no_usable_pair' ? err.params.source_lang : undefined
  if (typeof lang === 'string' && lang !== '') return t('tm.exchange.err_no_usable_pair_lang', err.params)
  const key = TMX_ERROR_KEYS[err.code]
  return key === undefined ? tError(err) : t(key, err.params)
}
