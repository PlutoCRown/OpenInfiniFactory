class MirrorBlock extends FactoryBlock implements Directional, Alternateable {
    static item_slot_color: Color = { r: 1, g: 1, b: 1 };
    direction: Direction;
    constructor(pos: Vec3Int, direction: Direction) {
        super(pos);
        this.direction = direction;
    }
    on_alternate() {
        return new VerticalMirrorBlock(this.pos, this.direction);
    }
    on_turn() {
    }
    reflect(facing: Facing): LaserLight | null {
        // 这边应该是水平分光， 
        //  东西 => /
        //  南北 => \
        if (facing === Facing.Y || facing === Facing.Y_NEG) return null;
        if (this.direction === Direction.East || this.direction === Direction.West) {
            if (facing === Facing.X) return new LaserLight(Facing.Z_NEG, this.pos);
            if (facing === Facing.X_NEG) return new LaserLight(Facing.Z, this.pos);
            if (facing === Facing.Z) return new LaserLight(Facing.X, this.pos);
            if (facing === Facing.Z_NEG) return new LaserLight(Facing.X, this.pos);
        }
        if (this.direction === Direction.North || this.direction === Direction.South) {
            if (facing === Facing.Z) return new LaserLight(Facing.X, this.pos);
            if (facing === Facing.Z_NEG) return new LaserLight(Facing.X_NEG, this.pos);
            if (facing === Facing.X) return new LaserLight(Facing.Z, this.pos);
            if (facing === Facing.X_NEG) return new LaserLight(Facing.Z_NEG, this.pos);
        }
        return null;
    }
}

class VerticalMirrorBlock extends MirrorBlock {
    on_alternate() {
        return new MirrorBlock(this.pos, this.direction);
    }
    override reflect(facing: Facing): LaserLight | null {
        const thisDir = Vec3Unit.from_direction(this.direction);
        if (facing === Facing.Y) return new LaserLight(thisDir.inverse().to_facing(), this.pos);
        if (facing === Facing.Y_NEG) return new LaserLight(thisDir.to_facing(), this.pos);
        if (facing === thisDir.to_facing()) return new LaserLight(Facing.Y, this.pos);
        if (facing === thisDir.inverse().to_facing()) return new LaserLight(Facing.Y_NEG, this.pos);
        return null;
    }
}

class SplitterBlock extends FactoryBlock implements Directional {
    static item_slot_color: Color = { r: 1, g: 1, b: 1 };
    direction: Direction;
    constructor(pos: Vec3Int, direction: Direction) {
        super(pos);
        this.direction = direction;
    }
    on_turn() { }
    reflect(facing: Facing): LaserLight[] {
        const lights = [];
        lights.push(new MirrorBlock(this.pos, this.direction).reflect(facing));
        lights.push(new VerticalMirrorBlock(this.pos, this.direction).reflect(facing));
        return lights;
    }
}