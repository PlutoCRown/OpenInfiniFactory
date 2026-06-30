class LifterBlock extends FactoryBlock {
    static item_slot_color: Color = { r: 1, g: 1, b: 1 };
    static range: number = 5;
    on_turn({ turn_world }: RuntimeTurn) {
        for (let i = 1; i <= LifterBlock.range; i++) {
            const structure = turn_world.get_structure_by_pos(this.pos.add(Vec3Unit.Unit_Y.multiply(i)));
            if (structure instanceof MoveableStructure) structure.create_movement_tag(this.runtime_id, Facing.Y, MovementType.Lift);
        }
    }
}