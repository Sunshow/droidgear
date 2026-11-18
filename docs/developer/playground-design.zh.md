# Playground 产品结构与界面设计

Playground 是 DroidGear 里的模型测试台：选一个模型，发一次真实请求，查看结果。它只读取渠道，不写任何工具的配置文件。

**当前状态（P3）**：视频页和图片页都已接入真实请求。

视频页支持三种上游协议：

- **New API Sora / OpenAI Videos**：`/v1/videos`、`/v1/videos/{id}`、`/v1/videos/{id}/content`。
- **New API 原生**：`/v1/video/generations`、`/v1/video/generations/{task_id}`。
- **Doubao / Ark**：`/api/v3/contents/generations/tasks` 系列任务接口。通过 New API 渠道访问时会自动加 `/doubao` 前缀。

图片页支持三种上游协议：

- **ChatCompletions**：`/v1/chat/completions`，图片从 `message.images` 或 `message.content` 里的 markdown data URL 提取。
- **Responses**：`/v1/responses` + `image_generation` 工具，结果在 `image_generation_call.result`。
- **Google**：`/v1beta/models/{model}:generateContent`（`imagen*` 模型用 `:predict`）。

模型可以从渠道导入，也可以手填任意模型 ID；每个模型单独选择协议。模型和任务记录都保存在 `~/.droidgear/playground/`，重启后恢复。

相关文档：

- See [ui-patterns.md](./ui-patterns.md)
- See [state-management.md](./state-management.md)
- See [i18n-patterns.md](./i18n-patterns.md)

## 范围

| 项     | 结论                                                                                         |
| ------ | -------------------------------------------------------------------------------------------- |
| 平台   | 只做桌面 GUI。TUI 暂不支持，这是对 AGENTS.md 规则 9（GUI/TUI 同步）的有意例外                |
| 写配置 | 只写 `~/.droidgear/playground/`，不修改 `~/.factory`、`~/.codex` 等任何工具配置              |
| 协议   | 视频：`sora`、`new-api`、`doubao`（第 2 节）；图片：`chat`、`responses`、`google`（第 8 节） |
| P2     | 视频页：渠道导入、协议选择、创建/查询/下载、进度刷新、`<video>` 预览、任务记录持久化         |
| P3     | 图片页：渠道导入、协议选择、同步生成、多图结果、参考图、`<img>` 预览、任务记录持久化         |

## 1. 导航

```
┌──────────────────────────────────┐
│ [渠道]  [Dsh ▾]  [⚗]              │  ⚗ = Playground，currentView = 'playground'
├──────────────────────────────────┤
│  视频                          2  │  2 = 进行中的任务数
│  图片                          1  │  1 = 生成中的图片任务数
│                                  │
│ ──────────────────────────────── │
│ 用真实请求测试模型。                │
└──────────────────────────────────┘
```

- Playground 是第三个顶级视图，和“渠道”、工具切换并列。入口是固定宽度的图标按钮，悬停显示 “Playground”，这样不挤压工具名。
- 不放进工具下拉菜单，因为下拉里每一项都是配置写入目标。
- 在 Playground 时，工具按钮和在“渠道”时一样，点击直接回到上次的工具；进入 Playground 不改变 `lastToolView`。
- 有进行中的任务且当前不在 Playground 时，图标右上角显示小圆点。
- 侧栏在“视频”下面追加“图片”，切换的是 `ui-store` 的 `playgroundSubView`（`video` / `image`），不重新进入 Playground，也不改变 `lastToolView`。
- 后续场景（对话等）在侧栏列表追加，未实现的不放占位。

## 2. 协议

每个视频模型带一个 `protocol` 字段，创建、查询、下载都按它选择客户端。

| 协议      | 创建                                            | 查询                                                | 内容                                |
| --------- | ----------------------------------------------- | --------------------------------------------------- | ----------------------------------- |
| `sora`    | `POST {base}/v1/videos`                         | `GET {base}/v1/videos/{id}`                         | `GET {base}/v1/videos/{id}/content` |
| `new-api` | `POST {base}/v1/video/generations`              | `GET {base}/v1/video/generations/{task_id}`         | 查询响应里的 `url`（直接播放）      |
| `doubao`  | `POST {base}/api/v3/contents/generations/tasks` | `GET {base}/api/v3/contents/generations/tasks/{id}` | 查询响应里的 `content.video_url`    |

旧的 `new-api-videos` 是 `sora` 的别名：读取旧模型/任务时会自动映射成 `sora`，下一次保存写成 `sora`。

### 2.1 New API Sora / OpenAI Videos

请求体：

```json
{
  "model": "sora-2",
  "prompt": "一只橘猫在阳光下钓路亚",
  "seconds": "5",
  "size": "1280x720",
  "image": "https://example.com/cat.png"
}
```

