from collections.abc import Generator
from pathlib import Path


def read_iter(
    file: Path,
    *,
    chunk_size: int = 8_192,
) -> Generator[str]:
    with file.open() as f:
        while out := f.read(chunk_size):
            yield from out
