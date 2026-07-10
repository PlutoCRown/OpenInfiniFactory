class WelderBlock extends FactoryBlock implements Directional, Alternateable {
    static item_slot_color: Color = { r: 1, g: 1, b: 1 };
    direction: Direction;
    work_pos: Vec3Int;
    in_network_id: RuntimeNetworkID;
    constructor(pos: Vec3Int, direction: Direction) {
        super(pos);
        this.direction = direction;
        this.work_pos = pos.add(Vec3Unit.from_direction(direction));
    }
    on_alternate() {
        return new DownWelderBlock(this.pos, this.direction);
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
            turn_world.blocks.find_neighbors(this.work_pos).forEach(neighbor => {
                if (neighbor instanceof MaterialBlock && neighbor.in_structure_id !== this.in_structure_id) {
                    const other_structure = turn_world.structures.get(neighbor.in_structure_id) as MaterialStructure;
                    const this_structure = turn_world.structures.get(this.in_structure_id) as MaterialStructure;
                    this_structure.merge(other_structure);
                    turn_world.structures.delete(other_structure.id);
                }
            });
        }
    }
}
class DownWelderBlock extends WelderBlock {
    constructor(pos: Vec3Int, direction: Direction) {
        super(pos, direction);
        this.work_pos = pos.add(Vec3Unit.Unit_Y_NEG);
    }
}

class WelderPointBlock extends VirtualBlock implements Connectable {
    connected;
    on_update() {
        // FIXME: 链接逻辑

    }
}