# Playground 产品结构与界面设计

Playground 是 DroidGear 里的模型测试台：选一个模型，发一次真实请求，查看结果。它只读取渠道，不写任何工具的配置文件。

**当前状态（P2）**：视频页已接入真实请求，支持两种上游协议：

- **New API Videos**：`/v1/videos`、`/v1/videos/{id}`、`/v1/videos/{id}/content`。
- **Doubao / Ark**：`/api/v3/contents/generations/tasks` 系列任务接口。通过 New API 渠道访问时会自动加 `/doubao` 前缀。

模型可以从渠道导入，也可以手填任意模型 ID；每个模型单独选择协议。模型和任务记录都保存在 `~/.droidgear/playground/`，重启后恢复。

相关文档：

- See [ui-patterns.md](./ui-patterns.md)
- See [state-management.md](./state-management.md)
- See [i18n-patterns.md](./i18n-patterns.md)

## 范围

| 项     | 结论                                                                                 |
| ------ | ------------------------------------------------------------------------------------ |
| 平台   | 只做桌面 GUI。TUI 暂不支持，这是对 AGENTS.md 规则 9（GUI/TUI 同步）的有意例外        |
| 写配置 | 只写 `~/.droidgear/playground/`，不修改 `~/.factory`、`~/.codex` 等任何工具配置      |
| 协议   | `new-api-videos` 与 `doubao`，见第 2 节                                              |
| P2     | 视频页：渠道导入、协议选择、创建/查询/下载、进度刷新、`<video>` 预览、任务记录持久化 |

## 1. 导航

```
┌──────────────────────────────────┐
│ [渠道]  [Dsh ▾]  [⚗]              │  ⚗ = Playground，currentView = 'playground'
├──────────────────────────────────┤
│  视频                          2  │  2 = 进行中的任务数
│                                  │
│ ──────────────────────────────── │
│ 用真实请求测试模型。                │
└──────────────────────────────────┘
```

- Playground 是第三个顶级视图，和“渠道”、工具切换并列。入口是固定宽度的图标按钮，悬停显示 “Playground”，这样不挤压工具名。
- 不放进工具下拉菜单，因为下拉里每一项都是配置写入目标。
- 在 Playground 时，工具按钮和在“渠道”时一样，点击直接回到上次的工具；进入 Playground 不改变 `lastToolView`。
- 有进行中的任务且当前不在 Playground 时，图标右上角显示小圆点。
- 后续场景（图像、对话等）在侧栏列表追加，未实现的不放占位。

## 2. 协议

每个视频模型带一个 `protocol` 字段，创建、查询、下载都按它选择客户端。

| 协议             | 创建                                            | 查询                                                | 内容                                |
| ---------------- | ----------------------------------------------- | --------------------------------------------------- | ----------------------------------- |
| `new-api-videos` | `POST {base}/v1/videos`                         | `GET {base}/v1/videos/{id}`                         | `GET {base}/v1/videos/{id}/content` |
| `doubao`         | `POST {base}/api/v3/contents/generations/tasks` | `GET {base}/api/v3/contents/generations/tasks/{id}` | 查询响应里的 `content.video_url`    |

### 2.1 New API Videos

请求体：

```json
{
  "model": "doubao-seedance-2-0-260128",
  "prompt": "一只橘猫在阳光下钓路亚",
  "seconds": 5,
  "size": "1280x720",
  "input_reference": "https://example.com/cat.png"
}
```

- `seconds`、`size`、`input_reference` 都是可选的；界面里留空就不传。
- 高级参数 JSON 会合并进请求体，但 `model` 和 `prompt` 以表单为准。
- 创建和查询响应按 OpenAI Video 对象解析：`id`、`status`（`queued`/`in_progress`/`completed`/`failed`）、`progress`、`error`。
- Base URL 归一化：根地址补 `/v1/videos`，已经以 `/v1` 结尾的补 `/videos`，已经是 `/v1/videos` 的原样使用。
- 内容端点需要 Bearer 鉴权，WebView 的 `<video>` 无法带请求头，所以由 Rust 下载到本地后通过 asset 协议播放。

### 2.2 Doubao / Ark

请求体：

