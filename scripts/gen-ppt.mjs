import PptxGenJS from "pptxgenjs";
import fs from "fs";

const pptx = new PptxGenJS();
pptx.author = "Z-Pine";
pptx.title = "逐浪音乐 - MusicPlayer 项目总结";
pptx.subject = "Tauri v2 桌面音乐播放器";

// ============= 颜色主题 =============
const C = {
  bg: "0F172A",      // 深蓝黑背景
  card: "1E293B",    // 卡片色
  accent: "3B82F6",  // 蓝色强调
  accent2: "10B981", // 绿色（音乐库相关）
  accent3: "8B5CF6", // 紫色
  accent4: "F59E0B", // 金色
  text: "F1F5F9",    // 主文字色
  text2: "94A3B8",   // 次要文字
  white: "FFFFFF",
  darkBg: "0F172A",
  border: "1E293B",
};

// ============= 辅助函数 =============
function addSlide(title, subtitle) {
  const slide = pptx.addSlide();
  slide.background = { fill: C.bg };
  return slide;
}

function addTitle(slide, text, opts = {}) {
  slide.addText(text, {
    x: 0.6, y: 0.4, w: 9.0, h: 0.8,
    fontSize: 28, fontFace: "Microsoft YaHei",
    color: C.text, bold: true,
    ...opts,
  });
}

function addSubtitle(slide, text, y = 1.3) {
  slide.addText(text, {
    x: 0.6, y, w: 9.0, h: 0.5,
    fontSize: 14, fontFace: "Microsoft YaHei",
    color: C.text2,
  });
}

function addBody(slide, items, startY = 2.0, opts = {}) {
  let y = startY;
  const lineH = opts.lineH || 0.45;
  const fontSize = opts.fontSize || 13;
  const color = opts.color || C.text;

  for (const item of items) {
    if (typeof item === "string") {
      slide.addText(`• ${item}`, {
        x: 0.8, y, w: 8.6, h: lineH,
        fontSize, fontFace: "Microsoft YaHei",
        color, valign: "top",
      });
    } else {
      // { text, indent, color, bold, fontSize } object
      const x = 0.8 + (item.indent || 0) * 0.4;
      slide.addText(`${item.indent ? "  " : "•"} ${item.text}`, {
        x, y, w: 9.2 - x, h: lineH,
        fontSize: item.fontSize || fontSize,
        fontFace: "Microsoft YaHei",
        color: item.color || color,
        bold: item.bold || false,
        valign: "top",
      });
    }
    y += lineH;
  }
  return y;
}

function addCodeBlock(slide, code, y, w = 8.6) {
  slide.addShape(pptx.ShapeType.roundRect, {
    x: 0.8, y, w, h: 0.65,
    fill: { color: "1A2332" },
    line: { color: "2D3748", width: 0.5 },
    rectRadius: 4,
  });
  slide.addText(code, {
    x: 1.0, y: y + 0.08, w: w - 0.4, h: 0.5,
    fontSize: 10, fontFace: "Consolas",
    color: "E2E8F0",
  });
}


// ============= 幻灯片 =============

// --- 封面 ---
{
  const slide = pptx.addSlide();
  slide.background = { fill: C.bg };

  // 顶部装饰线
  slide.addShape(pptx.ShapeType.rect, {
    x: 0, y: 0, w: 10, h: 0.06, fill: { color: C.accent },
  });

  slide.addText("🎵 逐浪音乐", {
    x: 0.8, y: 1.8, w: 8.4, h: 1.0,
    fontSize: 42, fontFace: "Microsoft YaHei",
    color: C.white, bold: true,
  });

  slide.addText("MusicPlayer — 项目总结与技术分析", {
    x: 0.8, y: 2.8, w: 8.4, h: 0.6,
    fontSize: 20, fontFace: "Microsoft YaHei",
    color: C.text2,
  });

  slide.addShape(pptx.ShapeType.rect, {
    x: 0.8, y: 3.6, w: 2.0, h: 0.04, fill: { color: C.accent },
  });

  slide.addText([
    { text: "Tauri v2  ·  Vue 3  ·  Rust  ·  SQLite\n", options: { fontSize: 13, color: C.text2 } },
    { text: "2026 年 5 月", options: { fontSize: 11, color: "64748B" } },
  ], {
    x: 0.8, y: 3.9, w: 8.4, h: 0.8,
    fontFace: "Microsoft YaHei",
  });

  // 底部信息
  slide.addText("Author: Z-Pine  |  基于 Tauri v2 跨平台框架", {
    x: 0.8, y: 6.6, w: 8.4, h: 0.4,
    fontSize: 10, fontFace: "Microsoft YaHei",
    color: "475569",
  });
}

