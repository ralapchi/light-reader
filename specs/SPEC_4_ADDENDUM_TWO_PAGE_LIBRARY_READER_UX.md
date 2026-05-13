# 第四期补充：双页面书架 / 阅读模型与打开过渡页设计稿

## 1. 文档目的

本补充稿用于把当前应用的主体验调整为更接近成熟桌面阅读器的双页面模型：

1. **书架页（Library / Home）**
2. **阅读页（Reader）**

在这两个正式页面之间，增加一个**短暂的打开过渡页（Loading / Opening Book）**：

- 显示当前书籍封面
- 显示书名 / 作者
- 显示打开进度图标或轻量动画
- 完成解析后再切换到阅读页

本稿同时定义：

- UI 结构调整清单
- 行为边界与约束
- 视觉层级建议
- 使用 skill 的要求
- 团队启动口令

---

## 2. 设计目标

本次调整不是做“再多一个状态页”，而是明确应用的**主页面模型**。

目标如下：

1. 应用长期只有两个正式页面：
   - 书架页
   - 阅读页
2. 打开书籍时，必须经过统一的“打开过渡态”
3. 阅读页保持沉浸，不默认暴露过多控制 UI
4. 目录与设置改为浮层，不再成为阅读页常驻结构
5. 书架页与阅读页的视觉语言要明显区分：
   - 书架页更偏浏览与管理
   - 阅读页更偏沉浸与消费

---

## 3. 页面模型

### 3.1 正式页面

#### Page A：书架页

用途：

- 浏览本地书库
- 查看封面
- 继续阅读
- 搜索 / 排序 / 筛选
- 进入书籍打开流程

#### Page B：阅读页

用途：

- 阅读正文
- 进行搜索、翻章、听书、书签、设置

### 3.2 过渡状态页

#### Transitional State：打开书籍页

用途：

- 作为从书架进入阅读页的统一过渡
- 向用户明确表达“当前正在打开哪一本书”
- 用封面和进度动画减少跳转突兀感

此状态不是第三个长期页面，而是：

- 一个短暂的全屏状态
- 成功后进入阅读页
- 失败后进入错误态

---

## 4. 目标交互流

```text
启动应用
  -> 进入书架页
  -> 用户点击某本书
  -> 进入打开书籍过渡页
  -> 解析成功
  -> 进入阅读页
  -> 用户返回书架
  -> 回到书架页
```

失败流：

```text
书架页
  -> 点击书籍
  -> 打开书籍过渡页
  -> 解析失败 / 文件缺失
  -> 错误态
  -> 用户返回书架或重新选择文件
```

---

## 5. 书架页设计清单

### 5.1 信息结构

书架页建议固定为三层：

1. 顶部主标题区
   - 应用名
   - 搜索入口
   - 导入入口
2. 继续阅读区
   - 大卡片
   - 显示当前阅读进度
3. 全部书籍区
   - 更规整的封面网格

### 5.2 卡片设计

每张书籍卡片建议包含：

- 真实封面
- 书名
- 作者
- 进度文字
- 可选的小型进度条

要求：

1. 封面比例统一，禁止高度随内容抖动
2. 进度文字明显小于书名和作者
3. 卡片底部信息行高统一
4. 书名最多两行，超出省略
5. 作者和进度都使用次级文本色

### 5.3 交互建议

书架页主路径应从“查看详情”改为“直接进入阅读流程”。

推荐策略：

1. 单击卡片：直接进入“打开书籍过渡页”
2. 详情查看：放到单独的 `...` 按钮、右键菜单或信息按钮
3. “继续阅读”区卡片优先直接打开

不建议：

- 继续让“单击只开详情、双击才打开”作为主路径

原因：

- 用户要的是读书，不是先处理书籍详情
- 双页面模型下，书架页的核心任务应是“进入书”

### 5.4 排版要求

1. 继续阅读区卡片尺寸大于普通书架卡片
2. 普通书架卡片网格必须稳定，不因文字长短改变列宽
3. 卡片间距要统一
4. 进度信息的视觉重量应降低
5. 缺失封面时使用高质量占位封面，而不是空白块

