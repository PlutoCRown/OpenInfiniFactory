
abstract class Network {
    id: RuntimeNetworkID;
    connections_set: Set<RuntimeBlockID>;
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
abstract class WelderNetwork extends Network {
}

interface SingalDevice {
    in_network_id: RuntimeNetworkID;
}