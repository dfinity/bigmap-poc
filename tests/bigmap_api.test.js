
const bigmap = require('../cli/bigmap_cli');

// These tests require more time than the default 10 seconds
jest.setTimeout(30 * 1000);

describe("BigMap", () => {
  beforeAll(async () => {
    await bigmap.setDataBucketWasmBinary('target/wasm32-unknown-unknown/release/bigmap_data.wasm');
  });

  describe("put", () => {
    test("should return the number of stored bytes when saving a value", async () => {
      const key = "foo";
      const val = "bar";

      const res = await bigmap.put(key, val);

      expect(res.toNumber()).toBe(3);
    });
  });

  describe("get", () => {
    test("should return a stored value", async () => {
      const key = "bar";
      const val = "baz";

      await bigmap.put(key, val);
      const res = await bigmap.get(key);

      expect(bigmap.arrToStr(res)).toBe(val);
    });
  });
});