---

## 6. 打开书籍过渡页设计清单

### 6.1 结构

打开书籍时进入一个中心聚焦的过渡页，结构如下：

1. 中央封面
2. 书名
3. 作者
4. 加载状态图标 / 旋转进度图标
5. 一行轻量状态文案

### 6.2 行为要求

1. 点击书籍后，不要瞬间切正文
2. 至少进入一次可感知的过渡态
3. 如果本地解析极快，也应保留一个极短的最小时长，避免闪屏
4. 若打开失败，应从该页自然转为错误态

### 6.3 视觉要求

1. 封面是视觉中心
2. 背景应简洁，避免复杂装饰
3. 加载图标应轻，不抢封面
4. 书名与作者应垂直对齐在封面下方
5. 若封面可用，严禁 loading 页退回占位块

### 6.4 动画要求

建议轻量即可：

- 封面淡入
- 加载图标旋转
- 完成后整体淡出进入阅读页

不做：

- 复杂 3D 翻页
- 大面积粒子或重动画

---

## 7. 阅读页设计清单

### 7.1 基本原则

阅读页只服务一件事：

**读正文**

因此要求：

1. 默认情况下只显示正文
2. 工具栏不常驻
3. 目录不常驻
4. 侧栏不常驻

### 7.2 顶部操作栏

行为要求：

1. 鼠标移动到顶部热区时显示
2. 鼠标离开后自动隐藏
3. 当搜索、设置、目录浮层打开时，可保持可见

推荐按钮：

- 返回书架
- 目录
- 搜索
- 书签
- 听书
- 设置
- 上一章 / 下一章

要求：

1. 优先使用图标或短文案
2. 高度克制，不抢正文
3. 背景半透明
4. 不推动正文布局

### 7.3 目录行为

目录默认不显示。

触发方式：

- 点击顶部工具栏“目录”按钮

显示方式：

- 左上角浮层或独立悬浮窗

关闭方式：

- 点击目录按钮再次关闭
- 点击浮层外部关闭
- 切换章节后自动关闭

要求：

1. 目录不再作为阅读页固定侧栏
2. 当前章节有明显高亮
3. 章节列表支持滚动

### 7.4 正文区

要求：

1. 正文始终是视觉中心
2. 左右留白稳定
3. 正文字体与行距保持阅读友好
4. 图片按正文真实顺序插入
5. 图片宽度自适应正文宽度，不超出正文列

### 7.5 状态栏

建议：

- 默认弱化显示
- 仅在用户开启时保留

不建议：

- 让状态栏在沉浸阅读中与正文争抢注意力

---

## 8. Skill 使用要求

本次 UI 调整设计**必须使用相关 skill**。

当前建议使用：

- `imagegen` skill

用途：

1. 先生成 3 张 UI mockup，用于确认页面结构
2. 再进入具体 Rust / egui 实现

### 必做的 3 张 mockup

1. 书架页 mockup
2. 打开书籍过渡页 mockup
3. 阅读页 mockup

### 使用方式

本设计稿已按 `imagegen` skill 中的 `ui-mockup` 结构整理，可直接复用为视觉生成 prompt。

### Mockup Prompt 1：书架页

```text
Use case: ui-mockup
Asset type: desktop app library home
Primary request: a macOS Books-inspired bookshelf page for a desktop novel reader
Scene/backdrop: full application window, calm neutral background, no marketing hero
Subject: bookshelf home page with continue reading row and a clean grid of book covers
Style/medium: polished product UI mockup
Composition/framing: straight-on desktop app window, library sidebar on the left, main content on the right
Lighting/mood: quiet, refined, work-focused, premium reading app
Color palette: soft neutral tones with restrained accent color
Constraints: real cover-first cards, small progress text, stable aligned grid, no oversized decorative cards, no landing page layout
Avoid: mobile layout, marketing hero, giant buttons, card-within-card composition
```

### Mockup Prompt 2：打开书籍过渡页

