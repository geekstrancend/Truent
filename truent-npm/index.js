"use strict";

const path = require("path");
const fs = require("fs").promises;
const { spawnSync } = require("child_process");
const { getBinaryPath, isBinaryInstalled } = require("./lib/binary-path");

/**
 * Run a Truent analysis programmatically.
 * 
 * @param {object} options Analysis options
 * @param {string} options.path - Path to analyze
 * @param {string} options.chain - "evm" | "solana" | "move"
 * @param {string} [options.failOn] - "low" | "medium" | "high" | "critical"
 * @param {string} [options.config] - Path to .truent.toml config
 * @param {boolean} [options.verbose] - Verbose output
 * @returns {Promise<object>} Parsed report object
 * @throws {Error} If analysis fails
 */
async function analyze(options) {
  const { path: targetPath, chain, failOn, config, verbose } = options;

  if (!targetPath) {
    throw new Error("analyze() requires options.path");
  }
  if (!chain) {
    throw new Error("analyze() requires options.chain");
  }

  if (!isBinaryInstalled()) {
    throw new Error(
      "Truent binary not installed. Run: npm install @truent/cli"
    );
  }

  const binaryPath = getBinaryPath();
  const tmpFile = path.join(__dirname, `.truent-report-${Date.now()}.json`);

  try {
    const args = ["check", targetPath, "--chain", chain, "--format", "json", "--output", tmpFile];

    if (failOn) {
      args.push("--fail-on", failOn);
    }
    if (config) {
      args.push("--config", config);
    }

    const result = spawnSync(binaryPath, args, {
      encoding: "utf8",
      stdio: verbose ? "inherit" : ["pipe", "pipe", "pipe"],
    });

    if (result.error) {
      throw new Error(`Failed to run Truent: ${result.error.message}`);
    }

    // Read the generated report
    const reportJson = await fs.readFile(tmpFile, "utf8");
    const report = JSON.parse(reportJson);

    return report;
  } finally {
    // Cleanup
    try {
      await fs.unlink(tmpFile);
    } catch (e) {
      // Ignore unlink errors
    }
  }
}

/**
 * Run truent doctor and return component health status.
 * 
 * @returns {Promise<object>} Doctor result object
 * @throws {Error} If doctor command fails
 */
async function doctor() {
  if (!isBinaryInstalled()) {
    throw new Error(
      "Truent binary not installed. Run: npm install @truent/cli"
    );
  }

  const binaryPath = getBinaryPath();
  const result = spawnSync(binaryPath, ["doctor", "--format", "json"], {
    encoding: "utf8",
  });

  if (result.error) {
    throw new Error(`Failed to run Truent doctor: ${result.error.message}`);
  }

  if (result.status !== 0) {
    throw new Error(`Truent doctor failed: ${result.stderr}`);
  }

  return JSON.parse(result.stdout);
}

/**
 * Initialize a .truent.toml in the given directory.
 * 
 * @param {string} directory - Directory to initialize
 * @returns {Promise<void>}
 * @throws {Error} If initialization fails
 */
async function init(directory) {
  if (!isBinaryInstalled()) {
    throw new Error(
      "Truent binary not installed. Run: npm install @truent/cli"
    );
  }

  const binaryPath = getBinaryPath();
  const result = spawnSync(binaryPath, ["init", directory], {
    encoding: "utf8",
  });

  if (result.error) {
    throw new Error(`Failed to run Truent init: ${result.error.message}`);
  }

  if (result.status !== 0) {
    throw new Error(`Truent init failed: ${result.stderr}`);
  }
}

/**
 * Get the version of the installed Truent binary.
 * 
 * @returns {Promise<string>} Version string (e.g. "0.1.3")
 * @throws {Error} If version check fails
 */
async function version() {
  if (!isBinaryInstalled()) {
    throw new Error(
      "Truent binary not installed. Run: npm install @truent/cli"
    );
  }

  const binaryPath = getBinaryPath();
  const result = spawnSync(binaryPath, ["--version"], {
    encoding: "utf8",
  });

  if (result.error) {
    throw new Error(`Failed to get Truent version: ${result.error.message}`);
  }

  const versionMatch = result.stdout.match(/\d+\.\d+\.\d+/);
  if (!versionMatch) {
    throw new Error(`Invalid version output: ${result.stdout}`);
  }

  return versionMatch[0];
}

/**
 * Check if the Truent binary is installed and working.
 * 
 * @returns {Promise<boolean>}
 */
async function isInstalled() {
  return isBinaryInstalled();
}

module.exports = { analyze, doctor, init, version, isInstalled };
