class DetectorBlock extends FactoryBlock implements Directional, Alternateable, SingalDevice {
    static item_slot_color: Color = { r: 1, g: 1, b: 1 };
    direction: Direction;
    in_network_id: RuntimeNetworkID;
    work_pos: Vec3Int;
    constructor(pos: Vec3Int, direction: Direction) {
        super(pos);
        this.direction = direction;
        this.work_pos = pos.add(Vec3Unit.from_direction(direction));
    }
    on_alternate() {
        return new DownDetectorBlock(this.pos, this.direction);
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
        const block = turn_world.blocks.get_block_by_pos(this.work_pos);
        if (block instanceof PlatformBlock || block instanceof MaterialBlock) {
            turn_world.signal_networks.get(this.in_network_id)?.activate();
        }
    }
}
class DownDetectorBlock extends DetectorBlock {
    on_alternate() {
        return new DetectorBlock(this.pos, this.direction);
    }
}