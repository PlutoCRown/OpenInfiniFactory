"""生成更丰富的背景音乐，以及放置/删除/框选相关音效。

输出覆盖：
  assets/audio/music.wav
  assets/audio/block_place.wav
  assets/audio/block_break.wav
  assets/audio/selection_tick.wav
"""

from __future__ import annotations

import math
import wave
from pathlib import Path

import numpy as np

RATE = 22050
ROOT = Path(__file__).resolve().parents[2] / "assets" / "audio"
HEADROOM = 0.89


def write_wav(name: str, samples: np.ndarray, *, stereo: bool = False) -> None:
    samples = np.asarray(samples, dtype=np.float64)
    if samples.ndim == 1:
        if stereo:
            samples = np.stack([samples, samples], axis=1)
    elif samples.ndim == 2 and samples.shape[1] == 2:
        stereo = True
    else:
        raise ValueError(f"unexpected sample shape: {samples.shape}")

    peak = float(np.max(np.abs(samples))) or 1.0
    samples = samples * (HEADROOM / peak)
    pcm = np.clip(samples * 32767.0, -32767, 32767).astype(np.int16)
    ROOT.mkdir(parents=True, exist_ok=True)
    with wave.open(str(ROOT / name), "wb") as output:
        output.setnchannels(2 if stereo else 1)
        output.setsampwidth(2)
        output.setframerate(RATE)
        output.writeframes(pcm.tobytes())


def envelope(n: int, attack: float, release: float) -> np.ndarray:
    attack_n = max(1, int(attack * RATE))
    release_n = max(1, int(release * RATE))
    env = np.ones(n, dtype=np.float64)
    env[:attack_n] = np.linspace(0.0, 1.0, attack_n, endpoint=False)
    if release_n < n:
        env[-release_n:] = np.linspace(1.0, 0.0, release_n)
    elif n > attack_n:
        env[attack_n:] = np.linspace(1.0, 0.0, n - attack_n)
    return env


def one_pole_lowpass(x: np.ndarray, cutoff_hz: float) -> np.ndarray:
    # y[n] = y[n-1] + a * (x[n] - y[n-1])
    a = 1.0 - math.exp(-2.0 * math.pi * cutoff_hz / RATE)
    y = np.empty_like(x)
    prev = 0.0
    for i, sample in enumerate(x):
        prev += a * (float(sample) - prev)
        y[i] = prev
    return y


def one_pole_highpass(x: np.ndarray, cutoff_hz: float) -> np.ndarray:
    return x - one_pole_lowpass(x, cutoff_hz)


def soft_noise(n: int, seed: int) -> np.ndarray:
    rng = np.random.default_rng(seed)
    return rng.uniform(-1.0, 1.0, n)


def midi_hz(note: float) -> float:
    return 440.0 * (2.0 ** ((note - 69.0) / 12.0))


def sine(freq: float, t: np.ndarray, phase: float = 0.0) -> np.ndarray:
    return np.sin(2.0 * math.pi * freq * t + phase)


def make_music(seconds: float = 32.0) -> np.ndarray:
    """偏工厂氛围的慢速循环：垫音和弦 + 低音脉动 + 轻琶音。"""
    n = int(RATE * seconds)
    t = np.arange(n, dtype=np.float64) / RATE
    bpm = 72.0
    beat = 60.0 / bpm
    bar = beat * 4.0

    # Am - F - C - G 循环（四小节）
    chords = np.array(
        [
            [45, 48, 52],  # A2 C3 E3
            [41, 45, 48],  # F2 A2 C3
            [48, 52, 55],  # C3 E3 G3
            [43, 47, 50],  # G2 B2 D3
        ],
        dtype=np.float64,
    )

    left = np.zeros(n, dtype=np.float64)
    right = np.zeros(n, dtype=np.float64)

    # 垫音：按小节切和弦，边界用慢淡化
    bar_index = (t / bar).astype(np.int64) % len(chords)
    local = (t % bar) / bar
    chord_fade = np.clip(np.minimum(local * 8.0, (1.0 - local) * 8.0), 0.0, 1.0)
    chord_fade = 0.15 + 0.85 * np.sin(math.pi * chord_fade)
    for voice, pan in enumerate((-0.35, 0.0, 0.35)):
        notes = chords[bar_index, voice]
        freq = midi_hz_array(notes)
        wobble = 1.0 + 0.003 * np.sin(2.0 * math.pi * (0.08 + 0.02 * voice) * t)
        tone = 0.55 * np.sin(2.0 * math.pi * freq * wobble * t)
        tone += 0.18 * (
            2.0 * np.abs(2.0 * ((freq * 0.5 * wobble * t) % 1.0) - 1.0) - 1.0
        )
        tone *= 0.22 * chord_fade
        left += tone * (0.5 - 0.5 * pan)
        right += tone * (0.5 + 0.5 * pan)

    # 低音脉动（每拍一次柔和冲击）
    for beat_i in range(int(seconds / beat) + 1):
        start = int(beat_i * beat * RATE)
        length = int(0.28 * RATE)
        end = min(n, start + length)
        if start >= n:
            break
        tt = np.arange(end - start, dtype=np.float64) / RATE
        bar_i = int(((beat_i * beat) / bar) % len(chords))
        root = midi_hz(chords[bar_i][0] - 12)
        body = np.sin(2.0 * math.pi * root * tt) * np.exp(-tt * 7.0)
        body += 0.35 * np.sin(2.0 * math.pi * root * 2.0 * tt) * np.exp(-tt * 10.0)
        pulse = body * envelope(end - start, 0.005, 0.22)
        left[start:end] += pulse * 0.28
        right[start:end] += pulse * 0.28

    # 轻琶音（每小节后半）
    arp_pattern = [0, 2, 1, 2]
    for bar_i in range(int(seconds / bar) + 1):
        chord = chords[bar_i % len(chords)]
        for step, idx in enumerate(arp_pattern):
            start_t = bar_i * bar + 2.0 * beat + step * (beat * 0.5)
            start = int(start_t * RATE)
            length = int(0.35 * RATE)
            end = min(n, start + length)
            if start >= n or start < 0:
                continue
            tt = np.arange(end - start, dtype=np.float64) / RATE
            freq = midi_hz(chord[idx] + 12)
            note = np.sin(2.0 * math.pi * freq * tt) * np.exp(-tt * 5.5)
            note += 0.25 * np.sin(2.0 * math.pi * freq * 2.0 * tt) * np.exp(-tt * 8.0)
            note *= envelope(end - start, 0.01, 0.25)
            pan = -0.4 + 0.25 * step
            left[start:end] += note * 0.12 * (0.5 - 0.5 * pan)
            right[start:end] += note * 0.12 * (0.5 + 0.5 * pan)

    # 高频微闪（短噪声经简易滤波）
    shimmer = soft_noise(n, seed=19)
    # 向量化近似高通：减去强低通
    shimmer_lp = np.convolve(shimmer, np.ones(24) / 24.0, mode="same")
    shimmer = shimmer - shimmer_lp
    shimmer = np.convolve(shimmer, np.ones(8) / 8.0, mode="same")
    shimmer *= 0.035 * (0.55 + 0.45 * np.sin(2.0 * math.pi * 0.07 * t))
    left += shimmer * 0.85
    right += shimmer * 1.15

    # 首尾交叉淡化，便于 LOOP
    cross = int(0.75 * RATE)
    fade = np.linspace(0.0, 1.0, cross)
    left[-cross:] = left[-cross:] * (1.0 - fade) + left[:cross] * fade
    right[-cross:] = right[-cross:] * (1.0 - fade) + right[:cross] * fade

    return np.stack([left, right], axis=1)


