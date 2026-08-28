import re
import pytest
import ashwa

def _standard_suite(search_fn):
    assert search_fn(b"", ord("a")) is None

    haystack = b"the quick brown fox jumps over the lazy dog"
    assert search_fn(haystack, ord("Z")) is None
    assert search_fn(haystack, ord("!")) is None
    assert search_fn(haystack, ord("o")) == 0x0C

    for length in range(1, 8):
        h = bytearray(b"x" * length)
        h[length - 1] = ord("a")
        assert search_fn(bytes(h), ord("a")) == length - 1

    for size, marker in [(8, "A"), (0x10, "B"), (0x18, "C"), (0x20, "D")]:
        h = bytearray(b"-" * size)
        h[size - 1] = ord(marker)
        assert search_fn(bytes(h), ord(marker)) == size - 1

    haystack = bytearray(b"-" * 0x200)
    for i in range(len(haystack)):
        haystack[i] = ord("A")
        result = search_fn(bytes(haystack), ord("A"))
        assert result == i, f"sweep failed at index {i}"
        haystack[i] = ord("-")

    h = bytearray([0x80] * 0x40)
    h[0x3F] = 0xFF
    assert search_fn(bytes(h), 0x7F) is None
    assert search_fn(bytes(h), 0xFF) == 0x3F

    h = bytearray([0xFF] * 0x50)
    h[0x2A] = 0x00
    assert search_fn(bytes(h), 0x00) == 0x2A

    assert search_fn(b"\x01" * 8, 0x01) == 0
    assert search_fn(b"\x80" * 8, 0x80) == 0

    base = bytearray(b"-" * 0x60)
    for offset in range(1, 8):
        h = bytearray(base[offset:])
        h[0x19] = ord("Z")
        assert search_fn(bytes(h), ord("Z")) == 0x19

        end = len(h) - 2
        h[end] = ord("Y")
        assert search_fn(bytes(h), ord("Y")) == end

    for length in [0x09, 0x0F, 0x11, 0x1F, 0x21, 0x3F, 0x41, 0x7F, 0x81, 0xFF, 0x101]:
        h = bytearray(b"-" * length)
        h[length - 1] = ord("A")
        result = search_fn(bytes(h), ord("A"))
        assert result == length - 1, f"tail fallback failed for length {length:#x}"

    h = bytearray(b"-" * 0x100)
    for i in range(0, 0x100, 16):
        h[i] = ord("*")
    assert search_fn(bytes(h), ord("*")) == 0
    
    h[0] = ord("-")
    assert search_fn(bytes(h), ord("*")) == 0x10

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
        sliced = data[2:13]
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

    def test_search_two_is_callable(self):
        assert callable(ashwa.search_two)

    def test_search_one_in_all(self):
        assert "search_one" in ashwa.__all__

    def test_search_two_in_all(self):
        assert "search_two" in ashwa.__all__

def test_standard_suite_bytes():
    _standard_suite(ashwa.search_one)


