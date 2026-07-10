
class WireBlock extends FactoryBlock implements Connectable, SingalDevice {
    static item_slot_color: Color = { r: 1, g: 1, b: 1 };
    in_network_id: Number;
    on_turn(_: RuntimeTurn) { }

    connected: [boolean, boolean, boolean, boolean, boolean, boolean];
    on_update() {
        // FIXME: 链接逻辑
    }
    on_placed(world: World): void {
        const neighbors = world.find_neighbors(this.pos);
        neighbors.forEach(neighbor => {
            // 更新链接状态
            if (neighbor instanceof WireBlock) {
                this.connected[ConnectedIndexMapper(neighbor.pos.subtract(this.pos).as_unit()!)] = true;
                neighbor.connected[ConnectedIndexMapper(this.pos.subtract(neighbor.pos).as_unit()!)] = true;
            }
            // 加入电网
            let isIndividual = true;
            if ('in_network_id' in neighbor) { // neighbor instanceof SingalDevice
                const device = neighbor as SingalDevice;
                const network = world.signal_networks.get(device.in_network_id);
                if (network) {
                    if (isIndividual) {
                        isIndividual = false;
                        device.in_network_id = network.id;
                    } else {
                        world.signal_networks.merge(this.in_network_id, device.in_network_id);
                    }
                }
            }
            if (isIndividual) {
                const network = world.signal_networks.create();
                this.in_network_id = network.id;
            }
        });
    }
}