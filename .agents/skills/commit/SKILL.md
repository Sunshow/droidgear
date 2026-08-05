---
name: commit
description: Commit staged changes with conventional commit message. Use when user wants to commit code changes.
---

# Commit Changes

## Instructions

1. Check current status:

   ```bash
   git status --porcelain
   ```

2. If no staged changes, stage all modified files:

   ```bash
   git add -A
   ```

3. Review changes to be committed:

   ```bash
   git diff --cached --stat
   git diff --cached
   ```

4. **Security check**: Scan diff for sensitive data:
   - API keys, tokens, passwords
   - Private keys, certificates
   - Environment secrets
   - If detected, STOP and warn user

5. Analyze changes and determine commit type:
   - `feat:` - New feature or functionality
   - `fix:` - Bug fix
   - `chore:` - Maintenance, dependencies, release
   - `docs:` - Documentation only
   - `style:` - Code formatting, no logic change
   - `refactor:` - Code restructure without behavior change
   - `test:` - Adding or updating tests

6. Generate commit message:
   - Format: `<type>: <description>`
   - Description should be concise (under 72 chars)
   - Focus on "what" and "why", not "how"
   - Use lowercase, no period at end
   - Examples from this repo:
     - `feat: openclaw support streaming settings`
     - `fix: openclaw profile apply policy`
     - `docs: update changelog for v0.3.2`
     - `style: format Rust code`

7. Execute commit:

   ```bash
   git commit -m "<type>: <description>"
   ```

8. If commit fails due to pre-commit hooks:
   - Review hook output
   - Fix issues or include auto-fixed files
   - Retry commit once

## Verification

- Confirm `git status` shows clean working tree after commit
- Confirm commit message follows conventional format
- Confirm no sensitive data was committed
