# GitHub Copilot BYOK

DroidGear selects the provider and model **before starting Copilot**. Changing a
profile does not switch the model in an existing CLI session; start another
session with that profile. The environment-based launch mechanism follows
[copilot-cli-local](https://github.com/Dark-Athena/copilot-cli-local).

## Desktop use

1. Select **Copilot → Profiles** in the tool sidebar.
2. Create a profile or edit the default profile. For BYOK, select **Import from
   Channel** and choose a channel, API key, and model, or enter them manually.
   Import fills the editor; save or run when ready. Supported provider types are
   **OpenAI** and **Anthropic**. Token limits are optional positive integers.
3. Select **Temporary run**, choose the project directory, and Copilot opens in
   the preferred terminal with that profile's environment and an explicit
   `--model <model>` argument, so the configured model is selected at startup.

OpenAI base URLs must end in `/v1`, including Sub2API URLs. Anthropic base URLs
must omit the trailing `/v1`. Importing, saving, applying, and launching normalize
these paths automatically, preserving any gateway prefix. For example,
`https://api.example.com/gateway` becomes `https://api.example.com/gateway/v1` for
OpenAI, while `https://api.example.com/gateway/v1` becomes
`https://api.example.com/gateway` for Anthropic.

Install the GitHub Copilot CLI so `copilot` is available on your terminal's PATH.
Native installations (including Homebrew) and npm installations are supported.
For example:

```sh
npm install -g @github/copilot
```

A profile can also use **GitHub Copilot official subscription** mode. That mode
clears BYOK overrides and uses the CLI's existing login or authentication token.
Provider and model availability depend on the installed Copilot CLI version and
the selected provider.

## Saving and applying

| Action              | Result                                                                                   |
| ------------------- | ---------------------------------------------------------------------------------------- |
| Save profile        | Save the editable profile and its exported `.env` file                                   |
| Temporary run       | Save the profile, then launch with process-scoped settings                               |
| Apply to local file | Save and export the profile to the standalone launcher's `config.env`                    |
| Load from config    | Replace the selected profile's provider fields with values from the applied `config.env` |

Temporary runs leave the applied `config.env` and active-profile marker unchanged.
Profile files live in DroidGear's storage:

```text
~/.droidgear/copilot/
├── profiles/
│   ├── <id>.json
│   └── <id>.env
├── config.env
└── active-profile.txt
```

Saving edits to an applied profile does not refresh `config.env`; select Apply
again when ready. Deleting the profile clears its active marker and removes its
profile files; the last exported `config.env` remains available to the launcher.
Files containing API keys are written atomically with owner-only permissions on
Unix. Tauri launches reuse the existing terminal launcher, passing the key through
its secret environment mechanism.

## Terminal UI and CLI

Open the Copilot section in `droidgear-tui`. Use `n` to create, `e` to edit the
profile JSON with the configured editor, `c` to copy, `d` to delete, `a` to apply,
`l` to load the applied configuration, `i` to import a channel model, and Enter
or `t` to run. Channel import selects the channel, API key, and model, then saves
the current profile without applying it. Multi-protocol channels also offer a
protocol choice. Runs use the current working directory.

```sh
droidgear-tui run copilot --list
droidgear-tui run copilot 1
droidgear-tui run copilot "My provider"
droidgear-tui run copilot <profile-id>
```

Indices are one-based. Exact ids take priority over names; ambiguous names
require an id or index. Listing profiles never prints API keys.

## Standalone launcher

After applying a profile, run the wrapper from this repository:

```sh
# macOS / Linux
./copilot-local.sh
./copilot-local.sh --model another-model
./copilot-local.sh --config
./copilot-local.sh -- --version

# Windows
copilot-local.cmd
copilot-local.cmd --model another-model
```

Both wrappers use `scripts/copilot-local.js`; keep that file with the wrappers.
Node.js must be on PATH, or set `NODE_EXE` in the environment to its executable.
The launcher looks for a Copilot executable on PATH, then the global npm entry.
`COPILOT_ENTRY` can override this with a custom `npm-loader.js` path in the
environment or configuration file.

Configuration file priority:

1. `COPILOT_CONFIG_FILE` environment variable (absolute path recommended).
2. `config.env` beside the wrapper, if present.
3. `~/.droidgear/copilot/config.env`.

To launch an exported profile without changing the applied profile:

```sh
COPILOT_CONFIG_FILE="$HOME/.droidgear/copilot/profiles/<id>.env" ./copilot-local.sh
```

Example BYOK configuration:

```dotenv
COPILOT_OFFLINE=true
COPILOT_PROVIDER_BASE_URL=https://api.example.com/v1
COPILOT_PROVIDER_TYPE=openai
COPILOT_PROVIDER_API_KEY=your-key
COPILOT_MODEL=your-model
# COPILOT_PROVIDER_MAX_PROMPT_TOKENS=128000
# COPILOT_PROVIDER_MAX_OUTPUT_TOKENS=8192
```

Use `COPILOT_OFFLINE=false` for official subscription mode. Config values are
literal, unquoted text; they are never evaluated as shell code. Blank lines and
lines starting with `#` are ignored, values may contain `=`, and the last value
wins for duplicate keys. `--model` overrides the model for one run. `--config`
hides the API key. The launcher supplies both `COPILOT_MODEL` and `--model`.
It clears inherited BYOK settings, including provider model, wire-model,
credential, and protocol overrides, before loading the file. BYOK runs also
remove `COPILOT_AUTH_TOKEN` and set `COPILOT_OFFLINE=true`.
