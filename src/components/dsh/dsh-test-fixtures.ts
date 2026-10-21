import type {
  DshConfigEntryView,
  DshConfigFieldView,
  DshConfigSnapshot,
  DshMigrationWarning,
} from '@/lib/bindings'

/** Build one field view with sensible defaults for tests. */
export function makeField(
  overrides: Partial<DshConfigFieldView> & { name: string }
): DshConfigFieldView {
  return {
    kind: 'string',
    value: null,
    default: null,
    enumValues: [],
    volatile: true,
    secret: false,
    description: '',
    overridden: false,
    tier: 'A',
    ...overrides,
  }
}

/** Build one composed entry view with sensible defaults for tests. */
export function makeEntry(
  overrides: Partial<DshConfigEntryView> & { id: string }
): DshConfigEntryView {
  return {
    packageName: `@deepseek-ai/${overrides.id}`,
    layer: 'bundle:@deepseek-ai/dsh-base',
    tier: 'A',
    description: '',
    disabled: false,
    hasJs: false,
    config: {},
    overridden: false,
    shadowedByHome: false,
    fields: [],
    ...overrides,
  }
}

/** Build one migration warning for tests. */
export function makeWarning(
  overrides: Partial<DshMigrationWarning> & { code: string }
): DshMigrationWarning {
  return {
    severity: 'warning',
    entryId: null,
    message: `${overrides.code} message`,
    hint: `${overrides.code} hint`,
    ...overrides,
  }
}

/** Build a composed snapshot for tests. */
export function makeSnapshot(
  overrides: Partial<DshConfigSnapshot> = {}
): DshConfigSnapshot {
  return {
    schemaVersion: '0.2.1-rc.2',
    installedVersion: '0.2.1-rc.2',
    profile: 'desktop',
    profilePatchPath: '/home/user/.dsh/profiles/desktop/cordis.patch.yml',
    homePatchPath: '/home/user/.dsh/cordis.patch.yml',
    homePatchExists: false,
    legacySettingsPath: '/home/user/.dsh/settings.yaml.imported',
    legacySettingsExists: false,
    patchHash: 'abc123',
    entries: [],
    warnings: [],
    ...overrides,
  }
}
