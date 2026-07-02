# 阅读统计功能规格文档

> Light Reader — 桌面电子书阅读器 (Tauri 2 + React 19 + TypeScript + Zustand)

## 1. 概述

为 Light Reader 添加阅读统计页面，展示用户的阅读数据、习惯和成就。采用**追加式会话日志 + 本地聚合缓存**的双层数据模型，为未来联网同步做好准备。

### 1.1 设计参考

- 设计预览：`frontend/src/pages/statistic-page-v2.html`
- 设计风格：暖色调暗色主题，复用 `global.css` CSS 变量

### 1.2 核心原则

| 原则 | 说明 |
|------|------|
| 追加式会话日志 | 每次阅读产生一条不可变的 `ReadingSession` 记录，作为唯一数据源 |
| 两层数据模型 | `ReadingSession`（可同步）+ `ReadingAggregates`（本地缓存，可随时重算） |
| 用户标签系统 | 用户为书籍手动指定标签，标签云按本周各标签阅读时长加权展示 |
| 未来同步就绪 | UUID 会话 ID、RFC3339 时间戳、会话文件可独立同步无冲突 |

---

## 2. 数据模型

### 2.1 ReadingSession（会话日志）

每次阅读产生一条不可变记录。存储路径：`sessions/{uuid}.json`

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReadingSession {
    pub session_id: String,       // UUID v4
    pub book_id: String,
    pub started_at: String,       // RFC3339 UTC
    pub ended_at: String,         // RFC3339 UTC
    pub active_seconds: u64,      // 扣除空闲后的实际阅读秒数
    pub chapter_start: u32,
    pub chapter_end: u32,
    pub nav_events: u32,          // 翻页/滚动次数
}
```

### 2.2 ReadingAggregates（聚合缓存）

从会话日志派生的本地缓存，可随时从 sessions 重算。存储路径：`stats_cache.json`

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReadingAggregates {
    pub total_active_seconds: u64,
    pub daily_seconds: HashMap<String, u64>,      // "2026-06-18" → 秒数
    pub per_book_seconds: HashMap<String, u64>,    // book_id → 秒数
    pub hourly_seconds: HashMap<u8, u64>,          // 0-23 → 秒数
    pub active_dates: Vec<String>,                 // 有阅读记录的日期列表
    pub books_completed: u32,
    pub total_nav_events: u64,
    pub computed_at: String,                       // RFC3339
}
```

### 2.3 ReadingPersonality（阅读人格）

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReadingPersonality {
    pub id: String,           // "night_owl" | "early_bird" | "fragmented" | "marathon" | "steady"
    pub name: String,         // "深夜沉浸者"
    pub description: String,  // "62% 的阅读发生在 18 点之后..."
    pub badges: Vec<String>,  // ["夜猫子", "沉浸阅读", "7 天全勤"]
}
```

**判定算法**（`compute_personality`）：

| 条件 | 人格 |
|------|------|
| >=40% 在 22:00-04:00 | 夜猫子 (night_owl) |
| >=40% 在 05:00-11:00 | 晨读鸟 (early_bird) |
| 平均单次 < 10min | 碎片读者 (fragmented) |
| 平均单次 > 30min | 马拉松读者 (marathon) |
| 其他 | 稳读派 (steady) |

**徽章生成规则**：

| 徽章 | 条件 |
|------|------|
| "7 天全勤" | current_streak >= 7 |
| "夜猫子" | 夜间阅读占比 >= 40% |
| "沉浸阅读" | avg_session >= 30min |
| "长篇偏好" | avg_chapters_per_book >= 20 |

### 2.4 Milestone（里程碑）

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Milestone {
    pub milestone_type: MilestoneType,
    pub threshold: u64,
    pub label: String,           // "1 小时" / "5 本"
    pub achieved: bool,
    pub achieved_at: Option<String>,  // RFC3339, 首次达到的时间
    pub current_value: u64,      // 当前值（用于进度展示）
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MilestoneType {
    Time,
    Book,
}
```

**预定义里程碑**（两维度交替）：

```
时间: 1h → 10h → 50h → 100h → 500h
书籍: 1本 → 5本 → 10本 → 50本
```

### 2.5 BookTag（书籍标签）

