import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { Howl } from "howler";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import {
  createShuffleIndices,
  getNextIndex,
  getPreviousIndex,
} from "../utils/playback";

export interface Song {
  id: number;
  path: string;
  title: string;
  artist: string;
  album: string;
  albumArtist: string;
  year?: number;
  genre: string;
  duration: number;
  coverArt?: number[];
  bitrate?: number;
  sampleRate?: number;
  isAvailable: boolean;
  updatedAt: number;
}

export interface PlaylistItem {
  id: number;
  name: string;
  createdAt: number;
}

export const usePlayerStore = defineStore("player", () => {
  let howl: Howl | null = null;
  let progressInterval: ReturnType<typeof setInterval> | null = null;

  const currentSong = ref<Song | null>(null);
  const isPlaying = ref(false);
  const progress = ref(0);
  const duration = ref(0);
  const volume = ref(0.8);
  const playlist = ref<Song[]>([]);
  const currentIndex = ref(0);
  const playMode = ref<"sequence" | "loop_list" | "loop_single" | "shuffle">("sequence");

  // Shuffle tracking
  const shuffledIndices = ref<number[]>([]);
  const shufflePlayedCount = ref(0);

  const hasSong = computed(() => currentSong.value !== null);

  function startProgressTimer() {
    stopProgressTimer();
    progressInterval = setInterval(() => {
      if (howl && isPlaying.value) {
        progress.value = (howl.seek() as number) || 0;
      }
    }, 250);
  }

  function stopProgressTimer() {
    if (progressInterval) {
      clearInterval(progressInterval);
      progressInterval = null;
    }
  }

  function handleSongEnd() {
    if (playMode.value === "loop_single") {
      if (howl) {
        howl.seek(0);
        howl.play();
      }
      return;
    }
    next();
  }

  function buildHowl(song: Song): Howl {
    // Windows 路径转义为 URL 格式
    const normalizedPath = song.path.replace(/\\/g, "/");
    const assetUrl = convertFileSrc(normalizedPath);
    console.log("Loading audio from:", assetUrl);

    return new Howl({
      src: [assetUrl],
      html5: false,
      volume: volume.value,
      onload() {
        duration.value = howl?.duration() || song.duration || 0;
        console.log("Audio loaded, duration:", duration.value);
      },
      onplay() {
        isPlaying.value = true;
        startProgressTimer();
        if (currentSong.value) {
          invoke("record_recent_play", { songId: currentSong.value.id }).catch(console.error);
        }
      },
      onpause() {
        isPlaying.value = false;
        stopProgressTimer();
      },
      onstop() {
        isPlaying.value = false;
        stopProgressTimer();
        progress.value = 0;
      },
      onend() {
        handleSongEnd();
      },
      onloaderror(_id, error) {
        console.error("Failed to load audio:", song.path, error);
        isPlaying.value = false;
      },
    });
  }

  function loadSong(song: Song, autoPlay = true) {
    if (howl) {
      howl.stop();
      howl.unload();
      howl = null;
    }
    stopProgressTimer();

    currentSong.value = song;
    progress.value = 0;
    duration.value = song.duration || 0;

    howl = buildHowl(song);
    if (autoPlay) {
      howl.play();
    } else {
      isPlaying.value = false;
    }
  }

  function play() {
    if (howl) {
      howl.play();
    } else if (currentSong.value) {
      loadSong(currentSong.value, true);
    }
  }

  function pause() {
    if (howl) {
      howl.pause();
    }
  }

  function togglePlay() {
    if (isPlaying.value) {
      pause();
    } else {
      play();
    }
  }

  function stop() {
    if (howl) {
      howl.stop();
    }
  }

  function next() {
    if (playlist.value.length === 0) return;

    if (playMode.value === "shuffle" && shuffledIndices.value.length !== playlist.value.length) {
      shuffledIndices.value = createShuffleIndices(playlist.value.length, currentIndex.value);
      shufflePlayedCount.value = 0;
    }

    const result = getNextIndex(
      playMode.value,
      currentIndex.value,
      playlist.value.length,
      shuffledIndices.value,
      shufflePlayedCount.value
    );

    if (result.shouldStop) {
      stop();
      return;
    }

    shufflePlayedCount.value = result.nextShuffleCount;
    currentIndex.value = result.nextIndex;
    const song = playlist.value[currentIndex.value];
    if (song) loadSong(song, true);
  }

  function previous() {
    if (playlist.value.length === 0) return;

    if (playMode.value === "shuffle" && shuffledIndices.value.length !== playlist.value.length) {
      shuffledIndices.value = createShuffleIndices(playlist.value.length, currentIndex.value);
      shufflePlayedCount.value = 0;
    }

    const result = getPreviousIndex(
      playMode.value,
      currentIndex.value,
      playlist.value.length,
      shuffledIndices.value,
      shufflePlayedCount.value
    );

    shufflePlayedCount.value = result.nextShuffleCount;
    currentIndex.value = result.prevIndex;
    const song = playlist.value[currentIndex.value];
    if (song) loadSong(song, true);
  }

  function setVolume(v: number) {
    const clamped = Math.max(0, Math.min(1, v));
    volume.value = clamped;
    if (howl) {
      howl.volume(clamped);
    }
  }

  function setProgress(p: number) {
    const clamped = Math.max(0, Math.min(duration.value, p));
    progress.value = clamped;
    if (howl) {
      howl.seek(clamped);
    }
  }

  function setPlayMode(mode: typeof playMode.value) {
    playMode.value = mode;
    if (mode !== "shuffle") {
      shuffledIndices.value = [];
      shufflePlayedCount.value = 0;
    }
  }

  function setPlaylist(songs: Song[], startIndex = 0) {
    playlist.value = songs;
    currentIndex.value = startIndex;
    shuffledIndices.value = [];
    shufflePlayedCount.value = 0;
    if (songs.length > 0) {
      loadSong(songs[startIndex], true);
    }
  }

  function playSong(song: Song, allSongs: Song[] = []) {
    if (allSongs.length > 0) {
      const idx = allSongs.findIndex((s) => s.id === song.id);
      if (idx >= 0) {
        setPlaylist(allSongs, idx);
      } else {
        setPlaylist([song], 0);
      }
    } else {
      setPlaylist([song], 0);
    }
  }

  async function savePlaybackState() {
    try {
      await invoke("save_playback_state_cmd", {
        stateData: {
          songId: currentSong.value?.id ?? null,
          positionMs: Math.floor(progress.value * 1000),
          volume: volume.value,
          playMode: playMode.value,
          isPlaying: isPlaying.value,
        },
      });
    } catch (e) {
      console.error("Failed to save playback state:", e);
    }
  }

  async function restorePlaybackState() {
    try {
      const state = await invoke<{
        songId: number | null;
        positionMs: number;
        volume: number;
        playMode: string;
        isPlaying: boolean;
      }>("load_playback_state_cmd");

      // 恢复音量
      if (state.volume != null) {
        setVolume(state.volume);
      }
      
      // 恢复播放模式
      if (state.playMode) {
        const mode = state.playMode as typeof playMode.value;
        if (["sequence", "loop_list", "loop_single", "shuffle"].includes(mode)) {
          setPlayMode(mode);
        }
      }

      // 恢复歌曲和播放进度
      if (state.songId != null) {
        const song = await invoke<Song | null>("get_song_by_id", { id: state.songId });
        if (song) {
          // 加载歌曲但不自动播放
          loadSong(song, false);
          
          // 恢复播放进度
          const positionSec = state.positionMs / 1000;
          if (positionSec > 0 && howl) {
            howl.seek(positionSec);
            progress.value = positionSec;
          }
          
          // 如果上次是播放状态，则继续播放
          if (state.isPlaying && howl) {
            // 延迟一下确保音频加载完成
            setTimeout(() => {
              if (howl) {
                howl.play();
              }
            }, 100);
          }
          
          console.log(`Restored playback: ${song.title} at ${positionSec.toFixed(1)}s`);
        }
      }
    } catch (e) {
      console.error("Failed to restore playback state:", e);
    }
  }

  function dispose() {
    if (howl) {
      howl.stop();
      howl.unload();
      howl = null;
    }
    stopProgressTimer();
  }

  return {
    currentSong,
    isPlaying,
    progress,
    duration,
    volume,
    playlist,
    currentIndex,
    playMode,
    hasSong,
    play,
    pause,
    togglePlay,
    stop,
    setSong: loadSong,
    next,
    previous,
    setVolume,
    setProgress,
    setPlayMode,
    setPlaylist,
    playSong,
    savePlaybackState,
    restorePlaybackState,
    dispose,
  };
});
