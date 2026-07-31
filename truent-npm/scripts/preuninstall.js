#!/usr/bin/env node
"use strict";

/**
 * Preuninstall script — clean up the downloaded binary directory.
 */

const fs = require("fs");
const { BINARY_DIR } = require("../lib/detect-platform");

try {
  if (fs.existsSync(BINARY_DIR)) {
    fs.rmSync(BINARY_DIR, { recursive: true, force: true });
    console.log("✓ Truent binary removed");
  }
} catch (e) {
  // Non-fatal — ignore cleanup errors
  if (process.env.DEBUG) {
    console.warn("Warning: Failed to clean up Truent binary directory:", e.message);
  }
}
