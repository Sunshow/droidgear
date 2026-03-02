# 自定义配置路径 / Custom Configuration Paths

DroidGear 支持自定义 Droid/Factory、OpenCode 和 Codex 的配置目录路径，这对于 WSL 用户或需要跨系统共享配置的用户特别有用。

DroidGear supports customizing configuration directory paths for Droid/Factory, OpenCode, and Codex. This is especially useful for WSL users or those who need to share configurations across systems.

## 默认路径 / Default Paths

| 工具 / Tool     | 默认路径 / Default Path    |
| --------------- | -------------------------- |
| Droid/Factory   | `~/.factory/`              |
| OpenCode Config | `~/.config/opencode/`      |
| OpenCode Auth   | `~/.local/share/opencode/` |
| Codex           | `~/.codex/`                |
| OpenClaw        | `~/.openclaw/`             |

## 如何设置 / How to Configure

1. 打开 **Preferences** (Mac: `Cmd+,` / Windows: `Ctrl+,`)
2. 选择 **Paths** 面板
3. 点击 📁 图标浏览选择目录，或直接输入路径
4. 点击 **Save** 保存

1. Open **Preferences** (Mac: `Cmd+,` / Windows: `Ctrl+,`)
2. Select the **Paths** pane
3. Click the 📁 icon to browse for a directory, or type the path directly
4. Click **Save** to apply

## WSL 用户指南 / WSL User Guide

如果你在 Windows 主机上运行 DroidGear，但需要编辑 WSL2 内部的配置，可以将配置路径指向 WSL 文件系统：

If you're running DroidGear on Windows host but need to edit configurations inside WSL2, you can point configuration paths to the WSL filesystem:

```
Factory: \\wsl$\Ubuntu\home\username\.factory
OpenCode Config: \\wsl$\Ubuntu\home\username\.config\opencode
OpenCode Auth: \\wsl$\Ubuntu\home\username\.local\share\opencode
Codex: \\wsl$\Ubuntu\home\username\.codex
OpenClaw: \\wsl$\Ubuntu\home\username\.openclaw
```

将 `Ubuntu` 替换为你实际的 WSL 发行版名称（可通过 `wsl -l` 查看），`username` 替换为 WSL 内的用户名。

Replace `Ubuntu` with your actual WSL distro name (check with `wsl -l`), and `username` with your WSL username.

### 获取 WSL 用户名 / Get WSL Username

```bash
wsl whoami
```

### 验证路径 / Verify Path

在 Windows 资源管理器地址栏输入 `\\wsl$\Ubuntu` 验证是否可以访问。

Type `\\wsl$\Ubuntu` in Windows Explorer address bar to verify access.

## 配置存储 / Configuration Storage

自定义路径存储在 `~/.droidgear/settings.json` 的 `configPaths` 字段中：

Custom paths are stored in the `configPaths` field of `~/.droidgear/settings.json`:

```json
{
  "configPaths": {
    "factory": "/mnt/c/Users/YourName/.factory",
    "opencode": "/mnt/c/Users/YourName/.config/opencode"
  }
}
```

**注意 / Notes:**

- 只有明确设置的路径才会写入配置文件
- 未设置的路径使用默认值
- 重置路径会从配置中删除该条目

- Only explicitly set paths are written to the config file
- Unset paths use default values
- Resetting a path removes it from the config

## 重置为默认 / Reset to Default

在 Paths 设置面板中，点击 **Reset** 按钮可将单个路径重置为默认值。

In the Paths settings pane, click the **Reset** button to restore a single path to its default value.

## 注意事项 / Important Notes

1. **重启生效**: 更改配置路径后，可能需要重启应用才能完全生效
2. **目录权限**: 确保 DroidGear 对自定义目录有读写权限
3. **符号链接**: 支持符号链接路径

4. **Restart Required**: After changing configuration paths, you may need to restart the app for changes to take full effect
5. **Directory Permissions**: Ensure DroidGear has read/write access to custom directories
6. **Symlinks**: Symbolic link paths are supported