// --- 目录页 ---
{
  const slide = addSlide();
  addTitle(slide, "📋 目录");
  addSubtitle(slide, "CONTENTS");

  const items = [
    "项目概览",
    "技术架构",
    "核心功能解析",
    "数据库设计",
    "性能优化实践",
    "项目亮点与优势",
    "未来规划",
  ];

  items.forEach((item, i) => {
    const y = 2.0 + i * 0.65;
    slide.addShape(pptx.ShapeType.roundRect, {
      x: 0.8, y, w: 8.4, h: 0.5,
      fill: { color: i % 2 === 0 ? "1E293B" : "1A2332" },
      rectRadius: 4,
    });
    slide.addText(`0${i + 1}    ${item}`, {
      x: 1.2, y: y + 0.02, w: 7.6, h: 0.45,
      fontSize: 14, fontFace: "Microsoft YaHei",
      color: C.text,
    });
  });
}

// --- 项目概览 ---
{
  const slide = addSlide();
  addTitle(slide, "项目概览");
  addSubtitle(slide, "PROJECT OVERVIEW");

  addBody(slide, [
    "项目名称：逐浪音乐 (MusicPlayer)",
    "定位：自用桌面端音乐播放器，支持本地音乐库管理",
    "开发模式：Tauri v2（Rust 后端）+ Vue 3（前端）",
    "数据存储：SQLite 本地数据库，无需联网",
    "代码规模：22 个源文件，约 7,400 行代码",
    "测试覆盖：单元测试（Vitest）+ 端到端测试（Playwright）",
    "构建产物：原生 exe + MSI/NSIS 安装包",
  ], 2.0);
}

// --- 技术架构 ---
{
  const slide = addSlide();
  addTitle(slide, "技术架构");
  addSubtitle(slide, "TECHNICAL ARCHITECTURE");

  // 三层架构示意图 - 使用文字区域模拟
  const layers = [
    { label: "🎨 前端展示层 (Vue 3)", desc: "UI 组件 · 状态管理 (Pinia) · 路由 (Vue Router)", y: 2.0, color: C.accent },
    { label: "🔗 IPC 通信层 (Tauri API)", desc: "invoke 命令调用 · 事件监听 · 文件系统访问", y: 3.2, color: C.accent3 },
    { label: "⚙️ 后端服务层 (Rust)", desc: "Tauri 命令处理 · 元数据解析 (Lofty) · 音频解码", y: 4.4, color: C.accent2 },
    { label: "🗄️ 数据持久层 (SQLite)", desc: "rusqlite · 歌曲索引 · 播放列表 · 播放状态", y: 5.6, color: C.accent4 },
  ];

  for (const layer of layers) {
    slide.addShape(pptx.ShapeType.roundRect, {
      x: 0.8, y: layer.y, w: 8.4, h: 0.9,
      fill: { color: "1A2A4A" },
      line: { color: layer.color, width: 1.0 },
      rectRadius: 6,
    });
    slide.addText(layer.label, {
      x: 1.2, y: layer.y + 0.08, w: 7.6, h: 0.4,
      fontSize: 14, fontFace: "Microsoft YaHei",
      color: layer.color, bold: true,
    });
    slide.addText(layer.desc, {
      x: 1.2, y: layer.y + 0.45, w: 7.6, h: 0.35,
      fontSize: 11, fontFace: "Microsoft YaHei",
      color: C.text2,
    });
  }

  // 连接箭头
  for (let i = 0; i < 3; i++) {
    const arrowY = 2.0 + i * 1.2 + 0.9;
    slide.addText("▼", {
      x: 4.6, y: arrowY, w: 0.8, h: 0.3,
      fontSize: 10, color: "4B5563",
      align: "center",
    });
  }
}