- `seconds` 是字符串。OpenAI 的 `VideoSeconds` 只列了 `"4"`/`"8"`/`"12"`；Playground 允许手输，实际值由上游校验。
- `size` 是 `WxH`。OpenAI 只列了 `720x1280`/`1280x720`/`1024x1792`/`1792x1024`；Playground 允许手输。
- 单图发 `image` 字符串；多图发 `images` 数组。现在界面把它标成 **New API Sora**：它走 New API 的通用 `/v1/videos` 宿主协议，不再假设上游是已经关停的 OpenAI Sora。
- New API 的 sora 插件本身只读单张 `input_reference`/`image`，但各厂商插件（即梦、Vidu、阿里、豆包等）在同一个宿主协议上读取 `images` 数组：即梦映射到 `image_urls`，Vidu 映射到参考图，阿里映射到首尾帧/参考图，豆包映射到多个 `content` 图片条目。因此 Playground 不再限制一张。
- 多图是否真正生效仍由具体模型/插件决定；例如 Kling 的宿主协议只取 `image`/`image_tail`，可能不会消费 `images` 数组。
- Doubao/Seedance 模型的多图会走 `metadata.content` 并带 `role: "reference_image"`，避免 Ark 的 `role must be specified for image contents`；单图仍走 `images`（首帧推断）。
- 高级参数 JSON 会合并进请求体，但 `model`、`prompt`、`seconds`、`size`、`image`、`images` 以表单为准。
- 创建和查询响应按 OpenAI Video 对象解析：`id`、`status`（`queued`/`in_progress`/`completed`/`failed`）、`progress`、`error`。
- Base URL 归一化：根地址补 `/v1/videos`，已经以 `/v1` 结尾的补 `/videos`，已经是 `/v1/videos` 的原样使用。
- 内容端点需要 Bearer 鉴权，WebView 的 `<video>` 无法带请求头，所以由 Rust 下载到本地后通过 asset 协议播放。

### 2.2 New API 原生

单图请求体：

```json
{
  "model": "kling-v3",
  "prompt": "一只橘猫在阳光下钓路亚",
  "image": "https://example.com/cat.png",
  "duration": 5,
  "width": 1280,
  "height": 720,
  "seconds": "5",
  "size": "1280x720",
  "metadata": { "duration": 5, "resolution": "720p" }
}
```

多图时把 `image` 换成 `images`：

```json
{
  "model": "kling-v3",
  "prompt": "把两张参考图组合成一幅插画",
  "images": [
    "https://example.com/frame-a.png",
    "https://example.com/frame-b.png"
  ],
  "duration": 5,
  "width": 1280,
  "height": 720,
  "metadata": { "duration": 5, "resolution": "720p" }
}
```

- 单图用 `image`，多图用 `images`。new-api 的 `ValidateBasicTaskRequest` 会把单图 `image` 归一化进 `TaskSubmitReq.Images`；插件读的是 `images`。
- `duration`、`width`、`height` 是 new-api 原生文档字段。
- `seconds`、`size`、`metadata.duration`、`metadata.resolution` 是为了兼容 new-api 的 `TaskSubmitReq` 和 doubao 插件：插件提交钩子读 `seconds`、`images`，doubao 插件还会把 `metadata` 合并进 Ark 请求体。普通旧版适配器会忽略多余字段。
- 分辨率输入 `720p` 或 `1280x720` 都会拆成 `width`/`height`；无法解析时只把原值放进 `metadata.resolution`，由上游校验。
- Doubao/Seedance 模型的多图请求会改用 `metadata.content`，每张图带 `role: "reference_image"`。new-api 的 doubao 插件在把 `images` 转成 Ark content 时不会补 role，Ark 会返回 `role must be specified for image contents`；直接用 `images` 修复不了。单图仍走 `image`，语义是首帧。
- 高级参数 JSON 会合并进请求体，但表单字段优先。
- 创建响应是 `{ "task_id": "...", "status": "queued" }`。
- 查询响应实际是 `{ "code": "success", "data": { "task_id": "...", "status": "SUCCESS", "result_url": "https://...", ... } }`（New API 的 `TaskDto` 包装，不是文档示例里扁平的 `{ task_id, status, url }`）。`status` 为 `NOT_START`/`IN_PROGRESS`/`SUCCESS`/`FAILURE` 等，统一映射成 Playground 的状态。
- 完成后使用 `data.result_url`；代码同时兼容 `result_url`、`url`、`video_url` 和 `content.video_url`。没有单独的 content 端点，也不下载到本地。
- Base URL 归一化：根地址补 `/v1/video/generations`，已经以 `/v1` 结尾的补 `/video/generations`，已经是 `/v1/video/generations` 的原样使用。

### 2.3 Doubao / Ark

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

