class WelderBlock extends FactoryBlock implements Directional, Alternateable {
    static item_slot_color: Color = { r: 1, g: 1, b: 1 };
    direction: Direction;
    get work_pos() {
        return this.pos.add(Vec3Unit.from_direction(this.direction));
    }
    in_network_id: RuntimeNetworkID;
    constructor(pos: Vec3Int, direction: Direction) {
        super(pos);
        this.direction = direction;
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
    }
    override get work_pos() {
        return this.pos.add(Vec3Unit.Unit_Y_NEG);
    }
}

class WelderPointBlock extends VirtualBlock implements Connectable {
    connected;
    on_turn({ turn_world }: RuntimeTurn) {
        const block = turn_world.blocks.get_block_by_pos(this.pos)
        if (!(block && block instanceof MaterialBlock)) return;
        // 这里通过 connected 开过滤的话还能省点
        turn_world.blocks.find_neighbors(this.pos).forEach(neighbor => {
            if (neighbor instanceof MaterialBlock && neighbor.in_structure_id !== block.in_structure_id) {
                const other_structure = turn_world.structures.get(neighbor.in_structure_id) as MaterialStructure;
                const this_structure = turn_world.structures.get(block.in_structure_id) as MaterialStructure;
                this_structure.merge(other_structure);
                turn_world.structures.delete(other_structure.id);
            }
        });
    }
}