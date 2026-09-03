/**
 * xterm.js 在 macOS WebKit(WKWebView,即 Safari 内核,Tauri macOS 端使用)下的
 * 输入丢失问题修复。
 *
 * 根因:macOS 上按下 Shift / Caps Lock 或使用中文输入法时,WebKit 可能在字符
 * 自身的 keydown 之前派发 beforeinput/input。此时 xterm 的 `_keyDownSeen` 仍
 * 被前一个修饰键 keydown 置为 true,于是 xterm 把真实的 input 事件当作重复
 * 事件丢弃,导致首个字符丢失(如输入法下 Shift+3 的 "#"、Caps Lock 后的首字母)。
 *
 * 修复:监听 beforeinput,对 composed 且非 IME 组合中的 insertText 事件,先把
 * `_keyDownSeen` 重置为 false,让随后的 input 事件能被 xterm 正常处理。
 *
 * @see https://github.com/xtermjs/xterm.js/issues/5374#issuecomment-5390337647
 */

export interface XtermWebkitFixTarget {
  /** xterm 内部的 CoreBrowserTerminal,仅访问其输入状态标记 */
  _core?: {
    _keyDownSeen: boolean
  }
  /** 终端用于捕获键盘输入的隐藏 textarea */
  textarea?: HTMLTextAreaElement
}

export function installWebkitBeforeInputFix(
  target: XtermWebkitFixTarget
): (() => void) | undefined {
  const textarea = target.textarea
  const core = target._core
  if (!textarea || !core) {
    return undefined
  }

  const handleBeforeInput = (event: InputEvent): void => {
    if (
      !event.composed ||
      !event.data ||
      event.inputType !== 'insertText' ||
      event.isComposing
    ) {
      return
    }
    // WebKit 可能在字符自身 keydown 前派发 beforeinput/input,此时
    // _keyDownSeen 仍被前一个修饰键 keydown 置为 true,xterm 会把真实
    // 输入当作重复事件丢弃。这里重置标记,让随后的 input 被正常处理。
    core._keyDownSeen = false
  }

  textarea.addEventListener('beforeinput', handleBeforeInput)
  return () => {
    textarea.removeEventListener('beforeinput', handleBeforeInput)
  }
}
