"""生成焊接金属撞击与活塞液压动作音效。"""

from pathlib import Path
import wave

import numpy as np


SAMPLE_RATE = 48_000
PEAK_DB = -1.0
WELD_DURATION = 0.34
PUSHER_DURATION = 0.5
RANDOM_SEED = 20_260_813
OUTPUT_DIR = Path(__file__).resolve().parents[2] / "assets" / "audio"


def fade_edges(samples: np.ndarray, attack_seconds: float, release_seconds: float) -> np.ndarray:
    """只平滑文件边缘，避免一次性音效产生数字点击。"""
    result = samples.copy()
    attack = max(1, round(attack_seconds * SAMPLE_RATE))
    release = max(1, round(release_seconds * SAMPLE_RATE))
    result[:attack] *= np.sin(np.linspace(0.0, np.pi / 2.0, attack)) ** 2
    result[-release:] *= np.cos(np.linspace(0.0, np.pi / 2.0, release)) ** 2
    return result


def normalize(samples: np.ndarray) -> np.ndarray:
    """移除直流并归一到统一峰值。"""
    samples = samples - np.mean(samples)
    peak = np.max(np.abs(samples))
    if peak == 0.0:
        raise ValueError("audio signal is silent")
    return samples * (10.0 ** (PEAK_DB / 20.0) / peak)


def hydraulic_motion(start_frequency: float, end_frequency: float, seed: int) -> np.ndarray:
    """用线性扫频和窄带气流噪声合成一个完整液压缸运动周期。"""
    count = round(PUSHER_DURATION * SAMPLE_RATE)
    time = np.arange(count) / SAMPLE_RATE
    sweep = (end_frequency - start_frequency) / PUSHER_DURATION
    phase = 2.0 * np.pi * (start_frequency * time + 0.5 * sweep * time**2)
    pressure = 0.52 * np.sin(phase) + 0.18 * np.sin(2.03 * phase + 0.35)

    rng = np.random.default_rng(seed)
    noise = rng.normal(0.0, 1.0, count)
    low = np.convolve(noise, np.ones(43) / 43.0, mode="same")
    airflow = noise - np.convolve(noise, np.ones(7) / 7.0, mode="same")
    signal = pressure + 0.14 * low + 0.055 * airflow

    # 压力随行程略增强；终点仅留 1.5 ms 防点击淡出，听感上会戛然而止。
    travel_pressure = 0.72 + 0.28 * (time / PUSHER_DURATION)
    signal *= travel_pressure
    return normalize(fade_edges(signal, 0.006, 0.0015))


def write_wav(name: str, samples: np.ndarray) -> None:
    """写入单声道 16-bit PCM WAV。"""
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    pcm = np.round(np.clip(samples, -1.0, 1.0) * 32_767.0).astype("<i2")
    with wave.open(str(OUTPUT_DIR / name), "wb") as output:
        output.setnchannels(1)
        output.setsampwidth(2)
        output.setframerate(SAMPLE_RATE)
        output.writeframes(pcm.tobytes())


def validate_wav(name: str, expected_duration: float) -> None:
    """检查运行时需要的 WAV 格式、时长、峰值和直流偏移。"""
    path = OUTPUT_DIR / name
    with wave.open(str(path), "rb") as source:
        channels = source.getnchannels()
        width = source.getsampwidth()
        rate = source.getframerate()
        frames = source.getnframes()
        samples = np.frombuffer(source.readframes(frames), dtype="<i2").astype(np.float64) / 32_767.0
    duration = frames / rate
    peak = np.max(np.abs(samples))
    dc_offset = abs(np.mean(samples))
    if channels != 1 or width != 2 or rate != SAMPLE_RATE:
        raise ValueError(f"{name}: unexpected WAV format")
    if abs(duration - expected_duration) > 1.0 / SAMPLE_RATE:
        raise ValueError(f"{name}: unexpected duration {duration:.6f}s")
    if not np.all(np.isfinite(samples)) or peak > 0.9 or dc_offset > 0.001:
        raise ValueError(f"{name}: invalid samples peak={peak:.4f} dc={dc_offset:.6f}")
    print(
        f"{name}: mono 16-bit {rate} Hz, {duration:.3f}s, "
        f"peak={20.0 * np.log10(peak):.2f} dBFS, dc={dc_offset:.6f}"
    )


def main() -> None:
    """生成固定文件集合并立即验证。"""
    count = round(WELD_DURATION * SAMPLE_RATE)
    time = np.arange(count) / SAMPLE_RATE
    weld = np.zeros(count)
    modes = (
        (286.0, 0.22, 15.0),
        (731.0, 0.38, 20.0),
        (1_147.0, 0.70, 25.0),
        (1_823.0, 0.56, 32.0),
        (2_957.0, 0.36, 42.0),
        (4_613.0, 0.20, 58.0),
        (7_420.0, 0.10, 78.0),
    )
    for frequency, gain, decay in modes:
        weld += gain * np.sin(2.0 * np.pi * frequency * time) * np.exp(-decay * time)
    rng = np.random.default_rng(RANDOM_SEED)
    noise = rng.normal(0.0, 1.0, count)
    smooth = np.convolve(noise, np.ones(31) / 31.0, mode="same")
    weld += 0.34 * (noise - smooth) * np.exp(-105.0 * time)
    # 很短的二次接触让撞击更像实体金属，而不是单纯电子铃声。
    second_hit = round(0.023 * SAMPLE_RATE)
    weld[second_hit:] += 0.18 * weld[: count - second_hit] * np.exp(
        -18.0 * time[: count - second_hit]
    )
    weld = normalize(fade_edges(weld, 0.0008, 0.012))

    outputs = (
        ("weld.wav", weld, WELD_DURATION),
        (
            "pusher_extend.wav",
            hydraulic_motion(165.0, 510.0, RANDOM_SEED + 1),
            PUSHER_DURATION,
        ),
        (
            "pusher_retract.wav",
            hydraulic_motion(510.0, 165.0, RANDOM_SEED + 2),
            PUSHER_DURATION,
        ),
    )
    for name, samples, duration in outputs:
        write_wav(name, samples)
        validate_wav(name, duration)


if __name__ == "__main__":
    main()