def _standard_two_suite(search_fn):
    assert search_fn(b"", b"ab") is None

    assert search_fn(b"a", b"ab") is None
    assert search_fn(b"b", b"ab") is None

    assert search_fn(b"ab", b"ab") == 0
    assert search_fn(b"ac", b"ab") is None
    assert search_fn(b"ba", b"ab") is None

    haystack = b"the quick brown fox jumps over the lazy dog"
    assert search_fn(haystack, b"ZZ") is None
    assert search_fn(haystack, b"!!") is None
    assert search_fn(haystack, b"th") == 0x00
    assert search_fn(haystack, b"he") == 0x01
    assert search_fn(haystack, b"qu") == 0x04
    assert search_fn(haystack, b"ox") == 0x11
    assert search_fn(haystack, b"do") == 0x28
    assert search_fn(haystack, b"og") == 0x29

    for length in range(2, 9):
        for pos in range(length - 1):
            h = bytearray(b"x" * length)
            h[pos] = ord("a")
            h[pos + 1] = ord("b")
            assert (
                search_fn(bytes(h), b"ab") == pos
            ), f"failed finding needle at pos {pos} in len {length}"

    edge_cases = [
        (8, 6, b"AB"),
        (9, 7, b"AB"),
        (16, 14, b"CD"),
        (17, 15, b"CD"),
        (24, 22, b"EF"),
        (25, 23, b"EF"),
        (32, 30, b"GH"),
        (33, 31, b"GH"),
        (34, 32, b"GH"),
    ]
    for size, pos, needle in edge_cases:
        h = bytearray(b"-" * size)
        h[pos] = needle[0]
        h[pos + 1] = needle[1]
        assert search_fn(bytes(h), needle) == pos, f"boundary failed for size {size} at pos {pos}"

    cross_positions = [
        0x03, 0x04, 0x07, 0x08, 0x0B, 0x0C, 0x0F, 0x10, 0x13, 0x14, 0x17, 0x18, 0x1B, 0x1C,
        0x1F, 0x20, 0x27, 0x28, 0x3F, 0x40,
    ]
    cross_buf = bytearray(b"-" * 0x80)
    for pos in cross_positions:
        cross_buf[pos] = ord("Y")
        cross_buf[pos + 1] = ord("Z")
        assert (
            search_fn(bytes(cross_buf), b"YZ") == pos
        ), f"Failed straddling cross-word position {pos:#x}"

        cross_buf[pos] = ord("-")
        cross_buf[pos + 1] = ord("-")

    haystack = bytearray(b"-" * 0x200)
    for i in range(len(haystack) - 1):
        haystack[i] = ord("A")
        haystack[i + 1] = ord("B")
        result = search_fn(bytes(haystack), b"AB")
        assert result == i, f"sweep failed at index {i}"

        haystack[i] = ord("-")
        haystack[i + 1] = ord("-")

    haystack_first = bytearray(b"A" * 0x100)
    assert search_fn(bytes(haystack_first), b"AB") is None

    haystack_first[0x7A] = ord("B")
    assert search_fn(bytes(haystack_first), b"AB") == 0x79

    haystack_second = bytearray(b"B" * 0x100)
    assert search_fn(bytes(haystack_second), b"AB") is None

    haystack_second[0x40] = ord("A")
    assert search_fn(bytes(haystack_second), b"AB") == 0x40

    assert search_fn(b"aaaaaa", b"aa") == 0
    assert search_fn(b"baaaaa", b"aa") == 1
    assert search_fn(b"bbaaaa", b"aa") == 2
    assert search_fn(b"ababab", b"ab") == 0
    assert search_fn(b"bababa", b"ab") == 1

    h = bytearray([0x80] * 0x40)
    h[0x3E] = 0xFE
    h[0x3F] = 0xFF
    assert search_fn(bytes(h), b"\x7f\x80") is None
    assert search_fn(bytes(h), b"\xfe\xff") == 0x3E

    h = bytearray([0xFF] * 0x50)
    h[0x2A] = 0x00
    h[0x2B] = 0x00
    assert search_fn(bytes(h), b"\x00\x00") == 0x2A

    assert search_fn(b"\x01" * 9, b"\x01\x01") == 0
    assert search_fn(b"\x80" * 9, b"\x80\x80") == 0
    assert search_fn(b"\xFF" * 9, b"\xFF\xFF") == 0
    assert search_fn(b"\x00" * 9, b"\x00\x00") == 0

    base = bytearray(b"-" * 0x60)
    for offset in range(1, 8):
        h = bytearray(base[offset:])
        h[0x19] = ord("Y")
        h[0x1A] = ord("Z")
        assert search_fn(bytes(h), b"YZ") == 0x19

        end = len(h) - 2
        h[end] = ord("K")
        h[end + 1] = ord("L")
        assert search_fn(bytes(h), b"KL") == end

    tail_lengths = [
        0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0E, 0x0F, 0x10, 0x11, 0x12,
        0x1F, 0x20, 0x21, 0x22, 0x2F, 0x30, 0x31, 0x3F, 0x40, 0x41, 0x7F, 0x80, 0x81, 0xFF,
        0x100, 0x101,
    ]
    for length in tail_lengths:
        h = bytearray(b"-" * length)
        h[length - 2] = ord("A")
        h[length - 1] = ord("B")
        result = search_fn(bytes(h), b"AB")
        assert result == length - 2, f"tail fallback failed for length {length:#x}"

    h_multi = bytearray(b"-" * 0x100)
    h_multi[0x10] = ord("M")
    h_multi[0x11] = ord("N")
    h_multi[0x50] = ord("M")
    h_multi[0x51] = ord("N")
    assert search_fn(bytes(h_multi), b"MN") == 0x10

    huge = bytearray(b"x" * (0x64 * 0x400))
    assert search_fn(bytes(huge), b"YZ") is None
    huge[-2] = ord("Y")
    huge[-1] = ord("Z")
    assert search_fn(bytes(huge), b"YZ") == 0x64 * 0x400 - 2

