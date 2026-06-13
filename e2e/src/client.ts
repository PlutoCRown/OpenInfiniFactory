import { join } from "path";

const DEFAULT_PORT = 9876;

export class DebugClient {
  constructor(private readonly port: number = DEFAULT_PORT) { }

  private url(path: string): string {
    return `http://127.0.0.1:${this.port}${path}`;
  }

  async get(path: string): Promise<any> {
    const response = await fetch(this.url(path));
    return response.json();
  }

  async post(path: string): Promise<any> {
    const response = await fetch(this.url(path), { method: "POST" });
    return response.json();
  }
}

export async function waitForServer(port: number, timeoutMs = 120_000): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(`http://127.0.0.1:${port}/status`);
      if (response.ok) {
        return;
      }
    } catch {
      // server not ready
    }
    await Bun.sleep(200);
  }
  throw new Error(`debug HTTP server did not start on port ${port}`);
}

export function spawnDebugServer(port: number) {
  const repoRoot = new URL("../..", import.meta.url).pathname;
  const binary = join(repoRoot, "target/debug/oif-debug-http");
  return Bun.spawn([binary, `--debug-http=${port}`], {
    cwd: repoRoot,
    stdout: "inherit",
    stderr: "inherit",
  });
}