def midi_hz_array(notes: np.ndarray) -> np.ndarray:
    return 440.0 * (2.0 ** ((notes - 69.0) / 12.0))


def make_block_place() -> np.ndarray:
    """低沉放置冲击：低频体感 + 短瞬态。"""
    duration = 0.2
    n = int(RATE * duration)
    t = np.arange(n, dtype=np.float64) / RATE
    rng = soft_noise(n, seed=7)

    # 主低频撞击（约 95Hz，快速下扫）
    freq = 115.0 * np.exp(-t * 9.0) + 70.0
    phase = 2.0 * math.pi * np.cumsum(freq) / RATE
    body = np.sin(phase) * np.exp(-t * 14.0)

    # 次低频共鸣
    thud = sine(55.0, t) * np.exp(-t * 10.0)

    # 短瞬态噪声（闷）
    click = one_pole_lowpass(rng, 900.0) * np.exp(-t * 55.0)

    # 轻微中频木质感
    wood = sine(210.0, t) * np.exp(-t * 28.0)

    samples = 0.72 * body + 0.45 * thud + 0.22 * click + 0.16 * wood
    samples *= envelope(n, 0.001, 0.08)
    return samples


def make_block_break() -> np.ndarray:
    """鼠标键帽式咔哒：短促、偏硬、清晰。"""
    duration = 0.09
    n = int(RATE * duration)
    t = np.arange(n, dtype=np.float64) / RATE
    rng = soft_noise(n, seed=21)

    # 壳体共鸣
    shell = sine(980.0, t) * np.exp(-t * 70.0)
    shell += 0.55 * sine(1450.0, t) * np.exp(-t * 95.0)

    # 开关瞬态
    transient = one_pole_highpass(rng, 1200.0) * np.exp(-t * 180.0)
    transient = one_pole_lowpass(transient, 6500.0)

    # 轻微二次回弹（真鼠标常见）
    bounce_at = int(0.018 * RATE)
    bounce = np.zeros(n)
    if bounce_at < n:
        bt = t[: n - bounce_at]
        bounce[bounce_at:] = 0.35 * sine(1200.0, bt) * np.exp(
            -bt * 120.0
        ) + 0.2 * one_pole_highpass(rng[: n - bounce_at], 1800.0) * np.exp(-bt * 200.0)

    samples = 0.55 * shell + 0.7 * transient + bounce
    samples *= envelope(n, 0.0005, 0.035)
    return samples


def make_selection_tick() -> np.ndarray:
    """框选尺寸变化时的轻微咔哒，比删除键更轻更短。"""
    duration = 0.045
    n = int(RATE * duration)
    t = np.arange(n, dtype=np.float64) / RATE
    rng = soft_noise(n, seed=33)

    tick = sine(1650.0, t) * np.exp(-t * 140.0)
    tick += 0.35 * sine(2400.0, t) * np.exp(-t * 180.0)
    noise = one_pole_highpass(rng, 1500.0) * np.exp(-t * 220.0)
    samples = 0.45 * tick + 0.25 * noise
    samples *= envelope(n, 0.0004, 0.02)
    return samples * 0.55


def main() -> None:
    write_wav("music.wav", make_music(), stereo=True)
    write_wav("block_place.wav", make_block_place())
    write_wav("block_break.wav", make_block_break())
    write_wav("selection_tick.wav", make_selection_tick())
    for name in (
        "music.wav",
        "block_place.wav",
        "block_break.wav",
        "selection_tick.wav",
    ):
        with wave.open(str(ROOT / name), "rb") as handle:
            print(
                f"{name}: ch={handle.getnchannels()} rate={handle.getframerate()} "
                f"dur={handle.getnframes() / handle.getframerate():.3f}s"
            )


if __name__ == "__main__":
    main()