- `content` 里参考图在前、文本在后，和 new-api 的 doubao 插件一致；多张参考图就是多个 `image_url` 条目，具体张数上限由模型/上游决定。
- `duration` 对应界面时长；`resolution` 对应界面分辨率。输入 `WxH` 时会映射到最近的 Ark 档位（`480p`/`720p`/`1080p`/`4k`），输入档位原样发送。
- 高级参数 JSON 会合并进请求体，适合放 `ratio`、`seed`、`watermark`、`generate_audio` 等；`model` 和 `content` 不能被覆盖。
- 查询状态：`queued`、`running`、`succeeded`、`failed`、`expired`、`cancelled`，统一映射成 Playground 的五种状态。
- 完成后直接使用 `content.video_url`，通常是可以匿名访问的签名地址。

### 2.4 参考图映射

| 协议      | 单图                              | 多图                         |
| --------- | --------------------------------- | ---------------------------- |
| `sora`    | `image`                           | `images: [...]`              |
| `new-api` | `image`                           | `images: [...]`              |
| `doubao`  | `content` 里一个 `image_url` 条目 | `content` 里多个 `image_url` |

参考图在创建区改成宫格：点末尾的“+”逐张添加，单张可以来自 URL 或本地文件。本地文件由浏览器读成 base64 data URL，和 URL 一样进入 `image`/`images`/`content`；已添加的图片可以单独删除。多图不再在客户端限制张数，由上游模型/插件校验。

### 2.5 路径前缀

- **New API Sora**：始终是宿主协议，请求 `{base}/v1/videos`，不加 `/doubao`。
- **New API 原生**：请求 `{base}/v1/video/generations`，不加 `/doubao`。
- **New API 渠道 + Doubao**：doubao 插件在 type-60 New API 渠道下把原生 Ark 路径挂在 `/doubao` 下。因此渠道类型为 `new-api` 时，Playground 自动请求 `{channel.baseUrl}/doubao/api/v3/contents/generations/tasks`。如果渠道 Base URL 已经以 `/doubao` 结尾，不会重复添加。
- **自定义地址**：原样使用。如果目标是 New API 兼容网关，需要自己把 `/doubao` 写进 Base URL（例如 `https://gateway.example/doubao`）。

### 2.6 状态映射

| Playground | Sora                       | New API 原生               | Doubao               |
| ---------- | -------------------------- | -------------------------- | -------------------- |
| 排队中     | `queued`                   | `queued`                   | `queued`             |
| 生成中     | `in_progress`              | `in_progress`              | `running`            |
| 已完成     | `completed`                | `completed`                | `succeeded`          |
| 失败       | `failed`                   | `failed`                   | `failed` / `expired` |
| 已取消     | `failed`（上游无取消接口） | `failed`（上游无取消接口） | `cancelled`          |

- Doubao/New API 原生没有百分比进度时：排队 0%，生成中 50%，终态 100%。
- Sora 返回 `progress` 时直接使用，终态固定 100%。
- 取消按钮只改本地状态，不会取消上游任务；Sora/New API 原生没有取消接口，本地取消后也不再轮询。

## 3. 视频页布局

```
┌──────────────────────────────────────────────────────────────────────┐
│ 视频生成                              [模型 ▾] [管理] │
├──────────────────┬───────────────────┬───────────────────┤
│ 提示词            │ [全部|进行中|已完成|失败] [⟳] │ [生成中] task_xxxx ⧉  [查询状态][取消][🗑] │
│ […………………]        │ ◉ 自动刷新 [每 5 秒▾]  更新于… ├───────────────────┤
│ 时长 [5] 分辨率   ├───────────────────┤                   │
│ 参考图 […]        │ ⟳ 一只橘猫…       │   <video controls> │
│ ▸ 高级参数 (JSON) │   Seedance · 40%  │                   │
│                  │   ━━━━░░░         ├───────────────────┤
│                  │ ✗ A drone shot…   │ 提示词全文  [载入参数] │
│                  │ ✓ 微距镜头下…      │ 模型 / 协议 / 时长 / │
├──────────────────┤                   │ 创建时间 / 耗时 / 高级参数 │
│ 可能产生费用 [创建任务] │                │                   │
└──────────────────┴───────────────────┴───────────────────┘
     创建区 32%           任务列表             任务详情（三栏均可拖拽）
```

| 区域     | 内容与规则                                                                        |
| -------- | --------------------------------------------------------------------------------- |
| 页头     | 标题；右上角放模型下拉（末项“添加视频模型”）和管理                                |
| 创建区   | 提示词、时长、分辨率、参考图（仅模型支持时显示）、高级参数 JSON；提交区固定在底部 |
| 任务列表 | 状态筛选、立即刷新、自动刷新开关和间隔；新任务在最上方并自动选中                  |
| 任务详情 | 状态、任务 ID（点击复制）、操作按钮、预览区、参数回显                             |

## 4. 交互

### 4.1 视频模型