// --- 技术栈详情 ---
{
  const slide = addSlide();
  addTitle(slide, "技术栈详情");
  addSubtitle(slide, "TECH STACK");

  const techs = [
    { cat: "前端框架", items: "Vue 3.5 + TypeScript 5.6 + Vite 6" },
    { cat: "状态管理", items: "Pinia 3（Setup Store 模式）" },
    { cat: "音频引擎", items: "Howler.js 2.2（Web Audio API 封装）" },
    { cat: "桌面框架", items: "Tauri v2.4（Rust 后端 + WebView 前端）" },
    { cat: "数据库", items: "SQLite via rusqlite 0.32（bundled 模式）" },
    { cat: "元数据解析", items: "Lofty 0.22（读取 ID3/FLAC/WMA 等标签）" },
    { cat: "插件生态", items: "对话框 · 文件系统 · 全局快捷键 · 通知" },
    { cat: "测试工具", items: "Vitest + @vue/test-utils + Playwright" },
  ];

  let y = 2.0;
  for (const t of techs) {
    slide.addText(t.cat, {
      x: 0.8, y, w: 2.2, h: 0.38,
      fontSize: 12, fontFace: "Microsoft YaHei",
      color: C.accent, bold: true,
    });
    slide.addText(t.items, {
      x: 3.2, y, w: 6.0, h: 0.38,
      fontSize: 12, fontFace: "Microsoft YaHei",
      color: C.text,
    });
    y += 0.52;
  }
}

// --- 核心功能：音乐库管理 ---
{
  const slide = addSlide();
  addTitle(slide, "核心功能：音乐库管理");
  addSubtitle(slide, "MUSIC LIBRARY");

  addBody(slide, [
    "📂 音乐文件夹扫描 — 支持多源目录，递归遍历音频文件",
    "  └ 支持格式：mp3 / flac / wav / aac / m4a / ogg / wma",
    "📝 元数据自动解析 — 使用 Lofty 库读取 ID3 标签",
    "  └ 标题 · 艺术家 · 专辑 · 年代 · 流派 · 比特率 · 采样率 · 封面图",
    "🗑️ 软删除机制 — 仅标记 deleted_from_library，不删磁盘文件",
    "  └ 歌单中保留已删歌曲记录，灰色标识，用户手动清理",
    "🔍 文件完整性检测 — 后台扫描标记缺失文件 (is_available)",
    "  └ 一键清理所有无效歌曲",
    "🔎 实时搜索 — 按标题 / 艺术家 / 专辑关键词模糊匹配",
  ], 2.0, { fontSize: 12, lineH: 0.38 });
}

// --- 核心功能：播放引擎 ---
{
  const slide = addSlide();
  addTitle(slide, "核心功能：播放引擎");
  addSubtitle(slide, "PLAYBACK ENGINE");

  addBody(slide, [
    "🎵 基于 Howler.js 的音频引擎",
    "  └ HTML5 Audio 模式 · 支持进度拖拽 · 音量控制",
    "🔄 四种播放模式",
    "  └ 顺序播放 · 列表循环 · 单曲循环 · 随机播放",
    "🛡️ 智能跳曲保护",
    "  └ 遇到不可用/已删除歌曲时自动跳过",
    "  └ 全部歌曲无效时自动停止",
    "💾 播放状态持久化",
    "  └ 每 10 秒自动保存 · 窗口关闭时保存",
    "  └ 下次启动自动恢复播放进度和音量",
    "📜 歌词同步 — 自动加载同目录 LRC 文件",
    "  └ 支持 UTF-8 / GBK / GB2312 编码自动检测",
  ], 2.0, { fontSize: 12, lineH: 0.38 });
}