class TestSearchTwoInputTypes:
    def test_bytes(self):
        assert ashwa.search_two(b"hello world", b"wo") == 6

    def test_bytearray(self):
        assert ashwa.search_two(bytearray(b"hello world"), b"wo") == 6

    def test_memoryview(self):
        assert ashwa.search_two(memoryview(b"hello world"), b"wo") == 6

    def test_memoryview_of_bytearray(self):
        assert ashwa.search_two(memoryview(bytearray(b"hello world")), b"wo") == 6

    def test_memoryview_slice(self):
        data = memoryview(b"xxhello worldxx")
        sliced = data[2:13]  # "hello world"
        assert ashwa.search_two(sliced, b"wo") == 6

    def test_needle_bytes(self):
        assert ashwa.search_two(b"hello world", b"el") == 1

    def test_needle_bytearray(self):
        assert ashwa.search_two(b"hello world", bytearray(b"el")) == 1

    def test_needle_memoryview(self):
        assert ashwa.search_two(b"hello world", memoryview(b"el")) == 1

    def test_needle_tuple(self):
        assert ashwa.search_two(b"hello world", (ord("e"), ord("l"))) == 1

    def test_needle_list(self):
        assert ashwa.search_two(b"hello world", [ord("e"), ord("l")]) == 1

    def test_full_suite_on_bytearray(self):
        def search(h, n):
            return ashwa.search_two(bytearray(h), n)
        _standard_two_suite(search)

    def test_full_suite_on_memoryview(self):
        def search(h, n):
            return ashwa.search_two(memoryview(h), n)
        _standard_two_suite(search)

    def test_full_suite_with_tuple_needle(self):
        def search(h, n):
            return ashwa.search_two(h, (n[0], n[1]))
        _standard_two_suite(search)

    def test_full_suite_with_list_needle(self):
        def search(h, n):
            return ashwa.search_two(h, [n[0], n[1]])
        _standard_two_suite(search)

class TestSearchTwoReturnType:
    def test_found_returns_int(self):
        result = ashwa.search_two(b"abcd", b"bc")
        assert result == 1
        assert type(result) is int

    def test_not_found_returns_none(self):
        result = ashwa.search_two(b"abcd", b"zz")
        assert result is None

    def test_found_at_zero(self):
        assert ashwa.search_two(b"abcd", b"ab") == 0

    def test_found_at_last(self):
        data = b"abcdef"
        assert ashwa.search_two(data, b"ef") == len(data) - 2

class TestSearchTwoNeedleBoundaries:
    def test_needle_zero(self):
        assert ashwa.search_two(b"\x00\x00", b"\x00\x00") == 0
        assert ashwa.search_two(b"\xff\x00\x00", b"\x00\x00") == 1
        assert ashwa.search_two(b"\xff\x00", b"\x00\x00") is None

    def test_needle_255(self):
        assert ashwa.search_two(b"\xff\xff", b"\xff\xff") == 0
        assert ashwa.search_two(b"\x00\xff\xff", b"\xff\xff") == 1
        assert ashwa.search_two(b"\x00\xff", b"\xff\xff") is None

    def test_all_needle_boundary_combinations(self):
        patterns = [(0, 0), (0, 255), (255, 0), (255, 255)]
        for needle_vals in patterns:
            needle = bytes(needle_vals)
            haystack = b"\xaa" + needle + b"\xbb"
            assert ashwa.search_two(haystack, needle) == 1
            other_needle = bytes([needle_vals[0] ^ 0xFF, needle_vals[1] ^ 0xFF])
            assert ashwa.search_two(haystack, other_needle) is None

class TestSearchTwoInvalidInputs:
    @pytest.mark.parametrize("bad_haystack", [
        "hello",        # str
        123,            # int
        3.14,           # float
        ["h", "i"],     # list
        None,
    ])
    def test_invalid_haystack_type(self, bad_haystack):
        with pytest.raises(TypeError):
            ashwa.search_two(bad_haystack, b"he")

    @pytest.mark.parametrize("bad_needle", [
        "ab",           # str instead of bytes/sequence of int
        123,            # int instead of sequence
        3.14,           # float
        None,
        [65, "b"],      # list with non-int element
    ])
    def test_invalid_needle_type(self, bad_needle):
        with pytest.raises(TypeError):
            ashwa.search_two(b"hello", bad_needle)

    @pytest.mark.parametrize("wrong_length_needle", [
        b"",
        b"a",
        b"abc",
        b"abcd",
        (),
        (ord("a"),),
        (ord("a"), ord("b"), ord("c")),
        [],
        [ord("a")],
        [ord("a"), ord("b"), ord("c")],
    ])
    def test_invalid_needle_length(self, wrong_length_needle):
        with pytest.raises(ValueError):
            ashwa.search_two(b"hello", wrong_length_needle)

    def test_needle_out_of_range_negative(self):
        with pytest.raises((TypeError, OverflowError)):
            ashwa.search_two(b"hello", [-1, ord("h")])

    def test_needle_out_of_range_high(self):
        with pytest.raises((TypeError, OverflowError)):
            ashwa.search_two(b"hello", [256, ord("h")])

def test_standard_two_suite_bytes():
    _standard_two_suite(ashwa.search_two)