- 模型相关控件统一在页头右上角：模型下拉、管理。创建区不再重复模型选择；切换模型时重置时长和分辨率。
- 下拉末项是“添加视频模型”，选中后打开添加对话框；没有单独的添加按钮。
- “管理”打开模型列表对话框，可编辑和删除；对话框底部也能添加模型，所以空列表时仍可从“管理”进入。
- 添加对话框字段：显示名称、模型 ID、**视频协议**、来源（渠道 / 自定义地址）、可选时长、可选分辨率、是否支持参考图。
- 模型 ID 是自由文本，不做白名单校验；导入时也不过滤 `/v1/models` 的返回结果。
- 协议决定路径、请求体和状态映射；切换协议不会清空已填的时长/分辨率。三个协议共用同一套建议值，模型自己保存可编辑的列表。
- 来源选“渠道”时只列已启用的渠道。手工选择渠道后使用该渠道第一个启用令牌；需要指定令牌时用“从渠道导入”。
- 来源选“自定义地址”时填 Base URL 和 API Key。API Key 不写进模型文件，单独存到 `~/.droidgear/playground/video_secrets.json`（0600）。
- 时长和分辨率用逗号分隔填写，作为创建表单里的下拉建议；创建表单同时允许手输，留空表示不传该参数。模型没保存自己的列表（老文件或列表被清空）时，下拉回退到下面那套共用建议值，不会变成不可点的灰按钮。
- 参考图开关只表示模型是否支持图生视频；默认开启，文本模型可以手动关掉。
- 删除模型不影响已有任务：任务保存了创建时的模型快照。

### 4.2 从渠道导入

三步：渠道 → 令牌 → 模型。

1. 只列已启用渠道。
2. API Key 类型的渠道（CLI Proxy API、Ollama、General、DeepSeek）直接读已保存的 Key；其他渠道列出令牌，选一个。
3. 用令牌请求渠道的 `/v1/models`，展示全部模型，支持搜索、全选和批量选择。
4. 顶部选择本次导入的默认协议（`sora` / `new-api` / `doubao`），每一行可以单独改成另一个协议；导入后仍可逐个编辑。导入不按模型 ID 推断协议，也不按协议改时长/分辨率建议值。

导入不复制渠道密钥：模型只保存 `channelId` 和 `tokenId`，调用时再从渠道存储解析。这样换令牌或改 Base URL 后不用重新导入。代价是每次调用可能需要登录渠道换令牌，Rust 侧对令牌做了 5 分钟内存缓存。

### 4.3 创建任务

- 模型和提示词必填；高级参数不是 JSON 对象时，提交按钮禁用并在输入框下方提示。
- `⌘/Ctrl+Enter` 提交。提交期间按钮显示“创建中…”，成功后表单不清空、不锁定，可以改参数继续提交，多个任务并行。
- 提交按钮左侧有清除按钮（扫帚图标，悬停显示“清空表单”），清空提示词、时长、分辨率、参考图和高级参数，但保留当前选中的模型。
- 创建失败不插入任务，只弹错误提示；错误信息来自上游（HTTP 状态 + 响应体里的 `error.message`）。
- 时长/分辨率是可输入的下拉组合：输入框自由填写，右侧按钮优先用该模型的建议列表，模型没有列表时用共用建议值；不再有单独的“默认”选项，留空即不传。
- 三个协议共用一套建议值（`2/3/4/5/6/8/10/12/15/20/30` 秒、`480p`/`720p`/`1080p`/`4K` 和常见 `WxH`）。上游实际限制不同，由上游校验：
  - New API Sora：`"4"`/`"8"`/`"12"` 秒；`720x1280`/`1280x720`/`1024x1792`/`1792x1024`。
  - Kling 4.0：3–30 秒。
  - Seedance：1.0 最长 12 秒，2.0 最长 15 秒，2.5 最长 30 秒，`-1` 表示自动。
  - Wan 2.x：2–30 秒；Wan 3.0 支持 `-1` 自动。
  - 因此没有统一的“3 秒以上”下限；Playground 只校验 `>0` 且 `<=3600`，避免把某个模型的限制强加给别的模型。
- 分辨率输入会按协议落到对应的线上字段：Sora 用 `size`，New API 原生用 `width`/`height`（并把档位写进 `metadata.resolution`），Doubao 用 `resolution` 档位；`WxH` 会映射到最近的 Ark 档位。

### 4.4 进度刷新

| 方式     | 行为                                                                |
| -------- | ------------------------------------------------------------------- |
| 自动刷新 | 默认开启，间隔可选 3 / 5 / 10 / 30 秒；没有进行中的任务时不发起轮询 |
| 全部刷新 | 任务列表右上角按钮，查询所有进行中的任务                            |
| 单个刷新 | 任务详情的“查询状态”，查询当前任务；已完成任务也可用来重试内容下载  |

列表头显示“更新于 hh:mm:ss”，刷新中显示加载图标并禁用刷新按钮，避免重复请求。任务进入终态后不再自动轮询。

任务记录在创建、状态变化、取消和删除时写入 `video_tasks.json`。重启后任务列表会恢复；进行中的任务继续轮询，已完成的任务直接显示本地视频或远程 URL。

#### 4.5 参考图