```json
{
  "model": "doubao-seedance-2-0-260128",
  "content": [
    {
      "type": "image_url",
      "image_url": { "url": "https://example.com/cat.png" }
    },
    { "type": "text", "text": "一只橘猫在阳光下钓路亚" }
  ],
  "duration": 5,
  "resolution": "720p",
  "ratio": "16:9"
}
```

- `content` 里参考图在前、文本在后，和 new-api 的 doubao 插件一致。
- `duration` 对应界面时长；`resolution` 对应界面分辨率。界面默认给 `480p`/`720p`/`1080p` 这类 Ark 档位。
- 高级参数 JSON 会合并进请求体，适合放 `ratio`、`seed`、`watermark`、`generate_audio` 等；`model` 和 `content` 不能被覆盖。
- 查询状态：`queued`、`running`、`succeeded`、`failed`、`expired`、`cancelled`，统一映射成 Playground 的五种状态。
- 完成后直接使用 `content.video_url`，通常是可以匿名访问的签名地址。

### 2.3 路径前缀

- **New API 渠道**：doubao 插件在 type-60 New API 渠道下把原生 Ark 路径挂在 `/doubao` 下。因此渠道类型为 `new-api` 时，Playground 自动请求 `{channel.baseUrl}/doubao/api/v3/contents/generations/tasks`。如果渠道 Base URL 已经以 `/doubao` 结尾，不会重复添加。
- **自定义地址**：原样使用。如果目标是 New API 兼容网关，需要自己把 `/doubao` 写进 Base URL（例如 `https://gateway.example/doubao`）。
- `new-api-videos` 始终是宿主协议，不加 `/doubao`。

### 2.4 状态映射

| Playground | New API Videos             | Doubao               |
| ---------- | -------------------------- | -------------------- |
| 排队中     | `queued`                   | `queued`             |
| 生成中     | `in_progress`              | `running`            |
| 已完成     | `completed`                | `succeeded`          |
| 失败       | `failed`                   | `failed` / `expired` |
| 已取消     | `failed`（上游无取消接口） | `cancelled`          |

- Doubao 没有百分比进度：排队 0%，生成中 50%，终态 100%。
- New API 返回 `progress` 时直接使用，终态固定 100%。
- 取消按钮只改本地状态，不会取消上游任务；New API 没有取消接口，本地取消后也不再轮询。

## 3. 视频页布局

```
┌──────────────────────────────────────────────────────────────────────┐
│ 视频生成                                             [+ 添加视频模型] │
│ 对接 New API Videos 或 Doubao 渠道…                                     │
├──────────────────┬───────────────────┬───────────────────────────────┤
│ 模型      [管理]  │ [全部|进行中|已完成|失败] [⟳] │ [生成中] task_xxxx ⧉  [查询状态][取消][🗑] │
│ [Seedance    ▾]  │ ◉ 自动刷新 [每 5 秒▾]  更新于… ├───────────────────────────────┤
│ 提示词            ├───────────────────┤                               │
│ […………………]        │ ⟳ 一只橘猫…       │        <video controls>        │
│ 时长 [5s▾] 分辨率 │   Seedance · 40%  │                               │
│ 参考图 […]        │   ━━━━░░░         ├───────────────────────────────┤
│ ▸ 高级参数 (JSON) │ ✗ A drone shot…   │ 提示词全文           [载入参数] │
│                  │ ✓ 微距镜头下…      │ 模型 / 协议 / 时长 / 分辨率 /   │
├──────────────────┤                   │ 创建时间 / 耗时 / 高级参数      │
│ 可能产生费用 [创建任务] │                │                               │
└──────────────────┴───────────────────┴───────────────────────────────┘
     创建区 32%           任务列表             任务详情（三栏均可拖拽）
```

| 区域     | 内容与规则                                                                                                        |
| -------- | ----------------------------------------------------------------------------------------------------------------- |
| 页头     | 标题、说明、“添加视频模型”主入口                                                                                  |
| 创建区   | 模型下拉（末项“添加视频模型”）、提示词、时长、分辨率、参考图（仅模型支持时显示）、高级参数 JSON；提交区固定在底部 |
| 任务列表 | 状态筛选、立即刷新、自动刷新开关和间隔；新任务在最上方并自动选中                                                  |
| 任务详情 | 状态、任务 ID（点击复制）、操作按钮、预览区、参数回显                                                             |

