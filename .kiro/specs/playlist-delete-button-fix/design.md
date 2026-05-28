# 歌单删除按钮无响应 Bugfix 设计文档

## 概述

本文档设计了修复歌单删除按钮无响应问题的方案。当前 `removeSongs` 函数依赖 `selectedSongIds` 集合来确定要删除的歌曲，当用户点击未选中歌曲的删除按钮时，由于集合为空导致函数直接返回。修复方案是让删除按钮传递歌曲 ID 参数，使 `removeSongs` 函数能够处理单个歌曲删除和批量删除两种场景。

## 术语表

- **Bug_Condition (C)**: 触发 bug 的条件 - 当用户点击未选中歌曲的删除按钮时
- **Property (P)**: 期望的行为 - 删除按钮应该删除指定的歌曲，无论是否被选中
- **Preservation**: 必须保持不变的现有行为 - 批量删除功能、右键菜单删除功能
- **removeSongs**: `PlaylistView.vue` 中的函数，负责从歌单中移除歌曲
- **selectedSongIds**: 响应式 Set 集合，存储当前选中的歌曲 ID
- **isRecent**: 计算属性，判断当前是否为"最近播放"列表

## Bug 详情

### Bug 条件

当用户点击歌曲右侧的删除按钮（❌）时，如果该歌曲未被选中（不在 `selectedSongIds` 集合中），`removeSongs` 函数会因为集合为空而直接返回，导致删除操作无响应。

**形式化规范:**
```
FUNCTION isBugCondition(input)
  INPUT: input of type { songId: number, isSelected: boolean, clickSource: string }
  OUTPUT: boolean
  
  RETURN input.clickSource = 'delete-button'
         AND input.isSelected = false
         AND selectedSongIds.size = 0
END FUNCTION
```

### 示例

- **示例 1**: 用户点击未选中歌曲（ID=5）的删除按钮 → 预期删除该歌曲 → 实际无响应
- **示例 2**: 用户选中歌曲（ID=3），然后点击另一首未选中歌曲（ID=7）的删除按钮 → 预期删除歌曲 7 → 实际删除歌曲 3（因为 selectedSongIds 包含 3）
- **示例 3**: 用户选中多首歌曲（ID=1,2,3），点击其中一首的删除按钮 → 预期删除所有选中的歌曲 → 实际正常工作（批量删除）
- **边缘情况**: 用户在"最近播放"列表中 → 删除按钮不显示 → 预期行为正确

## 预期行为

### 保留需求

**不变行为:**
- 批量删除功能必须继续工作：当用户选中多首歌曲后点击任意一首的删除按钮时，应删除所有选中的歌曲
- 右键菜单删除功能必须继续工作：通过右键菜单选择"从歌单移除"应正常删除选中的歌曲
- "最近播放"列表的删除按钮必须继续保持隐藏状态
- 删除操作成功后的 UI 反馈必须保持不变：显示 Toast 提示、刷新列表、清空选中状态

**范围:**
所有不涉及点击未选中歌曲删除按钮的操作都应完全不受此修复影响。这包括:
- 批量删除操作（选中多首歌曲后删除）
- 右键菜单删除操作
- 其他 UI 交互（播放、选择、拖拽等）

## 假设的根本原因

基于 bug 描述和代码分析，最可能的问题是:

1. **删除按钮未传递歌曲 ID**: 当前删除按钮的 `@click.stop="removeSongs"` 没有传递任何参数，导致函数无法知道要删除哪首歌曲

2. **函数逻辑依赖选中状态**: `removeSongs` 函数直接从 `selectedSongIds` 集合获取要删除的歌曲 ID，当集合为空时立即返回

3. **缺少参数处理逻辑**: 函数没有处理"删除指定歌曲"和"删除选中歌曲"两种场景的逻辑分支

4. **确认对话框消息不准确**: 当删除单个歌曲时，确认对话框应显示"删除 1 首歌曲"而不是基于 `selectedSongIds.size`

## 正确性属性

Property 1: Bug Condition - 删除按钮删除指定歌曲

_对于任何_ 点击删除按钮的操作，如果提供了歌曲 ID 参数，修复后的 `removeSongs` 函数应该删除该指定的歌曲，显示确认对话框提示删除 1 首歌曲，并在确认后执行删除操作，无论该歌曲是否被选中。

**验证需求: 2.1, 2.2, 2.3**

Property 2: Preservation - 批量删除和其他功能

_对于任何_ 不涉及点击未选中歌曲删除按钮的操作（批量删除、右键菜单删除、其他 UI 交互），修复后的代码应该产生与原始代码完全相同的行为，保留所有现有功能。

**验证需求: 3.1, 3.2, 3.3, 3.4, 3.5**

## 修复实现

### 需要的更改

假设我们的根本原因分析是正确的:

**文件**: `f:\VSCode Projects\MusicPlayer\src\views\PlaylistView.vue`

**函数**: `removeSongs`

**具体更改**:
1. **修改函数签名**: 添加可选参数 `songId?: number`
   - 当提供 `songId` 时，删除指定的歌曲
   - 当未提供 `songId` 时，删除所有选中的歌曲（保持原有批量删除逻辑）

2. **更新删除按钮调用**: 修改模板中的删除按钮，传递当前歌曲的 ID
   - 从 `@click.stop="removeSongs"` 改为 `@click.stop="removeSongs(song.id)"`

3. **更新函数逻辑**: 根据参数决定要删除的歌曲列表
   - 如果提供了 `songId`，使用 `[songId]` 作为删除列表
   - 如果未提供 `songId`，使用 `Array.from(selectedSongIds.value)` 作为删除列表

4. **更新确认对话框消息**: 根据实际删除数量显示准确的提示信息
   - 使用 `idsToRemove.length` 而不是 `selectedSongIds.value.size`

