import { expect, test } from "vite-plus/test";
import { TagSet } from "../index.js";

test("TagSet - new and push", () => {
  const set = new TagSet();
  expect(set.isEmpty()).toBe(true);

  set.push("test/foo", "value sdlks");
  expect(set.isEmpty()).toBe(false);

  set.push("foo", " sdf _ /+./sd");
  expect(set.encoded()).toBe("test%2Ffoo=value+sdlks&foo=+sdf+_+%2F%2B.%2Fsd");
});
