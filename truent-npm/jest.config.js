module.exports = {
  testEnvironment: "node",
  testMatch: ["**/__tests__/**/*.test.js"],
  collectCoverageFrom: [
    "lib/**/*.js",
    "index.js",
    "bin/truent.js",
    "!**/*.test.js",
  ],
  coverageDirectory: "coverage",
  coverageReporters: ["text", "lcov"],
};