```text
Use case: ui-mockup
Asset type: desktop app loading transition
Primary request: a centered book opening transition screen between library and reader views
Scene/backdrop: full app window with calm background
Subject: large book cover in the center, title, author, and a subtle loading spinner below
Style/medium: polished desktop UI mockup
Composition/framing: centered composition with cover as the visual anchor
Lighting/mood: calm, focused, premium reading experience
Color palette: restrained neutral palette
Constraints: no busy UI chrome, no large status panels, no extra sidebars, cover must be the focal point
Avoid: splash screen style branding, empty placeholder art, game-like loading visuals
```

### Mockup Prompt 3：阅读页

```text
Use case: ui-mockup
Asset type: desktop app reader page
Primary request: a minimal immersive reader page for a desktop novel reader
Scene/backdrop: full app window, reading mode
Subject: centered text column, hidden chrome, hover-reveal top toolbar, floating table of contents panel
Style/medium: premium reading UI mockup
Composition/framing:正文 column centered, top toolbar lightly overlaid, no fixed sidebar
Lighting/mood: quiet, immersive, refined
Color palette: subtle neutral reading tones
Constraints: body text must be the visual focus, top bar not always visible, floating toc only, no permanent left sidebar
Avoid: dashboard layout, heavy controls, noisy backgrounds, oversized toolbar
```

---

## 9. 代码层实施边界

本次设计调整只允许在现有 `Rust + egui` 主路径内推进。

### 允许做

1. 调整 `AppShell` 页面切换逻辑
2. 强化 `LoadingBook` 状态
3. 调整 `library_page` 的书卡交互方式
4. 调整 `reader_layout` 的 hover toolbar 和 floating toc
5. 收口状态字段与 UI props

### 不允许做

1. 不切换 UI 框架
2. 不把阅读页重新做回固定左右分栏
3. 不新增第三条长期 UI 主链
4. 不把目录重新做成默认常驻
5. 不把 loading 页做成仅一行文字提示

---

## 10. 任务清单

### P1：页面模型收口

1. 明确书架页是主首页
2. 明确阅读页是唯一阅读主页面
3. `LoadingBook` 作为正式过渡态收口
4. 移除“详情弹层作为主路径”的交互优先级

### P1：书架页优化

1. 统一封面卡片比例
2. 缩小进度文字
3. 统一网格对齐
4. 单击卡片直接进入打开流程
5. 将详情能力移到次级入口

### P1：打开过渡页优化

1. 修复封面加载链路
2. 增加加载图标与状态文案
3. 增加最短展示时长，避免闪屏
4. 打开失败时自然转错误态

### P1：阅读页沉浸化

1. 顶部工具栏 hover reveal
2. 默认隐藏目录
3. 目录改为浮层
4. 正文保持唯一视觉中心

### P2：视觉收口

1. 统一书架页 / 过渡页 / 阅读页的主题细节
2. 调整按钮密度、文字层级、留白
3. 为三张 mockup 和最终实现做一致性回看

---

## 11. 验收标准

完成本次调整后，应满足以下条件：

1. 应用长期只保留两个正式页面：
   - 书架页
   - 阅读页
2. 点击书籍后，先进入封面 loading 过渡页，再进入阅读页
3. 书架页卡片网格更整齐，进度文字明显更小
4. 阅读页默认只突出正文
5. 顶部工具栏只在 hover 或浮层开启时显示
6. 目录默认隐藏，通过按钮悬浮打开
7. UI 调整过程已先通过 skill 生成 mockup 再实施

---

## 12. 推荐团队执行方式

### 架构师

负责：

1. 拆分页面模型调整任务
2. 明确 loading 页状态机
3. 明确详情弹层降级策略
4. 明确 reader 页 hover / toc / toolbar 边界

### 开发 A

负责：

1. 页面状态切换
2. `LoadingBook` 数据准备
3. 事件流与状态字段收口

### 开发 B

负责：

1. 书架页卡片布局
2. 过渡页 UI
3. 阅读页工具栏与目录浮层

### Reviewer

重点检查：

1. 是否真的只剩两个正式页面
2. loading 页是否真实存在且封面可见
3. 目录是否默认隐藏
4. 阅读页是否真的以正文为唯一中心
5. 是否先用 skill 生成 mockup 再编码

