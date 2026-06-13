import { afterAll, beforeAll, describe, expect, test } from "bun:test";
import { DebugClient, spawnDebugServer, waitForServer } from "./client";

const PORT = 9876;
const SIM_FIXTURES = [
  "sim/four_converging_blockers_shared_head.json",
  "sim/reverse_conveyor_platform_fall_push.json",
  "sim/reverse_conveyor_self_push_on_stone.json",
];

let server: ReturnType<typeof spawnDebugServer>;

beforeAll(async () => {
  server = spawnDebugServer(PORT);
  await waitForServer(PORT);
});

afterAll(() => {
  server.kill();
});

describe("debug HTTP simulation core", () => {
  test("lists all registered block kinds", async () => {
    const client = new DebugClient(PORT);
    const body = await client.get("/blockKinds");
    expect(body.ok).toBe(true);
    expect(body.kinds.length).toBeGreaterThanOrEqual(32);
  });

  test("runs simulation fixtures", async () => {
    const client = new DebugClient(PORT);
    for (const path of SIM_FIXTURES) {
      const body = await client.post(`/runFixture?path=${path}`);
      expect(body.ok, JSON.stringify(body)).toBe(true);
    }
  });

  test("four converging blockers expose only one extended device via HTTP", async () => {
    const client = new DebugClient(PORT);
    await client.post("/world/reset");
    await client.post("/loadFixture?path=sim/four_converging_blockers_shared_head.json");
    await client.post("/beginSimulation");
    await client.post("/runOneTurn");
    const extended = await client.get("/getExtendedDevices");
    expect(extended.ok).toBe(true);
    expect(extended.count).toBe(1);
    expect(extended.devices).toHaveLength(1);
    expect(extended.devices[0].head).toEqual({ x: 1, y: 1, z: 1 });
    const center = await client.get("/getPosBlock?x=1&y=1&z=1");
    expect(center.ok).toBe(true);
    expect(center.block?.kind).toBe("BlockerHead");
    expect(center.block?.layer).toBe("virtual");
  });

  test("runN advances turn counter", async () => {
    const client = new DebugClient(PORT);
    await client.post("/world/reset");
    await client.post("/world/place?x=0&y=0&z=0&kind=Stone&facing=North");
    await client.post("/world/place?x=0&y=1&z=0&kind=Platform&facing=North");
    const body = await client.post("/runN?n=3");
    expect(body.ok, JSON.stringify(body)).toBe(true);
    expect(body.simulation.turn).toBe(3);
  });
});
