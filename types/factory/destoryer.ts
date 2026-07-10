class DrillBlock extends FactoryBlock implements Directional, Alternateable {
    static item_slot_color: Color = { r: 1, g: 1, b: 1 };
    direction: Direction;
    get work_pos() {
        return this.pos.add(Vec3Unit.from_direction(this.direction));
    }
    constructor(pos: Vec3Int, direction: Direction) {
        super(pos);
        this.direction = direction;
    }
    on_alternate() {
        return new LaserBlock(this.pos, this.direction);
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
        if (block instanceof MaterialBlock) {
            turn_world.blocks.destroy_block(block.pos);
        }
    }
}
class LaserBlock extends FactoryBlock implements Directional, Alternateable, SingalDevice {
    static item_slot_color: Color = { r: 1, g: 1, b: 1 };
    direction: Direction;
    in_network_id: RuntimeNetworkID;
    executed = false;
    constructor(pos: Vec3Int, direction: Direction) {
        super(pos);
        this.direction = direction;
    }
    on_alternate() {
        return new DrillBlock(this.pos, this.direction);
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
        if (!turn_world.signal_networks.get(this.in_network_id).actived) {
            if (this.executed) return;
            this.executed = true;
            new LaserLight(Vec3Unit.from_direction(this.direction).to_facing(), this.pos).execute(turn_world);
        } else {
            this.executed = false;
        }
    }
}

class LaserLight {
    facing: Facing;
    origin: Vec3Int;
    range: number = 30;
    constructor(facing: Facing, origin: Vec3Int) {
        this.origin = origin;
        this.facing = facing;
    }
    execute(turn_world: RuntimeWorld) {
        const direction = Vec3Unit.from_facing(this.facing)
        for (let i = 1; i <= this.range; i++) {
            const block = turn_world.blocks.get_block_by_pos(this.origin.add(direction.multiply(i)));
            if (block instanceof MaterialBlock) {
                turn_world.blocks.destroy_block(block.pos);
            } else if (block instanceof MirrorBlock) {
                block.reflect(this.facing)?.execute(turn_world);
            }
        }
    }
}