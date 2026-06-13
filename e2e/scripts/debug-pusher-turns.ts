#!/usr/bin/env bun
/** Dump per-turn simulation logs for the pusher fixture. */
import { DebugClient, spawnDebugServer, waitForServer } from "../src/client";

const port = 8778;
const client = new DebugClient(port);
const server = spawnDebugServer(port);
await waitForServer(port);

try {
  await client.post("/world/reset");
  await client.post("/loadFixture?path=sim/test_pusher_platform_stuck.json");
  await client.post("/beginSimulation");

  for (let turn = 1; turn <= 3; turn += 1) {
    await client.post("/runOneTurn");
    const logs = await client.get("/logs?limit=100");
    console.log(`\n===== turn ${turn} =====`);
    for (const line of logs.logs ?? []) {
      if (
        line.includes("devices") ||
        line.includes("merged") ||
        line.includes("gravity") ||
        line.includes("signals") ||
        line.includes("actuating") ||
        line.includes("translate")
      ) {
        console.log(line);
      }
    }
    const stuck = await client.get("/getPosBlock?x=3&y=2&z=1");
    const pusher = await client.get("/getPosBlock?x=2&y=2&z=1");
    console.log("platform (3,2,1):", stuck.block ?? "empty");
    console.log("pusher (2,2,1):", pusher.block ?? "empty");
  }
} finally {
  server.kill();
}
