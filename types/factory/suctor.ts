class SuctorBlock extends FactoryBlock implements Directional, SingalDevice {
    static item_slot_color: Color = { r: 1, g: 1, b: 1 };
    direction: Direction;
    in_network_id: RuntimeNetworkID;
    constructor(pos: Vec3Int, direction: Direction) {
        super(pos);
        this.direction = direction;
    }
    on_alternate() {
        return new SuctorBlock(this.pos, this.direction);
    }
    on_turn({ turn_world }: RuntimeTurn) {
        const block = turn_world.blocks.get_block_by_pos(this.pos.add(Vec3Unit.from_direction(this.direction)));
        if (block instanceof MaterialBlock) {
            turn_world.blocks.destroy_block(block.pos);
        }
    }
}
class DownSuctorBlock extends SuctorBlock {
    on_alternate() {
        return new SuctorBlock(this.pos, this.direction);
    }
}