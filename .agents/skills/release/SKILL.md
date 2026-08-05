---
name: release
description: Generate changelog from commits since last tag and release new version. Use when user wants to release a new version or update changelog.
---

# Release New Version

## Instructions

1. Get the latest tag:

   ```bash
   git tag --list --sort=-version:refname | head -1
   ```

2. Get commits since last tag:

   ```bash
   git log <latest-tag>..HEAD --oneline
   ```

3. Calculate next version based on latest tag:
   - Parse current version vX.Y.Z from step 1
   - Default: increment patch (Z+1), e.g. v0.3.2 → v0.3.3
   - Minor: increment minor, reset patch (Y+1, Z=0), e.g. v0.3.2 → v0.4.0
   - Major: increment major, reset others (X+1, Y=0, Z=0), e.g. v0.3.2 → v1.0.0

4. Show suggested version to user, allow modification before proceeding

5. Categorize commits by type:
   - `feat:` → **New Features / 新功能**
   - `fix:` → **Bug Fixes / 问题修复**

6. Update CHANGELOG.md (insert new version entry after `# Changelog` heading):
   - Use bilingual format (English / 中文) for each item
   - Format: `## vX.Y.Z` followed by `**New Features / 新功能**` and `**Bug Fixes / 问题修复**` sections
   - Each item format: `- English description / 中文描述`
   - Translation terminology: "Channel" must be translated as "渠道" (NOT "频道")
   - This file is used by GitHub Actions to generate release notes for auto-updater

7. (Conditional) If this release includes significant new features or major changes, update the feature descriptions in both README files:
   - README.md — Update relevant feature sections in Chinese
   - README_EN.md — Update relevant feature sections in English
   - Only update feature/capability descriptions, NOT the changelog section (both READMEs already link to CHANGELOG.md for full changelog)

8. Commit changelog changes (required before release:prepare):

   ```bash
   # If README files were updated:
   git add CHANGELOG.md README.md README_EN.md
   # Otherwise:
   git add CHANGELOG.md
   git commit -m "docs: update changelog for <version>"
   ```

9. Run release workflow:

   ```bash
   npm run release:prepare <version>
   ```

10. Execute git commands to complete release:

    ```bash
    git add .
    git commit -m "chore: release <version>"
    git tag <version>
    git push origin main --tags
    ```

## Verification

- Confirm CHANGELOG.md is updated with English changelog (used for auto-updater)
- Confirm changelog entries are correctly formatted
- Confirm version number follows vX.Y.Z pattern
- Confirm both README files are updated if significant features were added
- Confirm changelog is committed before running release:prepare
- Confirm git push completes successfully
