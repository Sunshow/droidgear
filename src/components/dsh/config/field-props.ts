/** Props shared by every controlled settings field in the DSH config forms. */
export interface BaseFieldProps {
  /** Whether the control is disabled. */
  disabled?: boolean
  /** Marks the value as modified but not saved yet. */
  dirty?: boolean
  /** DOM id used to associate the control with its row label. */
  id?: string
}
