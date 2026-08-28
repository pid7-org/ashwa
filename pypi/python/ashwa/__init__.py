"""
Hardware-accelerated routines for single substring search

Examples::

    import ashwa

    haystack = b"The quick brown fox jumps over the lazy dog"

    # Search for a single byte
    print(ashwa.search_one(haystack, ord("f")))  # 16
    print(ashwa.search_one(haystack, ord("!")))  # None

    # Search for a two-byte sequence
    print(ashwa.search_two(haystack, b"qu"))     # 4
    print(ashwa.search_two(haystack, b"ox"))     # 17
    print(ashwa.search_two(haystack, b"!!"))     # None
"""

from .ashwa import search_one, search_two, __version__

__all__ = ["search_one", "search_two", "__version__"]