// --- 核心功能：播放列表与歌单 ---
{
  const slide = addSlide();
  addTitle(slide, "核心功能：播放列表与歌单");
  addSubtitle(slide, "PLAYLISTS");

  addBody(slide, [
    "📁 自定义歌单 — 创建 · 重命名 · 删除",
    "  └ 从音乐库选择歌曲添加到歌单",
    "  └ 歌单歌曲去重保护",
    "🕐 最近播放 — 自动记录最近 100 首播放记录",
    "📋 多选操作 — Ctrl / Shift 多选 · 全选 · 批量添加到歌单",
    "  └ 批量从歌单移除 · 批量从音乐库删除",
    "🎯 右键菜单 — 播放 · 属性 · 添加到歌单 · 删除",
    "⚠️ 无效歌曲标识",
    "  └ 已删除歌曲：灰色 + 删除线",
    "  └ 文件缺失：标黄提示 · 一键清理",
    "📊 音乐库统计 — 总歌曲数 · 艺术家数 · 专辑数 · 总时长",
  ], 2.0, { fontSize: 12, lineH: 0.38 });
}

// --- 数据库设计 ---
{
  const slide = addSlide();
  addTitle(slide, "数据库设计");
  addSubtitle(slide, "DATABASE SCHEMA");

  // 表格 - 使用矩形模拟
  const tables = [
    { name: "songs", fields: "核心表：id / path / title / artist / album / duration / cover_art / bitrate / is_available / deleted_from_library" },
    { name: "library_sources", fields: "音乐库源：id / path / enabled" },
    { name: "playlists", fields: "歌单：id / name / created_at" },
    { name: "playlist_songs", fields: "关联表：playlist_id / song_id / position" },
    { name: "recent_plays", fields: "最近播放：song_id / played_at（上限 100 条）" },
    { name: "playback_state", fields: "播放状态：song_id / position_ms / volume / play_mode / is_playing" },
    { name: "lyrics_cache", fields: "歌词缓存：song_id / source / content" },
    { name: "settings", fields: "用户配置：key / value" },
  ];

  let y = 2.0;
  for (const t of tables) {
    slide.addShape(pptx.ShapeType.roundRect, {
      x: 0.8, y, w: 8.4, h: 0.48,
      fill: { color: y % 0.96 < 0.48 ? "1E293B" : "1A2332" },
      rectRadius: 3,
    });
    slide.addText(t.name, {
      x: 1.0, y: y + 0.02, w: 2.4, h: 0.44,
      fontSize: 11, fontFace: "Consolas", color: C.accent, bold: true,
    });
    slide.addText(t.fields, {
      x: 3.5, y: y + 0.02, w: 5.5, h: 0.44,
      fontSize: 10, fontFace: "Microsoft YaHei", color: C.text2,
    });
    y += 0.52;
  }

  addBody(slide, [
    "• 采用两级软删除机制：is_available（文件级） + deleted_from_library（用户级）",
    "• 外键级联（CASCADE），保证歌单/最近播放数据的引用完整性",
  ], y + 0.2, { fontSize: 11, lineH: 0.35 });
}

