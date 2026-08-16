import re
import pytest
import ashwa

def _standard_suite(search_fn):
    # Empty haystack
    assert search_fn(b"", ord("a")) is None

    # Pangram spot-checks
    haystack = b"the quick brown fox jumps over the lazy dog"
    assert search_fn(haystack, ord("Z")) is None
    assert search_fn(haystack, ord("!")) is None
    assert search_fn(haystack, ord("o")) == 0x0C

    # Short haystacks: needle at last byte, lengths 1–7
    for length in range(1, 8):
        h = bytearray(b"x" * length)
        h[length - 1] = ord("a")
        assert search_fn(bytes(h), ord("a")) == length - 1

    # Boundary at exact chunk edges: 8, 16, 24, 32 bytes
    for size, marker in [(8, "A"), (0x10, "B"), (0x18, "C"), (0x20, "D")]:
        h = bytearray(b"-" * size)
        h[size - 1] = ord(marker)
        assert search_fn(bytes(h), ord(marker)) == size - 1

    # Sweep: needle at every position in a 0x200-byte buffer
    haystack = bytearray(b"-" * 0x200)
    for i in range(len(haystack)):
        haystack[i] = ord("A")
        result = search_fn(bytes(haystack), ord("A"))
        assert result == i, f"sweep failed at index {i}"
        haystack[i] = ord("-")

    # High-byte values (0x80, 0xFF) — catches SWAR sign-extension bugs
    h = bytearray([0x80] * 0x40)
    h[0x3F] = 0xFF
    assert search_fn(bytes(h), 0x7F) is None
    assert search_fn(bytes(h), 0xFF) == 0x3F

    # Null byte in haystack
    h = bytearray([0xFF] * 0x50)
    h[0x2A] = 0x00
    assert search_fn(bytes(h), 0x00) == 0x2A

    # All-same-byte haystacks — first byte must be returned
    assert search_fn(b"\x01" * 8, 0x01) == 0
    assert search_fn(b"\x80" * 8, 0x80) == 0

    # Unaligned-start sweep: slice a 0x60-byte buffer at offsets 1–7
    base = bytearray(b"-" * 0x60)
    for offset in range(1, 8):
        h = bytearray(base[offset:])
        h[0x19] = ord("Z")
        assert search_fn(bytes(h), ord("Z")) == 0x19

        end = len(h) - 2
        h[end] = ord("Y")
        assert search_fn(bytes(h), ord("Y")) == end

    # Tail-length coverage: sizes that stress the scalar tail loop
    for length in [0x09, 0x0F, 0x11, 0x1F, 0x21, 0x3F, 0x41, 0x7F, 0x81, 0xFF, 0x101]:
        h = bytearray(b"-" * length)
        h[length - 1] = ord("A")
        result = search_fn(bytes(h), ord("A"))
        assert result == length - 1, f"tail fallback failed for length {length:#x}"

    # Lane-stride pattern: needle at every 16th byte in a 256-byte buffer
    h = bytearray(b"-" * 0x100)
    for i in range(0, 0x100, 16):
        h[i] = ord("*")
    assert search_fn(bytes(h), ord("*")) == 0
    h[0] = ord("-")
    assert search_fn(bytes(h), ord("*")) == 0x10

    # Large haystack
    huge = bytearray(b"x" * (0x64 * 0x400))
    assert search_fn(bytes(huge), ord("Z")) is None
    huge[-1] = ord("Z")
    assert search_fn(bytes(huge), ord("Z")) == 0x64 * 0x400 - 1


class TestInputTypes:
    def test_bytes(self):
        assert ashwa.search_one(b"hello world", ord("w")) == 6

    def test_bytearray(self):
        assert ashwa.search_one(bytearray(b"hello world"), ord("w")) == 6

    def test_memoryview(self):
        assert ashwa.search_one(memoryview(b"hello world"), ord("w")) == 6

    def test_memoryview_of_bytearray(self):
        assert ashwa.search_one(memoryview(bytearray(b"hello world")), ord("w")) == 6

    def test_memoryview_slice(self):
        """A sliced memoryview is still contiguous and must work."""
        data = memoryview(b"xxhello worldxx")
        sliced = data[2:13]  # "hello world"
        assert ashwa.search_one(sliced, ord("w")) == 6

    def test_full_suite_on_bytearray(self):
        """Run the full correctness suite feeding bytearray directly."""
        def search(h, n):
            return ashwa.search_one(bytearray(h), n)
        _standard_suite(search)

    def test_full_suite_on_memoryview(self):
        """Run the full correctness suite feeding memoryview directly."""
        def search(h, n):
            return ashwa.search_one(memoryview(h), n)
        _standard_suite(search)

class TestReturnType:
    def test_found_returns_int(self):
        result = ashwa.search_one(b"abc", ord("b"))
        assert result == 1
        assert type(result) is int

    def test_not_found_returns_none(self):
        result = ashwa.search_one(b"abc", ord("z"))
        assert result is None

    def test_found_at_zero(self):
        assert ashwa.search_one(b"abc", ord("a")) == 0

    def test_found_at_last(self):
        data = b"abc"
        assert ashwa.search_one(data, ord("c")) == len(data) - 1


class TestNeedleBoundaries:
    def test_needle_zero(self):
        assert ashwa.search_one(b"\x00", 0) == 0
        assert ashwa.search_one(b"\xff\x00", 0) == 1
        assert ashwa.search_one(b"\xff", 0) is None

    def test_needle_255(self):
        assert ashwa.search_one(b"\xff", 255) == 0
        assert ashwa.search_one(b"\x00\xff", 255) == 1
        assert ashwa.search_one(b"\x00", 255) is None

    def test_all_needle_values_single_byte_haystack(self):
        for v in range(256):
            assert ashwa.search_one(bytes([v]), v) == 0

    def test_all_needle_values_not_found(self):
        for v in range(256):
            other = v ^ 0xFF  # guaranteed different byte
            assert ashwa.search_one(bytes([other]), v) is None

class TestInvalidInputs:
    @pytest.mark.parametrize("bad_haystack", [
        "hello",        # str
        123,            # int
        3.14,           # float
        ["h", "i"],     # list
        None,
    ])
    def test_invalid_haystack_type(self, bad_haystack):
        with pytest.raises(TypeError):
            ashwa.search_one(bad_haystack, ord("h"))

    @pytest.mark.parametrize("bad_needle", [
        "a",            # str instead of int
        b"a",           # bytes instead of int
        3.14,           # float
        None,
        [65],           # list
    ])
    def test_invalid_needle_type(self, bad_needle):
        with pytest.raises(TypeError):
            ashwa.search_one(b"hello", bad_needle)

    def test_needle_out_of_range_negative(self):
        with pytest.raises((TypeError, OverflowError)):
            ashwa.search_one(b"hello", -1)

    def test_needle_out_of_range_high(self):
        with pytest.raises((TypeError, OverflowError)):
            ashwa.search_one(b"hello", 256)

class TestModule:
    def test_version_is_string(self):
        assert isinstance(ashwa.__version__, str)

    def test_version_not_empty(self):
        assert len(ashwa.__version__) > 0

    def test_version_semver_shape(self):
        assert re.fullmatch(r"\d+\.\d+\.\d+.*", ashwa.__version__), (
            f"unexpected version format: {ashwa.__version__!r}"
        )

    def test_search_one_is_callable(self):
        assert callable(ashwa.search_one)

def test_standard_suite_bytes():
    _standard_suite(ashwa.search_one)