## 4. 交互

### 4.1 视频模型

- 入口有三处：页头按钮、模型下拉末项“添加视频模型”、模型标签旁的“管理”。“管理”打开模型列表对话框，可编辑和删除。
- 添加对话框字段：显示名称、模型 ID、**视频协议**、来源（渠道 / 自定义地址）、可选时长、可选分辨率、是否支持参考图。
- 模型 ID 是自由文本，不做白名单校验；导入时也不过滤 `/v1/models` 的返回结果。
- 协议决定路径、请求体和状态映射；切换协议会把仍在使用默认值的时长/分辨率换成新协议的默认值。
- 来源选“渠道”时只列已启用的渠道。手工选择渠道后使用该渠道第一个启用令牌；需要指定令牌时用“从渠道导入”。
- 来源选“自定义地址”时填 Base URL 和 API Key。API Key 不写进模型文件，单独存到 `~/.droidgear/playground/video_secrets.json`（0600）。
- 时长和分辨率用逗号分隔填写，作为创建表单里的下拉选项；下拉第一项固定为“默认”，表示不传该参数。
- 删除模型不影响已有任务：任务保存了创建时的模型快照。

### 4.2 从渠道导入

三步：渠道 → 令牌 → 模型。

1. 只列已启用渠道。
2. API Key 类型的渠道（CLI Proxy API、Ollama、General、DeepSeek）直接读已保存的 Key；其他渠道列出令牌，选一个。
3. 用令牌请求渠道的 `/v1/models`，展示全部模型，支持搜索、全选和批量选择。
4. 顶部选择本次导入的默认协议，每一行可以单独改成另一个协议；导入后仍可逐个编辑。

导入不复制渠道密钥：模型只保存 `channelId` 和 `tokenId`，调用时再从渠道存储解析。这样换令牌或改 Base URL 后不用重新导入。代价是每次调用可能需要登录渠道换令牌，Rust 侧对令牌做了 5 分钟内存缓存。

### 4.3 创建任务

- 模型和提示词必填；高级参数不是 JSON 对象时，提交按钮禁用并在输入框下方提示。
- `⌘/Ctrl+Enter` 提交。提交期间按钮显示“创建中…”，成功后表单不清空、不锁定，可以改参数继续提交，多个任务并行。
- 创建失败不插入任务，只弹错误提示；错误信息来自上游（HTTP 状态 + 响应体里的 `error.message`）。
- 切换模型时重置时长和分辨率，因为不同模型的可选项不同。

### 4.4 进度刷新

| 方式     | 行为                                                                |
| -------- | ------------------------------------------------------------------- |
| 自动刷新 | 默认开启，间隔可选 3 / 5 / 10 / 30 秒；没有进行中的任务时不发起轮询 |
| 全部刷新 | 任务列表右上角按钮，查询所有进行中的任务                            |
| 单个刷新 | 任务详情的“查询状态”，查询当前任务；已完成任务也可用来重试内容下载  |

列表头显示“更新于 hh:mm:ss”，刷新中显示加载图标并禁用刷新按钮，避免重复请求。任务进入终态后不再自动轮询。

任务记录在创建、状态变化、取消和删除时写入 `video_tasks.json`。重启后任务列表会恢复；进行中的任务继续轮询，已完成的任务直接显示本地视频或远程 URL。

### 4.5 任务状态

| 状态   | 列表行            | 详情预览区              | 详情操作                       |
| ------ | ----------------- | ----------------------- | ------------------------------ |
| 排队中 | 时钟图标          | 加载动画 + “排队等待中” | 查询状态、取消、删除           |
| 生成中 | 旋转图标 + 进度条 | 加载动画 + 百分比进度条 | 查询状态、取消、删除           |
| 已完成 | 绿色对勾          | `<video controls>` 播放 | 查询状态、外部打开、重跑、删除 |
| 失败   | 红色叹号          | 错误信息                | 重跑、删除                     |
| 已取消 | 禁止图标          | “已取消”                | 重跑、删除                     |