// --- 性能优化实践 ---
{
  const slide = addSlide();
  addTitle(slide, "性能优化实践");
  addSubtitle(slide, "PERFORMANCE OPTIMIZATION");

  const opts = [
    {
      title: "1. 封面图 BLOB 优化",
      desc: "列表查询使用 NULL as cover_art 替代全字段 SELECT，避免读取大量二进制数据，\"最近播放\" 加载速度提升约 30%",
    },
    {
      title: "2. 数据库迁移兼容",
      desc: "PRAGMA table_info + ALTER TABLE 实现旧版 schema 平滑升级",
    },
    {
      title: "3. 异步后台扫描",
      desc: "启动 5 秒后静默检测文件完整性，不阻塞 UI 渲染",
    },
    {
      title: "4. 防重复播放",
      desc: "loadSong 中先 stop + unload 旧 Howl 实例，杜绝多音频叠加",
    },
    {
      title: "5. 搜索防抖",
      desc: "300ms 防抖 + 后端 SQL LIKE 模糊匹配，避免高频重复查询",
    },
    {
      title: "6. Fire-and-Forget IPC",
      desc: "不依赖返回值的命令（如打开资源管理器）使用 fire-and-forget 模式",
    },
  ];

  let y = 2.0;
  for (const o of opts) {
    slide.addText(o.title, {
      x: 0.8, y, w: 8.4, h: 0.35,
      fontSize: 13, fontFace: "Microsoft YaHei",
      color: C.accent, bold: true,
    });
    slide.addText(o.desc, {
      x: 1.0, y: y + 0.35, w: 8.2, h: 0.4,
      fontSize: 11, fontFace: "Microsoft YaHei",
      color: C.text2,
    });
    y += 0.85;
  }
}

// --- 项目亮点与优势 ---
{
  const slide = addSlide();
  addTitle(slide, "项目亮点与优势");
  addSubtitle(slide, "HIGHLIGHTS & ADVANTAGES");

  const highlights = [
    {
      icon: "⚡",
      title: "轻量高效",
      desc: "基于 Tauri v2 原生框架，编译后 exe < 10MB，内存占用远低于 Electron 方案",
    },
    {
      icon: "🛡️",
      title: "安全可靠",
      desc: "Rust 后端无内存安全问题 · SQLite 本地存储无需联网 · 软删除保护数据",
    },
    {
      icon: "🎨",
      title: "深色沉浸 UI",
      desc: "深蓝黑主题配蓝/绿渐变 · 圆角卡片 · 平滑过渡动画 · 沉浸听歌体验",
    },
    {
      icon: "🔌",
      title: "完整功能闭环",
      desc: "扫描 → 解析 → 管理 → 播放 → 歌词 → 状态恢复，全链路覆盖",
    },
    {
      icon: "🧩",
      title: "组件化架构",
      desc: "Pinia Store 驱动 · 组件解耦 · 可复用 Composable · 类型安全",
    },
    {
      icon: "🔧",
      title: "可维护性强",
      desc: "TypeScript strict 模式 · Rust 编译期检查 · 单元测试覆盖核心逻辑",
    },
  ];

  let y = 2.0;
  for (const h of highlights) {
    slide.addShape(pptx.ShapeType.roundRect, {
      x: 0.8, y, w: 8.4, h: 0.7,
      fill: { color: "1E293B" },
      line: { color: "2D3748", width: 0.5 },
      rectRadius: 5,
    });
    slide.addText(`${h.icon} ${h.title}`, {
      x: 1.2, y: y + 0.03, w: 7.6, h: 0.3,
      fontSize: 13, fontFace: "Microsoft YaHei",
      color: C.white, bold: true,
    });
    slide.addText(h.desc, {
      x: 1.2, y: y + 0.33, w: 7.6, h: 0.3,
      fontSize: 10, fontFace: "Microsoft YaHei",
      color: C.text2,
    });
    y += 0.78;
  }
}