- 创建区是宫格：末尾“+”打开添加对话框，二选一：URL 或本地图片。
- URL 输入框支持一次粘贴多行，每行一个链接，校验通过后一次性加入宫格；本地图片支持一次多选，逐张读成 base64 data URL 后一次性加入。单张上限 20 MB。
- URL 直接作为图片地址渲染并发送；本地图片用 `FileReader.readAsDataURL` 转成 base64 data URL，再和 URL 一样发送。
- 已添加的图片以缩略图显示，右上角删除按钮可以逐张移除；添加后的宫格行为不变。
- `sora`/`new-api` 单图发 `image`，普通多图发 `images`；Doubao/Seedance 模型的普通多图改为 `metadata.content`，每张图带 `role: "reference_image"`。
- `doubao` 把每张图变成一个 `content` 的 `image_url` 条目；多图时自动加 `role: "reference_image"`，单图不加 role、按首帧处理。
- 多图张数上限由模型/上游决定，Playground 不做限制；Kling 这类只认首尾帧的模型可能不消费 `images` 数组。
- 只支持 URL/data URL，不支持 multipart 文件上传；本地图片也会转成 data URL 后走 JSON。

## 4.6 任务状态

| 状态   | 列表行            | 详情预览区              | 详情操作                       |
| ------ | ----------------- | ----------------------- | ------------------------------ |
| 排队中 | 时钟图标          | 加载动画 + “排队等待中” | 查询状态、取消、删除           |
| 生成中 | 旋转图标 + 进度条 | 加载动画 + 百分比进度条 | 查询状态、取消、删除           |
| 已完成 | 绿色对勾          | `<video controls>` 播放 | 查询状态、外部打开、重跑、删除 |
| 失败   | 红色叹号          | 错误信息                | 重跑、删除                     |
| 已取消 | 禁止图标          | “已取消”                | 重跑、删除                     |

- 视频加载失败（如链接过期）时，预览区显示“视频加载失败，链接可能已过期”。
- Sora 内容下载失败时任务仍是“已完成”，预览区显示下载错误；可以点“查询状态”重试。New API 原生直接使用查询响应里的 `url`，链接过期时预览区显示加载失败。
- “重新生成”用相同参数和模型快照新建任务；“载入参数”把该任务的模型和参数回填到创建表单。

## 5. 代码结构

```
src-tauri/crates/droidgear-core/src/playground.rs # 渠道凭据、令牌缓存、HTTP 与文件助手（两个页面共用）
src-tauri/crates/droidgear-core/src/video.rs      # 视频类型、持久化、三种协议客户端
src-tauri/crates/droidgear-core/src/image.rs      # 图片类型、持久化、三种协议客户端
src-tauri/src/commands/video.rs                   # 视频 Tauri 命令包装
src-tauri/src/commands/image.rs                   # 图片 Tauri 命令包装
src/store/playground-store.ts                     # 视频模型、任务、草稿、刷新设置
src/store/image-playground-store.ts               # 图片模型、任务、草稿、提交与持久化
src/components/playground/
├── PlaygroundFeatureList.tsx                     # 侧栏：视频 / 图片
├── shared/
│   ├── reference-images.tsx                      # 参考图宫格，URL/本地图片，两页共用
│   ├── model-toolbar.tsx                         # 页头模型下拉（含添加）与管理
│   ├── model-form-dialog.tsx                     # 添加/编辑模型的通用表单
│   ├── model-import-dialog.tsx                   # 渠道 → 令牌 → 模型批量导入
│   ├── model-manager-dialog.tsx                  # 模型列表管理
│   └── format.ts                                 # 时间格式化、列表解析、JSON 校验
├── video/
│   ├── VideoPlaygroundPage.tsx                   # 页面骨架与三栏布局
│   ├── VideoCreatePanel.tsx                      # 创建区
│   ├── VideoTaskList.tsx                         # 任务列表、筛选、自动与手动刷新
│   ├── VideoTaskDetail.tsx                       # 任务详情与 <video> 预览
│   ├── VideoTaskStatus.tsx                       # 状态图标、徽标、进度条
│   ├── VideoModelDialog.tsx                      # 视频模型表单（协议、时长、分辨率）
│   ├── VideoModelManagerDialog.tsx               # 视频模型列表
│   └── video-protocol.ts                         # 时长/分辨率建议值、协议标签、导入映射
└── image/
    ├── ImagePlaygroundPage.tsx                   # 页面骨架与三栏布局
    ├── ImageCreatePanel.tsx                      # 创建区
    ├── ImageTaskList.tsx                         # 任务列表与筛选
    ├── ImageTaskDetail.tsx                       # 图片预览、缩略图与参数回显
    ├── ImageTaskStatus.tsx                       # 状态图标与徽标
    ├── ImageModelDialog.tsx                      # 图片模型表单（协议、尺寸、质量）
    ├── ImageModelManagerDialog.tsx               # 图片模型列表
    └── image-protocol.ts                         # 尺寸/质量建议值、协议标签、导入映射
src/lib/playground-utils.ts                       # 参考图按行解析
```

