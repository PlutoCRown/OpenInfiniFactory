class ConveyorBlock extends FactoryBlock implements Directional, Alternateable {
    static item_slot_color: Color = { r: 1, g: 1, b: 1 };
    direction: Direction;
    work_face: Vec3Int = Vec3Unit.Unit_Y;
    constructor(pos: Vec3Int, direction: Direction) {
        super(pos);
        this.direction = direction;
    }
    on_alternate() {
        return new ReverseConveyorBlock(this.pos, this.direction);
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
        const structure = turn_world.get_structure_by_pos(this.pos.add(this.work_face));
        const direction_vector = Vec3Unit.from_direction(this.direction);
        if (structure instanceof MoveableStructure) {
            structure.create_movement_tag(this.runtime_id, direction_vector.to_facing(), MovementType.Translate);
        }
    }
}
class ReverseConveyorBlock extends ConveyorBlock {
    work_face: Vec3Int = Vec3Unit.Unit_Y_NEG;
}