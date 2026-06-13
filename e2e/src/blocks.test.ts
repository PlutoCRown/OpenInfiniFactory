import { afterAll, beforeAll, describe, expect, test } from "bun:test";
import { DebugClient, spawnDebugServer, waitForServer } from "./client";

const PORT = 9876;
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

  test("runs every block placement fixture", async () => {
    const client = new DebugClient(PORT);
    const body = await client.post("/runAllFixtures");
    expect(body.ok).toBe(true);
    expect(body.total).toBeGreaterThanOrEqual(32);
    expect(body.passed).toBe(body.total);
    for (const result of body.results) {
      expect(result.ok, result.error ?? result.name).toBe(true);
    }
  });

  test("runs simulation fixtures", async () => {
    const client = new DebugClient(PORT);
    for (const path of ["sim/welder_weld_point.json", "sim/wire_detector_power.json"]) {
      const body = await client.post(`/runFixture?path=${path}`);
      expect(body.ok, JSON.stringify(body)).toBe(true);
    }
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
