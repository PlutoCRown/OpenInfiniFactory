class RotatorBlock extends FactoryBlock implements Alternateable {
    static item_slot_color: Color = { r: 1, g: 1, b: 1 };
    static is_counter = false;
    cache_exceuted_structure: RuntimeStructureID | null = null;
    on_alternate() {
        return new CounterRotatorBlock(this.pos);
    }
    on_turn({ turn_world }: RuntimeTurn) {
        const structure = turn_world.get_structure_by_pos(this.pos.add(Vec3Unit.Unit_Y));
        if (this.cache_exceuted_structure === (structure as MoveableStructure).id) return;
        if (structure instanceof MoveableStructure) {
            // FIXME: 这里要判断空间是否足够旋转
            structure.create_movement_tag(this.runtime_id, Facing.Y, MovementType.Rotate);
            // FIXME: 这里如果执行成功了，那么要给这个结构打标记，然后就不再执行
        } else {
            this.cache_exceuted_structure = null;
        }
    }
}
class CounterRotatorBlock extends RotatorBlock {
    static is_counter = true;
}