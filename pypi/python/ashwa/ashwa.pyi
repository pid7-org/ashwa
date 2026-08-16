"""
Type stubs for the ``ashwa`` native extension module
"""

from typing import Optional

__version__: str

def search_one(haystack: bytes | bytearray | memoryview, needle: int) -> Optional[int]:
    """
    Search for the first occurrence of ``needle`` in ``haystack``

    Uses the best available SIMD instruction set on the host CPU
    (AVX-512BW, AVX2, SSE2, ARM NEON) otherwise falls back to SWAR.

    Args:
        haystack: A bytes-like object to search in.
        needle:   The target byte value (0–255) to locate.

    Returns:
        The 0-based byte index of the first occurrence of *needle*,
        or ``None`` if not found.

    Examples:
        >>> import ashwa
        >>> ashwa.search_one(b"Hello, World!", ord("W"))
        7
        >>> ashwa.search_one(b"Hello, World!", ord("z")) is None
        True
        >>> ashwa.search_one(b"", ord("a")) is None
        True
    """
    ...