// --- 与 Electron 对比 ---
{
  const slide = addSlide();
  addTitle(slide, "Tauri vs Electron 对比");
  addSubtitle(slide, "WHY TAURI?");

  const headers = ["对比维度", "Tauri (本方案)", "Electron"];
  const rows = [
    ["安装包大小", "~5 MB", "~150 MB"],
    ["内存占用", "~50 MB", "~200 MB"],
    ["后端语言", "Rust（安全·高性能）", "Node.js"],
    ["前端技术", "系统 WebView", "Chromium 捆绑"],
    ["安全性", "强（内存安全 + 权限隔离）", "一般"],
    ["启动速度", "毫秒级", "秒级"],
  ];

  // 表头
  let y = 2.0;
  slide.addShape(pptx.ShapeType.rect, {
    x: 0.8, y, w: 2.4, h: 0.45,
    fill: { color: C.accent },
  });
  slide.addShape(pptx.ShapeType.rect, {
    x: 3.2, y, w: 3.2, h: 0.45,
    fill: { color: "2563EB" },
  });
  slide.addShape(pptx.ShapeType.rect, {
    x: 6.4, y, w: 2.8, h: 0.45,
    fill: { color: "475569" },
  });

  slide.addText("对比维度", { x: 0.8, y, w: 2.4, h: 0.45, fontSize: 11, fontFace: "Microsoft YaHei", color: C.white, bold: true, align: "center", valign: "middle" });
  slide.addText("Tauri (本方案)", { x: 3.2, y, w: 3.2, h: 0.45, fontSize: 11, fontFace: "Microsoft YaHei", color: C.white, bold: true, align: "center", valign: "middle" });
  slide.addText("Electron", { x: 6.4, y, w: 2.8, h: 0.45, fontSize: 11, fontFace: "Microsoft YaHei", color: C.white, bold: true, align: "center", valign: "middle" });

  y += 0.5;
  for (let i = 0; i < rows.length; i++) {
    const bg = i % 2 === 0 ? "1E293B" : "1A2332";
    slide.addShape(pptx.ShapeType.rect, { x: 0.8, y, w: 2.4, h: 0.4, fill: { color: bg } });
    slide.addShape(pptx.ShapeType.rect, { x: 3.2, y, w: 3.2, h: 0.4, fill: { color: bg } });
    slide.addShape(pptx.ShapeType.rect, { x: 6.4, y, w: 2.8, h: 0.4, fill: { color: bg } });
    slide.addText(rows[i][0], { x: 0.8, y, w: 2.4, h: 0.4, fontSize: 11, fontFace: "Microsoft YaHei", color: C.text, align: "center", valign: "middle" });
    slide.addText(rows[i][1], { x: 3.2, y, w: 3.2, h: 0.4, fontSize: 11, fontFace: "Microsoft YaHei", color: C.accent2, bold: true, align: "center", valign: "middle" });
    slide.addText(rows[i][2], { x: 6.4, y, w: 2.8, h: 0.4, fontSize: 11, fontFace: "Microsoft YaHei", color: C.text2, align: "center", valign: "middle" });
    y += 0.42;
  }
}

// --- 代码亮点 ---
{
  const slide = addSlide();
  addTitle(slide, "代码亮点展示");
  addSubtitle(slide, "CODE HIGHLIGHTS");

  addBody(slide, [
    { text: "Pinia Setup Store — 类型安全的状态管理", bold: true, color: C.accent, indent: 0 },
  ], 2.0, { fontSize: 12, lineH: 0.35 });

  addCodeBlock(slide,
    'const playSource = ref<PlaySource>({ type: \'library\' });\nconst playSourceLabel = computed(() => {\n  if (playSource.value.type === \'library\') return \'音乐库\';\n  return playSource.value.playlistName;\n});',
  2.35);

  addBody(slide, [
    { text: "智能跳曲保护 — 自动跳过无效歌曲", bold: true, color: C.accent, indent: 0 },
  ], 3.2, { fontSize: 12, lineH: 0.35 });

  addCodeBlock(slide,
    "for (let attempt = 0; attempt < playlist.value.length; attempt++) {\n  const result = getNextIndex(...);\n  if (result.shouldStop) { stop(); return; }\n  const song = playlist.value[result.nextIndex];\n  if (song && song.isAvailable && !song.deletedFromLibrary) {\n    loadSong(song, true); return;\n  }\n}\nstop(); // 全部无效",
  3.55);

  addBody(slide, [
    { text: "数据库迁移 — 平滑升级旧版 schema", bold: true, color: C.accent, indent: 0 },
  ], 5.1, { fontSize: 12, lineH: 0.35 });

  addCodeBlock(slide,
    "let has_column = conn.prepare(\"PRAGMA table_info(songs)\")?\n    .query_map([], |row| row.get::<_, String>(1))?\n    .any(|name| name.as_deref() == Ok(\"deleted_from_library\"));\nif (!has_column) {\n    conn.execute_batch(\"ALTER TABLE songs ADD COLUMN ...\");\n}",
  5.45);
}

