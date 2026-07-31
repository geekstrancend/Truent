"use strict";

const fs = require("fs");
const path = require("path");
const { BINARY_DIR } = require("./detect-platform");

/**
 * Get the path to the Truent binary.
 * 
 * Searches in order:
 * 1. TRUENT_BINARY_PATH environment variable
 * 2. .truent-bin/truent in package directory
 * 3. truent on PATH (let OS find it)
 * 
 * @returns {string} Path to binary
 */
function getBinaryPath() {
  // Check environment variable override
  if (process.env.TRUENT_BINARY_PATH) {
    const overridePath = process.env.TRUENT_BINARY_PATH;
    if (!fs.existsSync(overridePath)) {
      throw new Error(
        `Truent binary not found at TRUENT_BINARY_PATH (${overridePath}).\n` +
        `Run: npm install @truent/cli`
      );
    }
    return overridePath;
  }

  // Check installed binary in package — this is the most common case
  const binaryName = process.platform === "win32" ? "truent.exe" : "truent";
  const packageBinary = path.join(BINARY_DIR, binaryName);

  if (fs.existsSync(packageBinary)) {
    return packageBinary;
  }

  // Fall back to PATH — let the OS find it
  // Do NOT call spawnSync("which", ...) as it causes infinite recursion
  // when 'truent' on PATH is the Node wrapper we're in
  return binaryName;
}

/**
 * Check if the Truent binary is installed and executable.
 * 
 * @returns {boolean}
 */
function isBinaryInstalled() {
  try {
    const binaryPath = getBinaryPath();
    return fs.existsSync(binaryPath) && isExecutable(binaryPath);
  } catch (e) {
    return false;
  }
}

/**
 * Check if a file is executable.
 * 
 * @private
 * @param {string} filepath - Path to file
 * @returns {boolean}
 */
function isExecutable(filepath) {
  try {
    // On Unix, check execute bit
    if (process.platform !== "win32") {
      const stats = fs.statSync(filepath);
      return (stats.mode & 0o111) !== 0;
    }
    // On Windows, executable files are .exe
    return filepath.endsWith(".exe");
  } catch (e) {
    return false;
  }
}

module.exports = { getBinaryPath, isBinaryInstalled };
