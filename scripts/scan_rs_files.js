#!/usr/bin/env node

const fs = require("fs");
const path = require("path");

const ROOT = process.argv[2] ? path.resolve(process.argv[2]) : process.cwd();
const DEFAULT_LIMIT = 20;
const limit = Number(process.argv[3] ?? DEFAULT_LIMIT);

function walk(dir, files = []) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    if (entry.name === "target" || entry.name === ".git") {
      continue;
    }

    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      walk(fullPath, files);
    } else if (entry.isFile() && entry.name.endsWith(".rs")) {
      files.push(fullPath);
    }
  }
  return files;
}

function countLines(contents) {
  if (contents.length === 0) {
    return 0;
  }
  return contents.endsWith("\n")
    ? contents.split("\n").length - 1
    : contents.split("\n").length;
}

function functionStats(contents) {
  const lines = contents.split("\n");
  const starts = [];
  for (let index = 0; index < lines.length; index += 1) {
    if (/^\s*(pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+[A-Za-z0-9_]+\s*\(/.test(lines[index])) {
      starts.push(index);
    }
  }

  let longest = { start: 0, end: 0, lines: 0 };
  for (const start of starts) {
    const end = findBlockEnd(lines, start);
    const span = end - start + 1;
    if (span > longest.lines) {
      longest = { start: start + 1, end: end + 1, lines: span };
    }
  }

  return {
    functions: starts.length,
    longestFunctionLines: longest.lines,
    longestFunctionStart: longest.start,
    longestFunctionEnd: longest.end,
  };
}

function findBlockEnd(lines, start) {
  let depth = 0;
  let sawOpen = false;

  for (let index = start; index < lines.length; index += 1) {
    for (const char of stripLineComment(lines[index])) {
      if (char === "{") {
        depth += 1;
        sawOpen = true;
      } else if (char === "}") {
        depth -= 1;
        if (sawOpen && depth === 0) {
          return index;
        }
      }
    }
  }

  return start;
}

function stripLineComment(line) {
  const commentIndex = line.indexOf("//");
  return commentIndex === -1 ? line : line.slice(0, commentIndex);
}

const rows = walk(ROOT)
  .map((file) => {
    const contents = fs.readFileSync(file, "utf8");
    return {
      file: path.relative(ROOT, file),
      lines: countLines(contents),
      ...functionStats(contents),
    };
  })
  .sort((a, b) => b.lines - a.lines);

const shown = rows.slice(0, Number.isFinite(limit) ? limit : DEFAULT_LIMIT);
const width = Math.max(...shown.map((row) => row.file.length), "file".length);

console.log(`Scanned ${rows.length} Rust files under ${ROOT}`);
console.log(
  `${"lines".padStart(6)}  ${"fns".padStart(3)}  ${"max_fn".padStart(6)}  ${"file".padEnd(width)}  longest_fn`
);
for (const row of shown) {
  const longest =
    row.longestFunctionLines === 0
      ? "-"
      : `${row.longestFunctionStart}-${row.longestFunctionEnd}`;
  console.log(
    `${String(row.lines).padStart(6)}  ${String(row.functions).padStart(3)}  ${String(row.longestFunctionLines).padStart(6)}  ${row.file.padEnd(width)}  ${longest}`
  );
}