- 视图接入：`ui-store.ts` 的 `NavigationView` 增加 `'playground'`（不加入 `ToolView`），并用 `playgroundSubView` 区分 `video` / `image`；`LeftSideBar.tsx` 加入口按钮、侧栏功能列表和进行中任务角标；`MainWindowContent.tsx` 按子视图 lazy 加载 `VideoPlaygroundPage` 或 `ImagePlaygroundPage`。
- 状态必须放在 store：页面是 lazy 加载的，切换视图会卸载组件。草稿、任务和刷新设置放在组件 `useState` 里会在切出后丢失。
- 自动刷新的定时器在 `VideoTaskList` 里，离开 Playground 页面后暂停，回到页面后恢复。

## 6. 持久化与媒体

| 内容           | 位置                                         | 说明                                          |
| -------------- | -------------------------------------------- | --------------------------------------------- |
| 视频模型       | `~/.droidgear/playground/video_models.json`  | 普通 JSON；自定义来源的 API Key 会被清空      |
| 自定义 API Key | `~/.droidgear/playground/video_secrets.json` | 0600，按模型 ID 存；读模型时合并回去          |
| 下载的视频     | `~/.droidgear/playground/media/{taskId}.mp4` | Sora 内容；同名文件存在时直接复用             |
| 任务           | `~/.droidgear/playground/video_tasks.json`   | 模型快照 + 参数 + 状态；自定义 API Key 不落盘 |
| 图片模型       | `~/.droidgear/playground/image_models.json`  | 普通 JSON；自定义来源的 API Key 会被清空      |
| 图片 API Key   | `~/.droidgear/playground/image_secrets.json` | 0600，按模型 ID 存；读模型时合并回去          |
| 生成的图片     | `~/.droidgear/playground/media/images/`      | `{taskId}-{n}.{ext}`；同名文件存在时直接复用  |
| 图片任务       | `~/.droidgear/playground/image_tasks.json`   | 模型快照 + 参数 + 图片路径/URL + 状态         |

- New API Sora 内容通过 asset 协议播放。`tauri.conf.json` 需要 `assetProtocol.enable = true`，scope 限定在 `$HOME/.droidgear/playground/media/**`，CSP 的 `media-src` 需要包含 `asset: http://asset.localhost`；生成的图片同样走 asset 协议，所以 `img-src` 也要包含它。
- 模型文件里旧的 `protocol: "new-api-videos"` 读取时映射成 `sora`；下一次保存自动改写。
- 下载先写 `.part` 临时文件，完成后 rename；失败会清理临时文件。
- 媒体目录不会自动清理。后续可以按任务删除或按 TTL 清理。

## 7. 视频页已知限制

- 进行中任务的进度不会每次轮询都落盘；重启后以新的查询结果为准。
- 删除自定义模型后，其历史任务在重启后不能再查询（自定义 API Key 不会复制到任务文件）；已经下载的视频仍可播放。
- 取消是本地状态，不会通知上游。
- 只支持 JSON 请求，不支持 multipart 上传参考图；参考图可以填 URL，也可以从本地选择，本地文件会转成 base64 data URL 后随 JSON 发送。
- `sora`（New API Sora）协议发送 New API 通用 `/v1/videos` 字段（`seconds`/`size`/`image`/`images`），不再限制一张参考图。各厂商插件对多图的消费能力不同：即梦/Vidu/阿里/豆包读 `images`，Kling 只认 `image`/`image_tail`，Sora 插件本身只认单张 `input_reference`（Sora 已关停）。doubao 模型仍建议选择 `doubao` 协议。
- New API 原生格式的模型请选择 `new-api`；`sora` 和 `new-api` 现在都用 `image`/`images`，区别是路径和 `seconds`/`size` 与 `duration`/`width`/`height` 的字段形态。
- Doubao/Seedance 的多图角色映射按模型 ID 里的 `doubao`/`seedance` 识别；如果模型用了不含这两个词的别名，请改用 `doubao` 协议，或在高级参数里自行提供 `metadata.content` 和 `role`。
- New API 原生不下载内容，直接使用查询响应里的 `url`；如果上游返回的地址需要额外鉴权或已过期，预览会失败，需要重新查询或让上游提供可访问的地址。
- Sora 内容必须先下载才能播放，首次完成时会多花一次下载时间。
- 没有做模型能力探测：模型是否支持参考图、哪些时长/分辨率合法，由用户配置。
- TUI 未实现；这是有意例外。

## 8. 图片页

图片页复用了视频页的导航、三栏布局和模型管理方式，区别在于生成是**单次同步请求**：提交后等待上游返回图片，没有轮询。

### 8.1 布局

