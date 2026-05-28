# 需求文档

## 简介

音乐库管理功能为音乐播放器提供完整的音乐库和歌单管理能力。该功能实现了非破坏性的音乐库删除机制（仅删除数据库记录，保留磁盘文件）、文件有效性检测、无效歌曲标识和清理，以及播放来源显示。系统确保用户数据安全，同时提供清晰的视觉反馈和流畅的播放体验。

## 术语表

- **Music_Library（音乐库）**: 存储在数据库中的音乐元数据集合，包含歌曲的路径、标题、艺术家等信息
- **Playlist（歌单）**: 用户创建的音乐集合，包含对音乐库中歌曲的引用
- **Database_Record（数据库记录）**: 存储在数据库中的歌曲元数据条目
- **Disk_File（磁盘文件）**: 存储在文件系统中的实际音频文件
- **Invalid_Song（无效歌曲）**: 数据库记录存在但磁盘文件不存在或已损坏的歌曲
- **Deleted_Song（已删除歌曲）**: 已从音乐库中删除数据库记录的歌曲
- **File_Scanner（文件扫描器）**: 后台服务，用于检测磁盘文件的有效性
- **Player（播放器）**: 负责音频播放的核心组件
- **Playback_Source（播放来源）**: 当前播放队列的来源（音乐库、歌单或最近播放）
- **UI_Thread（UI线程）**: 用户界面的主执行线程
- **Background_Task（后台任务）**: 在独立线程中异步执行的操作

## 需求

### 需求 1: 非破坏性音乐库删除

**用户故事:** 作为用户，我希望从音乐库删除歌曲时不删除磁盘文件，以便保留原始音频文件供其他用途使用。

#### 验收标准

1. WHEN 用户从音乐库删除歌曲时，THE Music_Library SHALL 从数据库中删除该歌曲的 Database_Record
2. WHEN 用户从音乐库删除歌曲时，THE Music_Library SHALL 保留该歌曲的 Disk_File 不被删除
3. WHEN 用户触发删除操作时，THE Music_Library SHALL 显示确认对话框，说明将删除数据库记录但保留磁盘文件
4. WHEN 用户确认删除操作后，THE Music_Library SHALL 在 500 毫秒内完成数据库记录的删除
5. WHEN 删除操作完成后，THE Music_Library SHALL 从音乐库视图中移除该歌曲的显示

### 需求 2: 歌单中已删除歌曲的标识

**用户故事:** 作为用户，我希望在歌单中看到哪些歌曲已从音乐库删除，以便了解歌单的完整性状态。

#### 验收标准

1. WHEN 歌曲已从音乐库删除时，THE Playlist SHALL 在歌单视图中保留该歌曲的显示
2. WHEN 歌曲已从音乐库删除时，THE Playlist SHALL 对该歌曲应用视觉标识（灰色背景或半透明效果，透明度为 50%）
3. WHEN 歌曲已从音乐库删除时，THE Playlist SHALL 在该歌曲旁显示"已删除"标签或图标
4. WHEN 用户将鼠标悬停在已删除歌曲上时，THE Playlist SHALL 显示提示信息"此歌曲已从音乐库删除"

### 需求 3: 手动清理歌单中的无效歌曲

**用户故事:** 作为用户，我希望能够批量清理歌单中的无效歌曲，以便维护歌单的整洁性。

#### 验收标准

1. THE Playlist SHALL 在歌单视图中提供"清理无效歌曲"按钮
2. WHEN 用户点击"清理无效歌曲"按钮时，THE Playlist SHALL 显示确认对话框，说明将要删除的无效歌曲数量
3. WHEN 用户确认清理操作后，THE Playlist SHALL 从歌单中移除所有已从音乐库删除的歌曲
4. WHEN 用户确认清理操作后，THE Playlist SHALL 从歌单中移除所有磁盘文件不存在的歌曲
5. WHEN 清理操作完成后，THE Playlist SHALL 显示操作结果（例如"已清理 5 首无效歌曲"）

### 需求 4: 播放时跳过已删除歌曲

**用户故事:** 作为用户，我希望播放器在播放过程中自动跳过已从音乐库删除的歌曲，以便获得流畅的播放体验。

#### 验收标准

1. WHEN 播放队列中的下一首歌曲已从音乐库删除时，THE Player SHALL 跳过该歌曲并播放下一首有效歌曲
2. WHEN 播放器跳过已删除歌曲时，THE Player SHALL 在 200 毫秒内切换到下一首有效歌曲
3. WHEN 播放器跳过已删除歌曲时，THE Player SHALL 在界面上显示提示信息"已跳过无效歌曲"，持续 2 秒
4. WHEN 播放队列中所有剩余歌曲都已从音乐库删除时，THE Player SHALL 停止播放并显示"播放队列中没有有效歌曲"

### 需求 5: 后台文件有效性扫描

**用户故事:** 作为用户，我希望系统能够自动检测音乐库中的文件是否存在或已损坏，以便及时发现和处理无效歌曲。

