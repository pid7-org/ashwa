const isNode =
  typeof process !== "undefined" &&
  process.versions != null &&
  process.versions.node != null;

if (isNode) {
  const native = require("./native/index.js");

  module.exports = {
    isNative: true,
    init: async () => {},
    initSync: () => {},
    searchOne(haystack, needle) {
      const res = native.searchOne(haystack, needle);
      return res !== undefined && res !== null ? Number(res) : null;
    },
  };
} else {
  module.exports = require("./browser.js");
}
