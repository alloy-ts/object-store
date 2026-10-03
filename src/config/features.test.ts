import { describe, expect, test } from "vite-plus/test";
import { getFeatures } from "../main.ts";

describe("config features", () => {
  test("getFeatures returns feature flags state", () => {
    const features = getFeatures();
    expect(features).toBeDefined();
    expect(typeof features.fs).toBe("boolean");
    expect(typeof features.tokio).toBe("boolean");
    expect(typeof features.aws).toBe("boolean");
    expect(typeof features.azure).toBe("boolean");
    expect(typeof features.gcp).toBe("boolean");
    expect(typeof features.http).toBe("boolean");

    // By default, fs and tokio features are enabled
    expect(features.fs).toBe(true);
    expect(features.tokio).toBe(true);
  });
});