```
┌──────────────────────────────────────────────────────────────────────┐
│ 图片生成                                [模型 ▾] [管理] │
├──────────────────┬───────────────────┬───────────────────┤
│ 提示词            │ [全部|生成中|已完成|失败] │ [已完成] img_xxxx ⧉  [打开][在文件夹中显示][重跑][🗑] │
│ […………………]        ├───────────────────┤                   │
│ 尺寸/比例 [..]    │ ⟳ 一只橘猫…        │   <img object-contain> │
│ 质量 [..]         │   Nano Banana ·    │   [缩略图][缩略图]   │
│ 数量 [1]          │   17:03:21 · 2 张  ├───────────────────┤
│ 参考图 […]        │ ✓ 微距镜头下的…    │ 提示词全文  [载入参数] │
│ ▸ 高级参数 (JSON) │                   │ 模型 / 协议 / 尺寸 / │
│                  │                   │ 质量 / 数量 / 时间… │
├──────────────────┤                   │                   │
│ 可能产生费用 [生成图片] │                │                   │
└──────────────────┴───────────────────┴───────────────────┘
     创建区 32%           任务列表             任务详情（三栏均可拖拽）
```

| 区域     | 内容与规则                                                                   |
| -------- | ---------------------------------------------------------------------------- |
| 页头     | 标题；右上角模型下拉（末项“添加图片模型”）和管理                             |
| 创建区   | 提示词、尺寸/比例、质量、数量（仅支持多图的协议显示）、参考图、高级参数 JSON |
| 任务列表 | 全部 / 生成中 / 已完成 / 失败；不轮询，图片返回后任务直接变为终态            |
| 任务详情 | 状态、任务 ID、操作按钮、图片预览（多图有缩略图）、参数与模型回复回显        |

### 8.2 三种协议

| 协议                 | 创建                                                | 结果位置                                                                   |
| -------------------- | --------------------------------------------------- | -------------------------------------------------------------------------- |
| `chat`               | `POST {base}/v1/chat/completions`                   | `choices[].message.images[].image_url.url` 或 `content` 里的 markdown 图片 |
| `responses`          | `POST {base}/v1/responses`                          | `output[].result`（`type = image_generation_call`）                        |
| `google`             | `POST {base}/v1beta/models/{model}:generateContent` | `candidates[].content.parts[].inlineData.data`                             |
| `google` + `imagen*` | `POST {base}/v1beta/models/{model}:predict`         | `predictions[].bytesBase64Encoded`                                         |

- Base URL 归一化：Chat/Responses 会自动补 `/v1`，也接受直接粘贴的 `/v1/chat/completions` 或 `/v1/responses`；Google 会补 `/v1beta`，也接受已带 `/v1beta` 的地址。Google 的 OpenAI 兼容地址（`/v1beta/openai`）按 ChatCompletions 协议使用，Base URL 以 `/openai` 结尾时不会补 `/v1`。
- Google 鉴权同时发送 `x-goog-api-key` 与 `Authorization: Bearer`：Gemini 原生端点读前者，New API 兼容网关通常读后者，二者兼发对上游无影响。
- `imagen*` 模型按模型 ID 前缀识别，和 new-api 的判断一致。

### 8.3 请求映射

创建表单的字段在三种协议上落到不同位置：

| 表单字段 | `chat`                                                | `responses`                            | `google`                                                       |
| -------- | ----------------------------------------------------- | -------------------------------------- | -------------------------------------------------------------- |
| prompt   | `messages[0].content[0].text`                         | `input[0].content[0].text`             | `contents[0].parts[0].text`                                    |
| 参考图   | `messages[0].content[]` 的 `image_url`                | `input[0].content[]` 的 `input_image`  | `parts[].inlineData`（URL 先下载并转 base64）                  |
| size     | `WxH` 发 `size`；`A:B` 发 `image_config.aspect_ratio` | `tools[0].size`                        | `generationConfig.imageConfig.aspectRatio`（`WxH` 会约成比例） |
| quality  | 不发送，留给高级参数                                  | `tools[0].quality`                     | `generationConfig.imageConfig.imageSize`                       |
| count    | `n`（>1 时）                                          | 只有一张，不发送                       | `imagen` 用 `parameters.sampleCount`                           |
| 高级参数 | 合并进请求体                                          | 合并进请求体；`tools` 会与表单工具合并 | 合并进请求体；`generationConfig` 只补缺失键                    |

- Chat 请求固定带 `modalities: ["image", "text"]`：OpenRouter 一类网关靠它开启图片输出；New API 的 Gemini 渠道会按自己的模型配置补 `responseModalities`，多余字段会被忽略。
- Responses 的图片生成工具总会存在；高级参数里自己写了 `tools` 时保留用户的工具，再向其中合并尺寸/质量。
- Google 的 `size` 支持组合写法，例如 `16:9 2K`：比例进 `aspectRatio`，`1K`/`2K`/`4K` 进 `imageSize`。
- `imagen*` 不支持参考图，带上参考图会直接报错，提示改用 Gemini 图片模型。

