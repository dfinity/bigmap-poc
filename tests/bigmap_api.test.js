const bigmap_fn = require('../cli/bigmap_functions');
const bigmap = require('../cli/bigmap_cli');

// These tests require more time than the default 10 seconds
jest.setTimeout(30 * 1000);

describe("BigMap", () => {
  beforeAll(async () => {
    bigmap_fn.bigMapInitWithSearch();
  });


  test("put: should return the number of stored bytes", async () => {
    const key = "foo";
    const val = "bar";

    const res = await bigmap.put(key, val);

    expect(res.toNumber()).toEqual(3);
  });

  test("get: should return a stored value", async () => {
    const key = "bar";
    const val = "baz";

    await bigmap.put(key, val);
    const res = await bigmap.get(key);

    expect(bigmap.arrToStr(res)).toBe(val);
  });

  test("append: should append bytes", async () => {
    const key = "test_append";
    const val = "baz";

    await bigmap.put(key, val); // overwrite the old value if there was one

    const results = await Promise.all([
      await bigmap.append(key, val),
      await bigmap.append(key, val)
    ])

    console.log(results);
    expect(Math.max(...results)).toEqual(9);
  });

  test("delete: should delete entry", async () => {
    const key = "foo";
    const val = "bar";

    await bigmap.put(key, val);
    const res = await bigmap.deleteKey(key);

    expect(res.toNumber()).toEqual(3);
  });
});