用户为书籍手动指定的标签。存储路径：`book_tags.json`

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BookTag {
    pub book_id: String,
    pub tags: Vec<String>,
}
```

文件格式：JSON 数组
```json
[
  { "book_id": "abc123", "tags": ["古典文学", "小说"] },
  { "book_id": "def456", "tags": ["科幻", "小说"] }
]
```

### 2.6 TagCloudEntry（标签云条目）

按标签聚合的本周阅读时长。

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TagCloudEntry {
    pub tag: String,
    pub seconds: u64,
}
```

### 2.7 等级系统

```rust
// 计算公式
level = floor(total_active_seconds / 36000) + 1   // 每 10 小时升一级
level_progress = (total_active_seconds % 36000) / 36000.0
today_xp = daily_seconds[today]
daily_target = 7200  // 2 小时
```

### 2.8 连续天数

从今天倒推 `active_dates`，计算连续有阅读记录的天数。

---

## 3. 存储层

### 3.1 文件结构

```
{app_data_dir}/
  sessions/              ← 新增
    {uuid}.json          ← 每次阅读会话一个文件
  stats_cache.json       ← 新增，聚合缓存
  book_tags.json         ← 新增，书籍标签映射
  progress/              ← 已有
  bookmarks/             ← 已有
  library_index.json     ← 已有
  cache/                 ← 已有
```

### 3.2 路径函数（`src/storage/paths.rs`）

新增函数：

| 函数 | 返回 |
|------|------|
| `sessions_dir()` | `{app_data_dir}/sessions/` |
| `session_path(session_id)` | `sessions/{session_id}.json` |
| `stats_cache_path()` | `stats_cache.json` |
| `book_tags_path()` | `book_tags.json` |

`ensure_dirs()` 新增创建 `sessions/` 目录。

### 3.3 存储模块

| 模块 | 职责 |
|------|------|
| `session_store.rs` | `save_session`, `load_all_sessions`, `load_sessions_since(date)`, `load_sessions_for_book(book_id)` |
| `stats_store.rs` | `load_aggregates`, `save_aggregates` |
| `tag_store.rs` | `load_all_tags`, `save_tags_for_book(book_id, tags)`, `delete_tags_for_book(book_id)` |

所有写入使用已有的 `write_json_atomic` 工具函数。

---

## 4. Tauri 命令 API

### 4.1 新增状态类型（`src/tauri_api/commands/mod.rs`）

```rust
pub type ActiveSessionState = Mutex<Option<ActiveSession>>;
pub type AggregatesState    = Mutex<Option<ReadingAggregates>>;

pub struct ActiveSession {
    pub session_id: String,
    pub book_id: String,
    pub started_at: String,        // RFC3339
    pub chapter_start: u32,
    pub active_seconds: u64,
    pub nav_events: u32,
}
```

### 4.2 命令列表（`src/tauri_api/commands/stats.rs`）

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `stats_start_session` | `book_id: String, chapter_index: u32` | `String` (session_id) | 创建会话，写入 ActiveSessionState |
| `stats_end_session` | — | `Result<(), String>` | 结束会话，写入文件，增量更新聚合 |
| `stats_pulse` | `active_seconds_delta: u64, nav_events_delta: u32` | `Result<(), String>` | 前端每 30s 心跳 |
| `stats_get_aggregates` | — | `ReadingAggregatesDto` | 返回聚合数据 |
| `stats_get_weekly_trend` | — | `Vec<DailyTrendDto>` | 最近 7 天每日时长 |
| `stats_get_top_books` | — | `Vec<TopBookDto>` | 本周阅读时长 top N 书籍 |
| `stats_get_milestones` | — | `Vec<MilestoneDto>` | 里程碑列表 |
| `stats_get_tag_cloud` | — | `Vec<TagCloudEntryDto>` | 本周各标签阅读时长 |
| `library_get_tags` | `book_id: String` | `Vec<String>` | 获取某本书的标签 |
| `library_set_tags` | `book_id: String, tags: Vec<String>` | `Result<(), String>` | 设置某本书的标签 |

