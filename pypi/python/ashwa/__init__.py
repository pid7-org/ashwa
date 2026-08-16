"""
Hardware-accelerated routines for single substring search

Example::

    import ashwa

    haystack = b"The quick brown fox jumps over the lazy dog"
    print(ashwa.search_one(haystack, ord("f")))  # 16
    print(ashwa.search_one(haystack, ord("!")))  # None
"""

from .ashwa import search_one, __version__

__all__ = ["search_one", "__version__"]