#### 验收标准

1. THE File_Scanner SHALL 在应用启动后 10 秒内开始执行文件有效性扫描
2. THE File_Scanner SHALL 在 Background_Task 中执行扫描操作，不阻塞 UI_Thread
3. WHEN 扫描过程中检测到 Disk_File 不存在时，THE File_Scanner SHALL 在数据库中标记该歌曲为 Invalid_Song
4. WHEN 扫描过程中检测到 Disk_File 已损坏（无法读取或解析）时，THE File_Scanner SHALL 在数据库中标记该歌曲为 Invalid_Song
5. WHEN 扫描完成后，THE File_Scanner SHALL 记录扫描结果（扫描歌曲总数、发现的无效歌曲数量）
6. THE File_Scanner SHALL 每次扫描单个文件的时间不超过 100 毫秒
7. WHEN 扫描过程中系统资源使用率超过 80% 时，THE File_Scanner SHALL 暂停扫描并在资源使用率降低后恢复

### 需求 6: 手动触发文件扫描

**用户故事:** 作为用户，我希望能够手动触发文件有效性扫描，以便在需要时立即检查音乐库的完整性。

#### 验收标准

1. THE Music_Library SHALL 在音乐库视图中提供"扫描文件有效性"按钮
2. WHEN 用户点击"扫描文件有效性"按钮时，THE File_Scanner SHALL 立即开始扫描操作
3. WHEN 扫描正在进行时，THE Music_Library SHALL 显示扫描进度（已扫描数量/总数量，百分比）
4. WHEN 扫描正在进行时，THE Music_Library SHALL 提供"取消扫描"按钮
5. WHEN 用户点击"取消扫描"按钮时，THE File_Scanner SHALL 在 1 秒内停止扫描操作
6. WHEN 扫描完成后，THE Music_Library SHALL 显示扫描结果摘要（总数、有效数、无效数）

### 需求 7: 音乐库中无效歌曲的标识

**用户故事:** 作为用户，我希望在音乐库中看到哪些歌曲的文件不存在或已损坏，以便决定是否清理这些记录。

#### 验收标准

1. WHEN 歌曲被标记为 Invalid_Song 时，THE Music_Library SHALL 在音乐库视图中对该歌曲应用视觉标识（红色边框或警告图标）
2. WHEN 歌曲被标记为 Invalid_Song 时，THE Music_Library SHALL 在该歌曲旁显示"文件不存在"或"文件已损坏"图标
3. WHEN 用户将鼠标悬停在 Invalid_Song 上时，THE Music_Library SHALL 显示提示信息，包含文件路径和错误原因
4. THE Music_Library SHALL 在音乐库视图顶部显示无效歌曲的统计数量（例如"发现 3 首无效歌曲"）

### 需求 8: 一键清理音乐库中的无效歌曲

**用户故事:** 作为用户，我希望能够批量清理音乐库中的无效歌曲，以便快速移除所有无效记录。

#### 验收标准

1. WHEN 音乐库中存在 Invalid_Song 时，THE Music_Library SHALL 显示"清理无效歌曲"按钮
2. WHEN 音乐库中不存在 Invalid_Song 时，THE Music_Library SHALL 隐藏或禁用"清理无效歌曲"按钮
3. WHEN 用户点击"清理无效歌曲"按钮时，THE Music_Library SHALL 显示确认对话框，列出将要删除的无效歌曲数量
4. WHEN 用户确认清理操作后，THE Music_Library SHALL 从数据库中删除所有标记为 Invalid_Song 的 Database_Record
5. WHEN 清理操作完成后，THE Music_Library SHALL 显示操作结果（例如"已清理 3 首无效歌曲"）
6. WHEN 清理操作完成后，THE Music_Library SHALL 刷新音乐库视图，移除已清理歌曲的显示

### 需求 9: 歌单中无效文件的标识

**用户故事:** 作为用户，我希望在歌单中看到哪些歌曲的文件不存在或已损坏，以便了解歌单的可播放性。

#### 验收标准

1. WHEN 歌单中的歌曲被标记为 Invalid_Song 时，THE Playlist SHALL 对该歌曲应用视觉标识（橙色边框或警告图标）
2. WHEN 歌单中的歌曲被标记为 Invalid_Song 时，THE Playlist SHALL 在该歌曲旁显示"文件不存在"图标
3. WHEN 歌单中的歌曲同时是 Deleted_Song 和 Invalid_Song 时，THE Playlist SHALL 优先显示"已删除"标识
4. WHEN 用户将鼠标悬停在 Invalid_Song 上时，THE Playlist SHALL 显示提示信息"文件不存在或已损坏"

### 需求 10: 播放时跳过无效文件

**用户故事:** 作为用户，我希望播放器在播放过程中自动跳过文件不存在或已损坏的歌曲，以便获得流畅的播放体验。

#### 验收标准

