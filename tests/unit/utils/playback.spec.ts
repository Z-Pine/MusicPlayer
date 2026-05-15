import { describe, it, expect, vi } from "vitest";
import {
  createShuffleIndices,
  getNextIndex,
  getPreviousIndex,
} from "@/utils/playback";

describe("createShuffleIndices", () => {
  it("returns empty array for length 0", () => {
    expect(createShuffleIndices(0, 0)).toEqual([]);
  });

  it("returns [0] for length 1", () => {
    expect(createShuffleIndices(1, 0)).toEqual([0]);
  });

  it("shuffles indices and avoids starting with current", () => {
    // 使用 mock 随机种子使结果可预测
    const mathRandom = vi.spyOn(Math, "random");
    // 模拟一轮 Fisher-Yates 洗牌：
    // length=3, current=0
    // i=2: random=0.5 -> j=1, swap(2,1) -> [0,2,1]
    // i=1: random=0.3 -> j=0, swap(1,0) -> [2,0,1]
    // first element 2 !== 0, no extra swap needed
    mathRandom
      .mockReturnValueOnce(0.5)
      .mockReturnValueOnce(0.3);

    const result = createShuffleIndices(3, 0);
    expect(result).toEqual([2, 0, 1]);
    expect(new Set(result).size).toBe(3);

    mathRandom.mockRestore();
  });

  it("swaps first two elements if shuffle starts with current", () => {
    const mathRandom = vi.spyOn(Math, "random");
    // length=4, current=0
    // i=3: random=0.1 -> j=0, swap(3,0) -> [3,1,2,0]
    // i=2: random=0.1 -> j=0, swap(2,0) -> [2,1,3,0]
    // i=1: random=0.1 -> j=0, swap(1,0) -> [1,2,3,0]
    // first element 1 !== 0, ok
    mathRandom
      .mockReturnValueOnce(0.1)
      .mockReturnValueOnce(0.1)
      .mockReturnValueOnce(0.1);

    const result = createShuffleIndices(4, 0);
    expect(result).toEqual([1, 2, 3, 0]);

    mathRandom.mockRestore();
  });

  it("forces swap when current is at start after shuffle", () => {
    const mathRandom = vi.spyOn(Math, "random");
    // length=2, current=0
    // i=1: random=0 -> j=0, swap(1,0) -> [1,0]
    // first element 1 !== 0, ok
    mathRandom.mockReturnValueOnce(0);
    const result = createShuffleIndices(2, 0);
    expect(result).toEqual([1, 0]);
    mathRandom.mockRestore();
  });
});

describe("getNextIndex", () => {
  it("stops on sequence when reaching end", () => {
    const result = getNextIndex("sequence", 2, 3, [], 0);
    expect(result.shouldStop).toBe(true);
    expect(result.nextIndex).toBe(0); // fallback
  });

  it("wraps on loop_list", () => {
    const result = getNextIndex("loop_list", 2, 3, [], 0);
    expect(result.shouldStop).toBe(false);
    expect(result.nextIndex).toBe(0);
  });

  it("advances normally in the middle of list", () => {
    const result = getNextIndex("sequence", 0, 3, [], 0);
    expect(result.shouldStop).toBe(false);
    expect(result.nextIndex).toBe(1);
  });

  it("uses shuffle indices", () => {
    const shuffled = [2, 0, 1];
    const result = getNextIndex("shuffle", 0, 3, shuffled, 0);
    expect(result.nextIndex).toBe(2);
    expect(result.nextShuffleCount).toBe(1);
    expect(result.shouldStop).toBe(false);
  });

  it("returns stop for empty playlist", () => {
    const result = getNextIndex("sequence", 0, 0, [], 0);
    expect(result.shouldStop).toBe(true);
  });
});

describe("getPreviousIndex", () => {
  it("goes back normally in sequence", () => {
    const result = getPreviousIndex("sequence", 1, 3, [], 0);
    expect(result.prevIndex).toBe(0);
    expect(result.nextShuffleCount).toBe(0);
  });

  it("wraps from first to last", () => {
    const result = getPreviousIndex("sequence", 0, 3, [], 0);
    expect(result.prevIndex).toBe(2);
  });

  it("uses shuffle history", () => {
    const shuffled = [2, 0, 1];
    const result = getPreviousIndex("shuffle", 2, 3, shuffled, 2);
    expect(result.prevIndex).toBe(0); // shuffled[1]
    expect(result.nextShuffleCount).toBe(1);
  });

  it("does not go below shuffle count 0", () => {
    const result = getPreviousIndex("shuffle", 0, 3, [1, 2, 0], 0);
    // shufflePlayedCount <= 0, fallback to normal wrap
    expect(result.prevIndex).toBe(2);
    expect(result.nextShuffleCount).toBe(0);
  });
});