// --- 未来规划 ---
{
  const slide = addSlide();
  addTitle(slide, "未来规划");
  addSubtitle(slide, "FUTURE ROADMAP");

  const items = [
    {
      phase: "近期",
      color: C.accent2,
      list: [
        "在线歌词搜索与同步显示",
        "音频可视化（频谱/波形）",
        "播放列表拖拽排序",
        "歌曲封面图展示",
      ],
    },
    {
      phase: "中期",
      color: C.accent,
      list: [
        "均衡器（EQ）调节",
        "全局快捷键深度定制",
        "导入/导出播放列表 (m3u/pls)",
        "主题切换（浅色/深色）",
      ],
    },
    {
      phase: "远期",
      color: C.accent3,
      list: [
        "跨设备播放状态同步",
        "流媒体扩展（NAS/WebDAV）",
        "移动端 Tauri 适配",
        "插件系统",
      ],
    },
  ];

  let x = 0.6;
  for (const section of items) {
    slide.addShape(pptx.ShapeType.roundRect, {
      x, y: 2.0, w: 2.9, h: 4.5,
      fill: { color: "1E293B" },
      line: { color: section.color, width: 1 },
      rectRadius: 8,
    });

    slide.addText(section.phase, {
      x, y: 2.15, w: 2.9, h: 0.4,
      fontSize: 14, fontFace: "Microsoft YaHei",
      color: section.color, bold: true, align: "center",
    });

    slide.addShape(pptx.ShapeType.rect, {
      x: x + 0.3, y: 2.6, w: 2.3, h: 0.02,
      fill: { color: section.color },
    });

    let iy = 2.8;
    for (const item of section.list) {
      slide.addText(`• ${item}`, {
        x: x + 0.2, y: iy, w: 2.5, h: 0.35,
        fontSize: 10, fontFace: "Microsoft YaHei",
        color: C.text2,
      });
      iy += 0.42;
    }

    x += 3.1;
  }
}

// --- 致谢 ---
{
  const slide = pptx.addSlide();
  slide.background = { fill: C.bg };

  slide.addShape(pptx.ShapeType.rect, {
    x: 0, y: 0, w: 10, h: 0.06, fill: { color: C.accent },
  });

  slide.addText("感谢聆听", {
    x: 0.8, y: 2.2, w: 8.4, h: 1.0,
    fontSize: 40, fontFace: "Microsoft YaHei",
    color: C.white, bold: true, align: "center",
  });

  slide.addShape(pptx.ShapeType.rect, {
    x: 4.2, y: 3.3, w: 1.6, h: 0.04,
    fill: { color: C.accent },
  });

  slide.addText("逐浪音乐 · 用 Rust 打造轻量桌面播放器", {
    x: 0.8, y: 3.6, w: 8.4, h: 0.5,
    fontSize: 14, fontFace: "Microsoft YaHei",
    color: C.text2, align: "center",
  });

  slide.addText("https://github.com/Z-Pine/MusicPlayer", {
    x: 0.8, y: 5.5, w: 8.4, h: 0.4,
    fontSize: 11, fontFace: "Consolas",
    color: C.accent, align: "center",
  });
}


// ============= 输出文件 =============
const outputPath = "f:\\VSCode Projects\\MusicPlayer\\逐浪音乐-MusicPlayer项目总结.pptx";
await pptx.writeFile({ fileName: outputPath });
console.log(`PPT generated: ${outputPath}`);