1. WHEN 播放队列中的下一首歌曲被标记为 Invalid_Song 时，THE Player SHALL 跳过该歌曲并播放下一首有效歌曲
2. WHEN 播放器尝试播放 Invalid_Song 时，THE Player SHALL 在 200 毫秒内检测到文件无效并跳过
3. WHEN 播放器跳过 Invalid_Song 时，THE Player SHALL 在界面上显示提示信息"文件不存在，已跳过"，持续 2 秒
4. WHEN 播放队列中所有剩余歌曲都是 Invalid_Song 时，THE Player SHALL 停止播放并显示"播放队列中没有有效歌曲"

### 需求 11: 播放来源显示

**用户故事:** 作为用户，我希望在播放器界面看到当前播放的来源，以便了解正在播放的内容来自哪里。

#### 验收标准

1. THE Player SHALL 在播放器界面的左下角"正在播放"区域显示 Playback_Source
2. WHEN 用户从音乐库开始播放时，THE Player SHALL 显示"音乐库"作为 Playback_Source
3. WHEN 用户从歌单开始播放时，THE Player SHALL 显示"歌单：[歌单名称]"作为 Playback_Source
4. WHEN 用户从最近播放列表开始播放时，THE Player SHALL 显示"最近播放"作为 Playback_Source
5. WHEN Playback_Source 发生变化时，THE Player SHALL 在 100 毫秒内更新显示
6. WHEN 用户将鼠标悬停在 Playback_Source 上时，THE Player SHALL 显示完整的来源信息（如果歌单名称过长被截断）

### 需求 12: 空闲时自动扫描

**用户故事:** 作为用户，我希望系统能够在空闲时自动扫描文件有效性，以便在不影响使用体验的情况下保持音乐库的准确性。

#### 验收标准

1. WHEN 应用空闲超过 5 分钟时，THE File_Scanner SHALL 自动开始文件有效性扫描
2. WHEN 用户开始与应用交互时，THE File_Scanner SHALL 暂停正在进行的空闲扫描
3. WHEN 用户停止交互并再次空闲超过 5 分钟时，THE File_Scanner SHALL 从上次暂停的位置恢复扫描
4. WHEN 空闲扫描完成后，THE File_Scanner SHALL 静默记录结果，不显示通知（除非发现大量无效歌曲）
5. WHEN 空闲扫描发现超过 10 首 Invalid_Song 时，THE File_Scanner SHALL 显示通知提示用户查看

### 需求 13: 扫描性能优化

**用户故事:** 作为用户，我希望文件扫描不会影响播放器的性能和响应速度，以便在扫描过程中正常使用应用。

#### 验收标准

1. THE File_Scanner SHALL 使用独立的 Background_Task 执行所有扫描操作
2. THE File_Scanner SHALL 在扫描过程中每检查 10 个文件后暂停 50 毫秒，以降低系统负载
3. WHEN 播放器正在播放音乐时，THE File_Scanner SHALL 降低扫描优先级，确保播放不受影响
4. WHEN 扫描过程中 UI_Thread 响应时间超过 100 毫秒时，THE File_Scanner SHALL 进一步降低扫描速度
5. THE File_Scanner SHALL 使用文件系统缓存来减少磁盘 I/O 操作
6. THE File_Scanner SHALL 批量更新数据库（每 50 个文件更新一次），而不是逐个更新

### 需求 14: 删除操作的确认对话框

**用户故事:** 作为用户，我希望在执行删除操作前看到清晰的确认对话框，以便避免误操作。

#### 验收标准

1. WHEN 用户触发删除操作时，THE Music_Library SHALL 显示确认对话框
2. THE Music_Library SHALL 在确认对话框中说明操作的影响（删除数据库记录但保留磁盘文件）
3. THE Music_Library SHALL 在确认对话框中显示将要删除的歌曲数量
4. THE Music_Library SHALL 在确认对话框中提供"确认"和"取消"按钮
5. WHEN 用户点击"取消"按钮时，THE Music_Library SHALL 关闭对话框并取消删除操作
6. WHEN 用户点击"确认"按钮时，THE Music_Library SHALL 执行删除操作并关闭对话框
7. THE Music_Library SHALL 在确认对话框中使用清晰易懂的语言，避免技术术语

### 需求 15: 视觉标识的清晰性

**用户故事:** 作为用户，我希望无效歌曲和已删除歌曲的视觉标识清晰易懂，以便快速识别歌曲状态。

#### 验收标准

1. THE Music_Library SHALL 使用不同的视觉标识区分 Invalid_Song 和 Deleted_Song
2. THE Music_Library SHALL 使用图标和颜色的组合来标识歌曲状态，确保色盲用户也能识别
3. THE Music_Library SHALL 在设置中提供视觉标识的说明和示例
4. THE Playlist SHALL 使用与音乐库一致的视觉标识风格
5. WHEN 歌曲同时具有多个状态时，THE Music_Library SHALL 使用组合图标或优先级规则显示最重要的状态
