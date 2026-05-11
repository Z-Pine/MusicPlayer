export type PlayMode = "sequence" | "loop_list" | "loop_single" | "shuffle";

/**
 * 生成洗牌后的索引数组，确保第一个元素不与当前索引相同（避免连续重复）
 */
export function createShuffleIndices(
  length: number,
  currentIndex: number
): number[] {
  const indices = Array.from({ length }, (_, i) => i);
  for (let i = indices.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [indices[i], indices[j]] = [indices[j], indices[i]];
  }
  if (length > 1 && indices[0] === currentIndex) {
    [indices[0], indices[1]] = [indices[1], indices[0]];
  }
  return indices;
}

export interface NextIndexResult {
  index: number;
  shouldStop: boolean;
}

/**
 * 计算下一首歌曲的索引
 */
export function getNextIndex(
  mode: PlayMode,
  currentIndex: number,
  playlistLength: number,
  shuffledIndices: number[],
  shufflePlayedCount: number
): { nextIndex: number; nextShuffleCount: number; shouldStop: boolean } {
  if (playlistLength === 0) {
    return { nextIndex: 0, nextShuffleCount: shufflePlayedCount, shouldStop: true };
  }

  if (mode === "shuffle") {
    const idx = shuffledIndices[shufflePlayedCount % shuffledIndices.length];
    return {
      nextIndex: idx,
      nextShuffleCount: shufflePlayedCount + 1,
      shouldStop: false,
    };
  }

  const next = currentIndex + 1;
  if (mode === "sequence" && next >= playlistLength) {
    return { nextIndex: 0, nextShuffleCount: shufflePlayedCount, shouldStop: true };
  }

  return {
    nextIndex: next % playlistLength,
    nextShuffleCount: shufflePlayedCount,
    shouldStop: false,
  };
}

/**
 * 计算上一首歌曲的索引
 */
export function getPreviousIndex(
  mode: PlayMode,
  currentIndex: number,
  playlistLength: number,
  shuffledIndices: number[],
  shufflePlayedCount: number
): { prevIndex: number; nextShuffleCount: number } {
  if (playlistLength === 0) {
    return { prevIndex: 0, nextShuffleCount: shufflePlayedCount };
  }

  if (mode === "shuffle" && shufflePlayedCount > 0) {
    const idx = shuffledIndices[(shufflePlayedCount - 1) % shuffledIndices.length];
    return { prevIndex: idx, nextShuffleCount: shufflePlayedCount - 1 };
  }

  return {
    prevIndex: (currentIndex - 1 + playlistLength) % playlistLength,
    nextShuffleCount: shufflePlayedCount,
  };
}
