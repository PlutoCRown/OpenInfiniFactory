#!/usr/bin/env node

const fs = require("fs");
const path = require("path");
const { execFileSync } = require("child_process");

const ROOT = process.argv[2] ? path.resolve(process.argv[2]) : process.cwd();
const LOG_PATH = process.argv[3]
  ? path.resolve(process.argv[3])
  : path.join(ROOT, "logs", "rust-lines.json");

function git(args) {
  return execFileSync("git", args, {
    cwd: ROOT,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  }).trim();
}

function gitLines(args) {
  const output = git(args);
  return output ? output.split("\n").filter(Boolean) : [];
}

function countLines(contents) {
  if (!contents) {
    return 0;
  }
  return contents.endsWith("\n")
    ? contents.split("\n").length - 1
    : contents.split("\n").length;
}

function isSrcRustFile(file) {
  return file.startsWith("src/") && file.endsWith(".rs");
}

function trackedRustFiles() {
  return gitLines(["ls-files", "*.rs"]).filter(isSrcRustFile);
}

function untrackedRustFiles() {
  return gitLines(["ls-files", "--others", "--exclude-standard", "*.rs"]).filter(
    isSrcRustFile,
  );
}

function headRustLines(files) {
  let total = 0;
  for (const file of files) {
    try {
      total += countLines(git(["show", `HEAD:${file}`]));
    } catch {
      // A file listed by git can still be absent from HEAD during edge cases.
    }
  }
  return total;
}

function workspaceRustLines(files) {
  return files.reduce((total, file) => {
    const fullPath = path.join(ROOT, file);
    if (!fs.existsSync(fullPath)) {
      return total;
    }
    return total + countLines(fs.readFileSync(fullPath, "utf8"));
  }, 0);
}

function loadLog() {
  if (!fs.existsSync(LOG_PATH)) {
    return {};
  }
  return JSON.parse(fs.readFileSync(LOG_PATH, "utf8"));
}

function writeLog(log) {
  fs.mkdirSync(path.dirname(LOG_PATH), { recursive: true });
  fs.writeFileSync(LOG_PATH, `${JSON.stringify(log, null, 2)}\n`);
}

function signed(value) {
  return value > 0 ? `+${value}` : String(value);
}

const commit = git(["rev-parse", "HEAD"]);
const tracked = trackedRustFiles();
const untracked = untrackedRustFiles();
const headLines = headRustLines(tracked);
const workspaceLines = workspaceRustLines([...tracked, ...untracked]);
const delta = workspaceLines - headLines;

const log = loadLog();
log[commit] = headLines;
writeLog(log);

console.log(`commit ${commit}`);
console.log(`已提交的代码行数： ${headLines}`);
console.log(`当前代码行数： ${workspaceLines}`);
console.log(`差分： ${signed(delta)}`);