### 8.4 结果解析

- **Chat**：先读 `message.images`（OpenRouter 的 `image_url` 结构），再扫 `message.content` 里的 `![alt](url)`；`data:` URL 直接解码，http(s) URL 尝试下载，失败时保留 URL 供界面直接渲染。部分网关会用 Images API 的 `data[].b64_json` 形状回复 chat，也一并兼容。
- **Responses**：收集所有 `image_generation_call.result`；`revised_prompt` 会保存并展示。`status = failed` 的 item 直接返回错误。
- **Google**：收集 `inlineData`/`inline_data`，以及 `:predict` 的 `predictions[].bytesBase64Encoded`；`promptFeedback.blockReason`、`finishReason`（`SAFETY` 等）和 `raiFilteredReason` 会转成可读错误。
- 没有图片但有文字时，错误信息会带上模型回复的摘要；完全空响应返回 “The upstream returned no images.”。

### 8.5 图片落盘与显示

- 解码后的图片写到 `~/.droidgear/playground/media/images/{taskId}-{n}.{ext}`，任务记录只保存路径和 MIME，不保存 base64。
- 远程 URL 会先下载再保存；下载失败时记录保留 URL，界面直接渲染远程图片。
- 任务 ID 由前端生成（`img_` + 随机串）并传给 Rust，Rust 校验它只含字母数字、`-`、`_`、`.`，避免路径穿越。
- 预览用 `convertFileSrc` + asset 协议；`tauri.conf.json` 的 `assetProtocol.scope` 已覆盖整个 `media/**`，CSP 的 `img-src` 增加 `asset: http://asset.localhost`。
- 多图时详情页底部有缩略图切换，单图时只有预览。

### 8.6 任务状态与交互

| 状态   | 列表行   | 详情预览区                 | 操作                             |
| ------ | -------- | -------------------------- | -------------------------------- |
| 生成中 | 旋转图标 | 加载动画 + “正在生成图片…” | 删除                             |
| 已完成 | 绿色对勾 | 图片预览 + 缩略图切换      | 打开、在文件夹中显示、重跑、删除 |
| 失败   | 红色叹号 | 错误信息                   | 重跑、删除                       |

- 提交后任务立即以“生成中”插入列表并落盘；命令返回后改成已完成/失败并再次落盘。
- 没有“取消”按钮：生成是一个已经发出的 HTTP 请求，本地取消既停不下上游计费，也拿不回结果。等待期间按钮显示“生成中…”并禁用，避免重复提交；请求返回时若任务已被删除，结果会被丢弃。
- 没有轮询：生成是一个请求，等待期间 UI 用 `isCreating` 显示“生成中…”，按钮同时禁用，避免重复提交。
- 重启时仍为“生成中”的记录会被标记为失败，并提示“应用在图片生成完成前退出”（单次请求无法在重启后继续等待）。
- “重新生成”用相同参数和模型快照新建任务；“载入参数”把任务参数回填创建表单。

### 8.7 图片模型

- 字段：显示名称、模型 ID、**图片协议**、来源（渠道 / 自定义地址）、尺寸建议、质量建议、是否支持参考图。
- 尺寸/质量只是创建表单下拉里的建议值，可手输；默认值随协议变化（Chat：常见 `WxH` 与比例；Responses：`auto`/OpenAI 图片尺寸与 `low`/`medium`/`high`；Google：比例与 `1K`/`2K`/`4K`）。新增模型时切换协议会同步刷新这两组默认值。
- 参考图开关决定创建区是否显示参考图宫格；模型不支持时 Rust 也会拒绝带参考图的请求。
- 数量输入只在支持多图的协议显示：`chat` 总是显示；`google` 仅 `imagen*` 显示；`responses` 固定一张。
- 渠道导入、令牌解析、密钥保存和视频页一致：模型只存 `channelId` 与 `tokenId`，调用时再从渠道存储解析。

### 8.8 持久化

见第 6 节表格中的 `image_*` 行。图片页的 secrets 与视频页分开（`image_secrets.json`），删除模型时只会清理自己的密钥文件；已经生成到磁盘的图片不会随任务删除而清理。

### 8.9 已知限制

- Responses 与 Gemini 图片模型（`imagen*` 除外）一次只生成一张图，数量字段对它们不显示。
- ChatCompletions 没有统一的尺寸/质量参数名；Playground 只按 `WxH` → `size`、`A:B` → `image_config` 发送，其他写法需要放进高级参数。
- Google 的参考图必须是可访问的 http(s) URL 或 data URL；本地文件在创建区已经转成 data URL，不受影响。URL 下载失败会中断本次生成。
- 删除任务不会删除 `media/images/` 下的文件；媒体目录暂不自动清理。
- 任务记录保存模型快照，但自定义端点的 API Key 不复制进任务文件；删除自定义模型后，历史任务不能再重跑（已生成的图片仍可查看）。
- TUI 未实现图片页，与视频页一样是有意例外。
