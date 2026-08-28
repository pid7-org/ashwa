"""
Type stubs for the ``ashwa`` native extension module
"""

from typing import Optional, Sequence

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

def search_two(
    haystack: bytes | bytearray | memoryview,
    needle: bytes | bytearray | memoryview | tuple[int, int] | list[int] | Sequence[int],
) -> Optional[int]:
    """
    Search for the first occurrence of a two-byte ``needle`` in ``haystack``

    Uses the best available SIMD instruction set on the host CPU
    (AVX-512BW, AVX2, SSE4.2, SSSE3, SSE2, ARM NEON) otherwise falls back to SWAR.

    Args:
        haystack: A bytes-like object to search in.
        needle:   A 2-byte sequence (e.g. ``b"ab"``, ``(97, 98)``, ``[97, 98]``,
                  or a 2-byte ``bytearray``/``memoryview``) to locate.

    Returns:
        The 0-based byte index of the first occurrence of *needle*,
        or ``None`` if not found.

    Examples:
        >>> import ashwa
        >>> ashwa.search_two(b"Hello, World!", b"Wo")
        7
        >>> ashwa.search_two(b"Hello, World!", (ord("W"), ord("o")))
        7
        >>> ashwa.search_two(b"Hello, World!", b"zz") is None
        True
        >>> ashwa.search_two(b"H", b"He") is None
        True
        >>> ashwa.search_two(b"", b"ab") is None
        True
    """
    ...

