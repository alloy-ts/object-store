import { expect, test } from "vite-plus/test";
import { parseUrlScheme, ObjectStoreScheme } from "../index.js";

test("parseUrlScheme - local file URLs classify as Local with a filesystem path", () => {
  const res = parseUrlScheme("file:///tmp/foo/bar.txt");
  expect(res.scheme).toBe(ObjectStoreScheme.Local);
  expect(res.path).toBe("tmp/foo/bar.txt");
});

test("parseUrlScheme - memory URLs classify as Memory with an empty path", () => {
  const res = parseUrlScheme("memory://");
  expect(res.scheme).toBe(ObjectStoreScheme.Memory);
  expect(res.path).toBe("");
});

test("parseUrlScheme - s3 URLs classify as AmazonS3 (bucket is the host, key is the path)", () => {
  const res = parseUrlScheme("s3://my-bucket/my/key");
  expect(res.scheme).toBe(ObjectStoreScheme.AmazonS3);
  expect(res.path).toBe("my/key");
});

test("parseUrlScheme - gs URLs classify as GoogleCloudStorage", () => {
  const res = parseUrlScheme("gs://my-bucket/my/key");
  expect(res.scheme).toBe(ObjectStoreScheme.GoogleCloudStorage);
  expect(res.path).toBe("my/key");
});

test("parseUrlScheme - az URLs classify as MicrosoftAzure", () => {
  const res = parseUrlScheme("az://my-container/my/blob");
  expect(res.scheme).toBe(ObjectStoreScheme.MicrosoftAzure);
  expect(res.path).toBe("my/blob");
});

test("parseUrlScheme - http and https URLs classify as Http", () => {
  expect(parseUrlScheme("http://example.com/path").scheme).toBe(
    ObjectStoreScheme.Http,
  );
  expect(parseUrlScheme("https://example.com/path").scheme).toBe(
    ObjectStoreScheme.Http,
  );
  expect(parseUrlScheme("http://example.com/path").path).toBe("path");
});

test("parseUrlScheme - unsupported scheme throws", () => {
  expect(() => parseUrlScheme("ftp://example.com/file")).toThrow();
});
