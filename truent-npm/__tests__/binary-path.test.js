const path = require("path");
const fs = require("fs");
const { getBinaryPath, isBinaryInstalled } = require("../lib/binary-path");

describe("Binary Path Resolution", () => {
  const originalEnv = process.env;

  beforeEach(() => {
    process.env = { ...originalEnv };
    delete process.env.TRUENT_BINARY_PATH;
  });

  afterEach(() => {
    process.env = originalEnv;
  });

  test("TRUENT_BINARY_PATH environment variable overrides default", () => {
    // getBinaryPath() validates the override path exists (see "throws
    // helpful error" below), so this must point at a real file - this test
    // file itself is a convenient, always-present one - to isolate what's
    // actually being tested here: that the env var takes precedence over the
    // package-directory/PATH lookups, not path validation itself.
    process.env.TRUENT_BINARY_PATH = __filename;

    const binaryPath = getBinaryPath();

    expect(binaryPath).toBe(__filename);
  });

  test("isBinaryInstalled returns false when binary missing", () => {
    process.env.TRUENT_BINARY_PATH = "/nonexistent/path/to/truent";

    const installed = isBinaryInstalled();

    expect(installed).toBe(false);
  });

  test("isBinaryInstalled returns true when binary exists and is executable", () => {
    // Skip this test on systems without a truent in PATH
    const which = require("child_process").spawnSync("which", ["node"]);
    if (which.status === 0) {
      process.env.TRUENT_BINARY_PATH = which.stdout.toString().trim();
      const installed = isBinaryInstalled();
      expect(installed).toBe(true);
    }
  });

  test("getBinaryPath throws helpful error when not installed", () => {
    process.env.TRUENT_BINARY_PATH = "/nonexistent/truent";

    expect(() => {
      getBinaryPath();
    }).toThrow(/Truent binary not found/);

    expect(() => {
      getBinaryPath();
    }).toThrow(/npm install @truent\/cli/);
  });
});
