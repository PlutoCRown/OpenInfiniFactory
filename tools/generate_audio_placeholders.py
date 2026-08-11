import math
import random
import wave
from pathlib import Path

RATE = 22050
ROOT = Path(__file__).resolve().parents[1] / "assets" / "audio"


def write_wav(name, seconds, sample):
    frames = bytearray()
    for index in range(int(RATE * seconds)):
        value = max(-1.0, min(1.0, sample(index / RATE, index)))
        frames.extend(int(value * 32767).to_bytes(2, "little", signed=True))
    with wave.open(str(ROOT / name), "wb") as output:
        output.setnchannels(1)
        output.setsampwidth(2)
        output.setframerate(RATE)
        output.writeframes(frames)


def tone(frequency, duration, decay=8.0):
    def sample(time, _index):
        envelope = max(0.0, 1.0 - time / duration) ** decay
        return math.sin(2.0 * math.pi * frequency * time) * envelope

    return sample


def two_tone(first, second, duration):
    def sample(time, _index):
        half = duration / 2.0
        frequency = first if time < half else second
        envelope = min(1.0, time * 30.0) * max(0.0, 1.0 - time / duration) ** 2
        return 0.45 * math.sin(2.0 * math.pi * frequency * time) * envelope

    return sample


def noise(duration, frequency, amount=0.35):
    random.seed(frequency)

    def sample(time, index):
        envelope = max(0.0, 1.0 - time / duration) ** 2
        hum = math.sin(2.0 * math.pi * frequency * time) * (1.0 - amount)
        return (hum + random.uniform(-amount, amount)) * envelope

    return sample


def main():
    ROOT.mkdir(parents=True, exist_ok=True)
    write_wav("music.wav", 4.0, two_tone(220.0, 277.18, 4.0))
    write_wav("ui_click.wav", 0.08, tone(1200.0, 0.08, 2.0))
    write_wav("block_place.wav", 0.18, tone(420.0, 0.18, 2.0))
    write_wav("block_break.wav", 0.22, noise(0.22, 180.0, 0.75))
    write_wav("machine_work.wav", 0.32, two_tone(180.0, 260.0, 0.32))
    write_wav("machine_motor.wav", 1.0, noise(1.0, 95.0, 0.12))
    write_wav("weld.wav", 0.26, noise(0.26, 900.0, 0.65))
    write_wav("drill_break.wav", 0.35, noise(0.35, 250.0, 0.8))
    write_wav("sci_fi.wav", 0.5, two_tone(420.0, 840.0, 0.5))
    write_wav("acceptance.wav", 0.6, two_tone(660.0, 990.0, 0.6))


if __name__ == "__main__":
    main()
