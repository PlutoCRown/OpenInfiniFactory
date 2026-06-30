class PusherBlock extends FactoryBlock implements Directional, Alternateable, SingalDevice {
    static item_slot_color: Color = { r: 1, g: 1, b: 1 };
    direction: Direction;
    in_network_id: RuntimeNetworkID;
    active: boolean;
    constructor(pos: Vec3Int, direction: Direction) {
        super(pos);
        this.direction = direction;
    }
    on_alternate() {
        return new BlockerBlock(this.pos, this.direction);
    }
    serialize(writer: Writer) {
        this.pos.serialize(writer);
        writer.write(this.direction);
    }
    deserialize(reader: Reader) {
        this.pos.deserialize(reader);
        this.direction = reader.read_number() as Direction;
    }
    on_turn({ turn_world }: RuntimeTurn) {
        if (!turn_world.signal_networks.get(this.in_network_id).actived == this.active) return
        // WIP
    }
}
class BlockerBlock extends PusherBlock {
}