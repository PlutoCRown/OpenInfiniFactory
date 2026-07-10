
abstract class Network {
    id: RuntimeNetworkID;
    connections_set: Set<RuntimeBlockID>;
    constructor() {
        this.id = IDGenerator();
        this.connections_set = new Set();
    }
}

class SignalNetwork extends Network {
    actived: boolean;
    activate(): void {
        this.actived = true;
    }
    deactivate(): void {
        this.actived = false;
    }
}

interface SingalDevice {
    in_network_id: RuntimeNetworkID;
}

class NetworkManager {
    networks: Map<RuntimeNetworkID, SignalNetwork>;
    world: World; // 所在的世界
    constructor(world: World) {
        this.world = world;
        this.networks = new Map();
    }
    get(id: RuntimeNetworkID): SignalNetwork {
        return this.networks.get(id)!;
    }
    create() {
        const network = new SignalNetwork();
        this.networks.set(network.id, network);
        return network;
    }
    delete(id: RuntimeNetworkID): void {
        this.networks.delete(id);
    }
    merge(network_id1: RuntimeNetworkID, network_id2: RuntimeNetworkID): void {
        const network1 = this.get(network_id1);
        const network2 = this.get(network_id2);
        network2.connections_set.forEach(connection => {
            const block = this.world.blocks.get_block_by_id(connection)
            if ('in_network_id' in block) block.in_network_id = network_id2;
        });
        network1.connections_set.forEach(connection => network2.connections_set.add(connection));
        this.delete(network_id1);
    }
}