5. **保持右键菜单调用不变**: `removeSongsFromContext` 继续调用 `removeSongs()` 不传参数，保持批量删除行为

### 代码变更示例

**修改前:**
```typescript
async function removeSongs() {
  if (isRecent.value) return;
  if (playlistId.value === null) return;
  
  const idsToRemove = Array.from(selectedSongIds.value);
  if (idsToRemove.length === 0) return;
  // ...
}
```

**修改后:**
```typescript
async function removeSongs(songId?: number) {
  if (isRecent.value) return;
  if (playlistId.value === null) return;
  
  // 如果提供了 songId，删除指定歌曲；否则删除所有选中的歌曲
  const idsToRemove = songId !== undefined 
    ? [songId] 
    : Array.from(selectedSongIds.value);
  
  if (idsToRemove.length === 0) return;
  // ...
}
```

**模板修改前:**
```vue
<button
  v-if="!isRecent"
  class="remove-btn"
  @click.stop="removeSongs"
  :title="selectedSongIds.size > 0 ? `批量删除 ${selectedSongIds.size} 首` : '从歌单移除'"
>
  ✕
</button>
```

**模板修改后:**
```vue
<button
  v-if="!isRecent"
  class="remove-btn"
  @click.stop="removeSongs(song.id)"
  :title="selectedSongIds.size > 0 ? `批量删除 ${selectedSongIds.size} 首` : '从歌单移除'"
>
  ✕
</button>
```

## 测试策略

### 验证方法

测试策略遵循两阶段方法：首先在未修复的代码上暴露反例以演示 bug，然后验证修复后的代码正确工作且保留现有行为。

### 探索性 Bug 条件检查

**目标**: 在实现修复之前暴露演示 bug 的反例。确认或反驳根本原因分析。如果反驳，我们需要重新假设。

**测试计划**: 编写测试模拟点击未选中歌曲的删除按钮，并断言歌曲被删除。在未修复的代码上运行这些测试以观察失败并理解根本原因。

**测试用例**:
1. **单个未选中歌曲删除测试**: 模拟点击未选中歌曲（ID=5）的删除按钮（在未修复代码上会失败）
2. **选中其他歌曲时删除未选中歌曲测试**: 选中歌曲 3，点击歌曲 7 的删除按钮（在未修复代码上会失败或删除错误的歌曲）
3. **确认对话框消息测试**: 验证删除单个歌曲时对话框显示"删除 1 首歌曲"（在未修复代码上可能显示错误）
4. **边缘情况测试**: 在"最近播放"列表中验证删除按钮不显示（在未修复代码上应该正常）

**预期反例**:
- 点击未选中歌曲的删除按钮时，`removeSongs` 函数因 `selectedSongIds` 为空而直接返回
- 可能的原因：删除按钮未传递歌曲 ID、函数缺少参数处理逻辑、函数过度依赖选中状态

### 修复检查

**目标**: 验证对于所有满足 bug 条件的输入，修复后的函数产生预期行为。

**伪代码:**
```
FOR ALL input WHERE isBugCondition(input) DO
  result := removeSongs_fixed(input.songId)
  ASSERT expectedBehavior(result)
END FOR
```

**expectedBehavior 定义:**
```
FUNCTION expectedBehavior(result)
  INPUT: result of type { deleted: boolean, dialogShown: boolean, count: number }
  OUTPUT: boolean
  
  RETURN result.deleted = true
         AND result.dialogShown = true
         AND result.count = 1
END FUNCTION
```

### 保留检查

**目标**: 验证对于所有不满足 bug 条件的输入，修复后的函数产生与原始函数相同的结果。

**伪代码:**
```
FOR ALL input WHERE NOT isBugCondition(input) DO
  ASSERT removeSongs_original(input) = removeSongs_fixed(input)
END FOR
```

**测试方法**: 推荐使用基于属性的测试进行保留检查，因为:
- 它自动在输入域中生成许多测试用例
- 它能捕获手动单元测试可能遗漏的边缘情况
- 它为所有非 bug 输入提供强有力的保证，确保行为不变

**测试计划**: 首先在未修复的代码上观察批量删除和右键菜单删除的行为，然后编写基于属性的测试捕获这些行为模式。

**测试用例**:
1. **批量删除保留测试**: 在未修复代码上观察选中多首歌曲后点击删除按钮的行为，然后编写测试验证修复后此行为继续工作
2. **右键菜单删除保留测试**: 在未修复代码上观察右键菜单删除的行为，然后编写测试验证修复后此行为继续工作
3. **"最近播放"列表保留测试**: 在未修复代码上观察"最近播放"列表中删除按钮隐藏的行为，然后编写测试验证修复后此行为继续工作
4. **删除后 UI 反馈保留测试**: 在未修复代码上观察删除成功后的 Toast 提示、列表刷新、选中状态清空，然后编写测试验证修复后这些行为继续工作

### 单元测试

- 测试 `removeSongs` 函数带参数调用时删除指定歌曲
- 测试 `removeSongs` 函数不带参数调用时删除所有选中歌曲
- 测试确认对话框显示正确的删除数量
- 测试在"最近播放"列表中函数直接返回

### 基于属性的测试

- 生成随机歌单状态和选中状态，验证删除按钮传递歌曲 ID 时正确删除
- 生成随机批量选中配置，验证批量删除行为保持不变
- 测试在许多场景下右键菜单删除继续工作

### 集成测试

- 测试完整的删除流程：点击删除按钮 → 显示确认对话框 → 确认 → 歌曲被删除 → 显示 Toast → 列表刷新
- 测试在不同 UI 上下文中切换并使用删除功能
- 测试删除操作的视觉反馈和状态更新