- 视频加载失败（如链接过期）时，预览区显示“视频加载失败，链接可能已过期”。
- New API 内容下载失败时任务仍是“已完成”，预览区显示下载错误；可以点“查询状态”重试。
- “重新生成”用相同参数和模型快照新建任务；“载入参数”把该任务的模型和参数回填到创建表单。

## 5. 代码结构

```
src-tauri/crates/droidgear-core/src/video.rs   # 类型、持久化、凭据解析、两种协议客户端
src-tauri/src/commands/video.rs                # Tauri 命令包装
src/store/playground-store.ts                  # 视频模型、任务、草稿、刷新设置
src/components/playground/
├── PlaygroundFeatureList.tsx                  # 侧栏
└── video/
    ├── VideoPlaygroundPage.tsx                # 页面骨架与三栏布局
    ├── VideoCreatePanel.tsx                   # 创建区
    ├── VideoTaskList.tsx                      # 任务列表、筛选、自动与手动刷新
    ├── VideoTaskDetail.tsx                    # 任务详情与 <video> 预览
    ├── VideoTaskStatus.tsx                    # 状态图标、徽标、进度条
    ├── VideoModelDialog.tsx                   # 添加 / 编辑视频模型
    ├── VideoModelImportDialog.tsx             # 渠道 → 令牌 → 模型批量导入
    ├── VideoModelManagerDialog.tsx            # 模型列表管理
    ├── video-protocol.ts                      # 协议默认值、导入结果映射
    └── video-format.ts                        # 时间格式化、列表解析、JSON 校验
```

- 视图接入：`ui-store.ts` 的 `NavigationView` 增加 `'playground'`（不加入 `ToolView`）；`LeftSideBar.tsx` 加入口按钮和侧栏分支；`MainWindowContent.tsx` lazy 加载 `VideoPlaygroundPage`。
- 状态必须放在 store：页面是 lazy 加载的，切换视图会卸载组件。草稿、任务和刷新设置放在组件 `useState` 里会在切出后丢失。
- 自动刷新的定时器在 `VideoTaskList` 里，离开 Playground 页面后暂停，回到页面后恢复。

## 6. 持久化与媒体

| 内容           | 位置                                         | 说明                                          |
| -------------- | -------------------------------------------- | --------------------------------------------- |
| 视频模型       | `~/.droidgear/playground/video_models.json`  | 普通 JSON；自定义来源的 API Key 会被清空      |
| 自定义 API Key | `~/.droidgear/playground/video_secrets.json` | 0600，按模型 ID 存；读模型时合并回去          |
| 下载的视频     | `~/.droidgear/playground/media/{taskId}.mp4` | New API 内容；同名文件存在时直接复用          |
| 任务           | `~/.droidgear/playground/video_tasks.json`   | 模型快照 + 参数 + 状态；自定义 API Key 不落盘 |

- New API 内容通过 asset 协议播放。`tauri.conf.json` 需要 `assetProtocol.enable = true`，scope 限定在 `$HOME/.droidgear/playground/media/**`，CSP 的 `media-src` 需要包含 `asset: http://asset.localhost`。
- 下载先写 `.part` 临时文件，完成后 rename；失败会清理临时文件。
- 媒体目录不会自动清理。后续可以按任务删除或按 TTL 清理。

## 7. 已知限制

- 进行中任务的进度不会每次轮询都落盘；重启后以新的查询结果为准。
- 删除自定义模型后，其历史任务在重启后不能再查询（自定义 API Key 不会复制到任务文件）；已经下载的视频仍可播放。
- 取消是本地状态，不会通知上游。
- 只支持 JSON 请求，不支持 multipart 上传参考图；参考图必须是可访问的 URL（或 data URL，取决于上游）。
- New API Videos 协议只发送标准 OpenAI 字段。new-api 的 doubao 插件不会把 `input_reference`/`size` 映射到 Ark（`size` 只参与计费校验）；需要参考图或精确分辨率时，请对 doubao 模型选择 Doubao 协议。
- New API 内容必须先下载才能播放，首次完成时会多花一次下载时间。
- 没有做模型能力探测：模型是否支持参考图、哪些时长/分辨率合法，由用户配置。
- TUI 未实现；这是有意例外。