### 4.3 DTO 定义（`src/tauri_api/dto.rs`）

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReadingAggregatesDto {
    pub total_active_seconds: u64,
    pub current_streak: u32,
    pub level: u32,
    pub level_progress: f64,          // 0.0-1.0
    pub today_seconds: u64,
    pub daily_target: u64,            // 7200
    pub this_week_seconds: u64,
    pub daily_avg_seconds: u64,
    pub daily_avg_nav_events: u32,
    pub personality: ReadingPersonalityDto,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReadingPersonalityDto {
    pub id: String,
    pub name: String,
    pub description: String,
    pub badges: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DailyTrendDto {
    pub date: String,       // "2026-06-18"
    pub label: String,      // "今天" / "周二" / "周六"
    pub seconds: u64,
    pub is_today: bool,
    pub is_peak: bool,      // 是否为本周最大值
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TopBookDto {
    pub book_id: String,
    pub title: String,
    pub author: String,
    pub seconds: u64,
    pub chapter_progress: String,  // "读至第 68 章"
    pub cover_color: String,       // 确定性哈希生成的颜色
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MilestoneDto {
    pub milestone_type: String,    // "time" | "book"
    pub threshold: u64,
    pub label: String,
    pub achieved: bool,
    pub achieved_at: Option<String>,
    pub current_value: u64,
    pub status: String,            // "done" | "active" | "pending"
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TagCloudEntryDto {
    pub tag: String,
    pub seconds: u64,
}
```

### 4.4 会话生命周期

```
stats_start_session(book_id, chapter)
  → 创建 ActiveSession { session_id: UUID, started_at: now, ... }
  → 写入 ActiveSessionState

stats_pulse(active_delta, nav_delta)   [每 30s]
  → 累加到 ActiveSessionState

stats_end_session()                    [退出阅读 / 关闭窗口]
  → 从 ActiveSessionState 取出会话
  → 计算 ended_at = now
  → 写入 sessions/{uuid}.json
  → 增量更新 AggregatesState
  → 写入 stats_cache.json

reader_save_progress()                 [已有，需修改]
  → 同时累加 session_read_seconds 和 total_read_seconds
```

### 4.5 窗口关闭处理（`src/main.rs`）

在 `CloseRequested` 事件中：
1. 检查 `ActiveSessionState`，如果有活跃会话则 flush 结束
2. 将 `AggregatesState` 写入 `stats_cache.json`

---

## 5. 前端 API 层

### 5.1 类型定义（`frontend/src/services/api.ts`）

新增接口，镜像 Rust DTO：

```typescript
interface ReadingAggregatesDto { ... }
interface ReadingPersonalityDto { ... }
interface DailyTrendDto { ... }
interface TopBookDto { ... }
interface MilestoneDto { ... }
interface TagCloudEntryDto { ... }
```

### 5.2 API 函数

```typescript
export function statsStartSession(bookId: string, chapterIndex: number): Promise<string>
export function statsEndSession(): Promise<void>
export function statsPulse(activeSecondsDelta: number, navEventsDelta: number): Promise<void>
export function statsGetAggregates(): Promise<ReadingAggregatesDto>
export function statsGetWeeklyTrend(): Promise<DailyTrendDto[]>
export function statsGetTopBooks(): Promise<TopBookDto[]>
export function statsGetMilestones(): Promise<MilestoneDto[]>
export function statsGetTagCloud(): Promise<TagCloudEntryDto[]>
export function libraryGetTags(bookId: string): Promise<string[]>
export function librarySetTags(bookId: string, tags: string[]): Promise<void>
```

命名遵循已有模式：`invoke('stats_start_session', { bookId, chapterIndex })`

---

## 6. 前端状态管理

### 6.1 Zustand Store 扩展（`frontend/src/store/useAppStore.ts`）

```typescript
interface StatisticsState {
  aggregates: ReadingAggregatesDto | null
  weeklyTrend: DailyTrendDto[]
  topBooks: TopBookDto[]
  milestones: MilestoneDto[]
  tagCloud: TagCloudEntryDto[]
  loaded: boolean
}

// 新增 action
setStatistics(partial: Partial<StatisticsState>): void
```

---

## 7. 前端会话追踪

### 7.1 useReadingSession Hook（`frontend/src/hooks/useReadingSession.ts`）

进入阅读页面时挂载，退出时卸载。

**行为**：
1. 挂载时调用 `statsStartSession(bookId, chapterIndex)`
2. 1 秒定时器累加 `activeSeconds`（空闲时暂停）
3. 30 秒定时器调用 `statsPulse(delta, delta)` 发送增量
4. 空闲检测：60 秒无 `mousemove`/`keydown`/`scroll` 暂停计时
5. 暴露 `countNavEvent()` 供翻页/滚动调用
6. 卸载时 flush + `statsEndSession()`

**暴露接口**：
```typescript
interface UseReadingSessionReturn {
  countNavEvent: () => void
}
```

### 7.2 ReaderPage 接入（`frontend/src/pages/ReaderPage.tsx`）

在 `ReaderPage` 组件中调用 `useReadingSession(bookId, currentChapterIndex)`，在翻页/滚动事件中调用 `countNavEvent()`。

---

## 8. 路由与导航

### 8.1 路由（`frontend/src/app/router.tsx`）

新增路由：`/statistics` → `<StatisticsPage />`

放在 MainLayout 内（与 `/`, `/bookmarks`, `/settings` 同级）。

### 8.2 侧边栏（`frontend/src/components/Sidebar.tsx`）

NAV_ITEMS 新增"统计"项：
```typescript
{ section: '书库', items: [
  { id: 'library', label: '全部', path: '/', icon: <BookSVG/> },
  { id: 'bookmarks', label: '书签', path: '/bookmarks', icon: <BookmarkSVG/> },
  { id: 'statistics', label: '统计', path: '/statistics', icon: <ChartSVG/> },  // 新增
]}
```

`pathToActive` 新增 `/statistics` → `'statistics'` 映射。

图标：折线图 SVG（Feather 风格，`<polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>`）。

---

## 9. 统计页面 UI

### 9.1 页面结构

```
StatisticsPage
├── Header（"阅读统计" + 本周日期范围）
├── Hero Row（grid: 200px 1fr）
│   ├── RingsCard（三环 SVG）
│   └── ReportCard（左文字 + 右标签云 + 右下4格）
├── PersonalityCard（深色卡片）
├── Trend + Books（grid: 1fr 280px）
│   ├── TrendCard（每日趋势条形图）
│   └── BooksCard（本周最爱书籍列表）
└── MilestoneCard（横向时间轴）
```

### 9.2 RingsCard

三环 SVG（`stroke-dasharray` / `stroke-dashoffset`）：
- **外环 (r0)**：当日时钟进度 — `(hour*3600 + min*60) / 86400`，颜色 `#d0c8be`，hover 显示时间
- **中环 (r2)**：升级进度 — `(total_seconds % 36000) / 36000`，颜色 `var(--gold)`
- **内环 (r1)**：今日经验 — `today_seconds / daily_target(2h)`，颜色 `var(--accent)`
- **中心**：`Lv.N`（仅等级数字）
- **图例**：升级进度百分比 + 今日经验分数

### 9.3 ReportCard

- **左侧**（240px）：本周报告文字（"本周你花了 X 小时，连续阅读 N 天，日均 Y 分钟"）
- **右侧上方**：标签云
  - 数据来源：`statsGetTagCloud()` → `TagCloudEntryDto[]`
  - 渲染算法：按 `seconds / max_seconds` 计算权重，映射到字号(11-26px)、字重(300-600)、颜色(gold/tertiary)、透明度(0.3-0.9)
  - 布局：绝对定位 + 碰撞检测（60 次尝试/标签），随机旋转(-10°~10°)和倾斜(-4°~4°)
- **右侧下方**：4 格统计（阅读时长、连续天数、日均阅读、日均翻阅）

### 9.4 PersonalityCard

- 深色背景（`var(--text-primary)`），金色装饰圆
- 左侧：`READING PERSONA` 眉毛文字 + 人格名称
- 分隔线
- 右侧：人格描述 + 徽章列表

### 9.5 TrendBooksSection

- **左侧 TrendCard**：每日趋势条形图
  - 最近 7 天倒序（今天在最上方），`space-around` 均匀分布
  - 条形宽度 = `day_seconds / max_day_seconds`
  - 最大值条形用金色高亮（`tb-peak`）
  - 今天标签用金色高亮
- **右侧 BooksCard**：本周最爱书籍列表（top 4）
  - 排名 + 书脊色条 + 书名/作者/进度 + 阅读时长
  - 书脊颜色由 `cover.ts` 确定性哈希生成

### 9.6 MilestoneTimeline

- 横向时间轴，两维度交替排列
- 圆点状态：
  - 已完成（时间）：金色 `var(--gold)` + 光晕
  - 已完成（书籍）：绿色 `#7a8a6a` + 光晕
  - 进行中：红色脉冲动画 `var(--accent)`
  - 未完成：灰色 `var(--border)`
- 标签下方显示完成日期或当前进度（"12.6 / 50"）

### 9.7 CSS（`StatisticsPage.css`）

- 复用 `global.css` 变量：`--bg`, `--surface`, `--text-primary`, `--text-secondary`, `--text-tertiary`, `--accent`, `--accent-soft`, `--border`
- 额外变量：`--gold: #b8915a`, `--gold-lt: #e8d4b0`, `--radius: 12px`
- 响应式：窄屏单列，宽屏双列
- 入场动画：`@keyframes up` 渐入上移，各区块延迟 50ms 递增

---

## 10. 完整文件清单

### 10.1 新建文件

| 文件 | 说明 |
|------|------|
| `src/domain/reading_session.rs` | ReadingSession 结构体 |
| `src/domain/reading_aggregates.rs` | ReadingAggregates, ReadingPersonality, Milestone, MilestoneType |
| `src/domain/book_tag.rs` | BookTag 结构体 |
| `src/storage/session_store.rs` | 会话存储 |
| `src/storage/stats_store.rs` | 聚合缓存存储 |
| `src/storage/tag_store.rs` | 标签存储 |
| `src/services/stats_service.rs` | 统计服务（聚合计算、人格判定、里程碑、标签云） |
| `src/tauri_api/commands/stats.rs` | 统计相关 Tauri 命令 |
| `frontend/src/hooks/useReadingSession.ts` | 阅读会话追踪 hook |
| `frontend/src/pages/StatisticsPage.tsx` | 统计页面主组件 |
| `frontend/src/pages/StatisticsPage.css` | 统计页面样式 |
| `frontend/src/pages/statistics/useStatisticsPage.ts` | 页面数据加载 hook |
| `frontend/src/pages/statistics/RingsCard.tsx` | 三环统计卡 |
| `frontend/src/pages/statistics/ReportCard.tsx` | 报告卡（含标签云） |
| `frontend/src/pages/statistics/PersonalityCard.tsx` | 阅读人格卡 |
| `frontend/src/pages/statistics/TrendBooksSection.tsx` | 趋势图 + 书籍列表 |
| `frontend/src/pages/statistics/MilestoneTimeline.tsx` | 里程碑时间轴 |

### 10.2 修改文件

| 文件 | 修改内容 |
|------|---------|
| `src/domain/mod.rs` | 注册 `reading_session`, `reading_aggregates`, `book_tag` 模块 |
| `src/storage/paths.rs` | 新增路径函数，`ensure_dirs()` 加 `sessions/` |
| `src/storage/mod.rs` | 注册 `session_store`, `stats_store`, `tag_store` 模块 |
| `src/services/mod.rs` | 注册 `stats_service` 模块 |
| `src/tauri_api/dto.rs` | 新增 6 个 DTO 类型 |
| `src/tauri_api/commands/mod.rs` | 新增 `ActiveSessionState`, `AggregatesState` 类型，注册 `stats` 模块 |
| `src/tauri_api/commands/reader.rs` | `reader_save_progress` 累加时间字段 |
| `src/main.rs` | 注册新 state 和 10 个新命令，窗口关闭时 flush |
| `frontend/src/services/api.ts` | 新增类型和 API 函数 |
| `frontend/src/store/useAppStore.ts` | 新增 `StatisticsState` 和 `setStatistics` |
| `frontend/src/app/router.tsx` | 新增 `/statistics` 路由 |
| `frontend/src/components/Sidebar.tsx` | 新增"统计"导航项 |
| `frontend/src/pages/ReaderPage.tsx` | 接入 `useReadingSession` |

---

## 11. 未来同步准备

- `ReadingSession` 每条记录有唯一 UUID，可独立同步、无冲突合并
- `ReadingAggregates` 纯本地缓存，同步后从合并的 sessions 重算
- 所有时间用 RFC3339 UTC，前端转本地显示
- 未来可加 `device_id` 字段支持多设备统计
- 标签数据（`book_tags.json`）可同样纳入同步

---

## 12. 验证标准

1. 打开一本书阅读 → 退出 → 打开统计页面，能看到正确的阅读时长和连续天数
2. 空闲 60 秒后计时暂停，恢复操作后继续
3. 关闭应用后重新打开，统计数据保持
4. 里程碑在达到阈值后自动标记为已完成
5. 阅读人格根据时段分布正确判定
6. 为书籍添加标签后，标签云按本周各标签阅读时长正确加权展示
7. 书详情页可编辑标签，标签变更即时反映到 `book_tags.json`
