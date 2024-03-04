module.exports = {
  extends: ['@commitlint/config-conventional'],
  rules: {
    "body-max-line-length": [1, "always", 120],
    "subject-case": [2, "always", "sentence-case"],
  }
};
