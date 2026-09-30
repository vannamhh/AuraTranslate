/**
 * `CommandRegistry.dispatch` catches only synchronous throws, so a handler that runs
 * `void asyncFn()` leaks a rejection to `window`. This listener logs it with its cause.
 */
export function installUnhandledRejectionLog(): () => void {
  const listener = (event: Event): void => {
    const reason = (event as PromiseRejectionEvent).reason
    console.error('[unhandled-rejection] a rejected promise was not handled:', reason)
  }
  window.addEventListener('unhandledrejection', listener)
  return () => window.removeEventListener('unhandledrejection', listener)
}
