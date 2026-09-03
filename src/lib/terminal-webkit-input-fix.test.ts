import { describe, expect, it } from 'vitest'
import {
  installWebkitBeforeInputFix,
  type XtermWebkitFixTarget,
} from './terminal-webkit-input-fix'

function createTarget() {
  const textarea = document.createElement('textarea')
  const core = { _keyDownSeen: true }
  const target: XtermWebkitFixTarget = { textarea, _core: core }
  return { textarea, core, target }
}

function fireBeforeInput(
  textarea: HTMLTextAreaElement,
  init: Partial<InputEventInit> = {}
) {
  textarea.dispatchEvent(
    new InputEvent('beforeinput', {
      bubbles: true,
      cancelable: true,
      composed: true,
      data: 'a',
      inputType: 'insertText',
      isComposing: false,
      ...init,
    })
  )
}

describe('installWebkitBeforeInputFix', () => {
  it('resets _keyDownSeen for composed insertText events', () => {
    const { textarea, core, target } = createTarget()
    const dispose = installWebkitBeforeInputFix(target)

    fireBeforeInput(textarea)
    expect(core._keyDownSeen).toBe(false)

    dispose?.()
  })

  it('ignores IME composition events', () => {
    const { textarea, core, target } = createTarget()
    const dispose = installWebkitBeforeInputFix(target)

    fireBeforeInput(textarea, { isComposing: true })
    expect(core._keyDownSeen).toBe(true)

    dispose?.()
  })

  it('ignores non-insertText input types', () => {
    const { textarea, core, target } = createTarget()
    const dispose = installWebkitBeforeInputFix(target)

    fireBeforeInput(textarea, { inputType: 'insertCompositionText' })
    expect(core._keyDownSeen).toBe(true)

    fireBeforeInput(textarea, { inputType: 'insertFromPaste' })
    expect(core._keyDownSeen).toBe(true)

    dispose?.()
  })

  it('ignores non-composed events', () => {
    const { textarea, core, target } = createTarget()
    const dispose = installWebkitBeforeInputFix(target)

    fireBeforeInput(textarea, { composed: false })
    expect(core._keyDownSeen).toBe(true)

    dispose?.()
  })

  it('removes the listener when disposed', () => {
    const { textarea, core, target } = createTarget()
    const dispose = installWebkitBeforeInputFix(target)

    dispose?.()
    fireBeforeInput(textarea)
    expect(core._keyDownSeen).toBe(true)
  })

  it('returns undefined when textarea or core is missing', () => {
    expect(
      installWebkitBeforeInputFix({ _core: { _keyDownSeen: true } })
    ).toBeUndefined()

    const textarea = document.createElement('textarea')
    expect(installWebkitBeforeInputFix({ textarea })).toBeUndefined()
  })
})
