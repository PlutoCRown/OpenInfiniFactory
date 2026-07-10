// #region Base
enum Direction {
    North,
    East,
    South,
    West,
}

enum Facing {
    X, // East
    X_NEG, // West
    Y, // Up
    Y_NEG, // Down
    Z, // South
    Z_NEG, // North
}

interface Color {
    r: number;
    g: number;
    b: number;
}
interface Directional {
    direction: Direction;
}


interface Connectable {
    /** X,X_NEG,Y,Y_NEG,Z,Z_NEG */
    connected: [boolean, boolean, boolean, boolean, boolean, boolean];
    on_update: () => void;
}
/** helper 函数 ，用于将 方向向量填写进 Connectable.connected */
const ConnectedIndexMapper = (unit: Vec3Unit): number => {
    switch (unit) {
        case Vec3Unit.Unit_X:
            return 0;
        case Vec3Unit.Unit_X_NEG:
            return 1;
        case Vec3Unit.Unit_Y:
            return 2;
        case Vec3Unit.Unit_Y_NEG:
            return 3;
        case Vec3Unit.Unit_Z:
            return 4;
        case Vec3Unit.Unit_Z_NEG:
            return 5;
    }
}


// #endregion
// #region Utilities
class Vec3Int implements Serializable {
    x: number;
    y: number;
    z: number;
    constructor(x: number, y: number, z: number) {
        this.x = x;
        this.y = y;
        this.z = z;
    }
    serialize(writer: Writer) {
        writer.write(this.x);
        writer.write(this.y);
        writer.write(this.z);
    }
    deserialize(reader: Reader) {
        this.x = reader.read_number();
        this.y = reader.read_number();
        this.z = reader.read_number();
    }
    add(other: Vec3Int): Vec3Int {
        return new Vec3Int(this.x + other.x, this.y + other.y, this.z + other.z);
    }
    subtract(other: Vec3Int): Vec3Int {
        return new Vec3Int(this.x - other.x, this.y - other.y, this.z - other.z);
    }
    multiply(other: number): Vec3Int {
        return new Vec3Int(this.x * other, this.y * other, this.z * other);
    }
    inverse(): Vec3Unit {
        return new Vec3Unit(-this.x, -this.y, -this.z);
    }

    /** 转为单位方向 */
    as_unit(): Vec3Unit | undefined {
        const key = `${this.x},${this.y},${this.z}`;
        const map = {
            '1,0,0': Vec3Unit.Unit_X,
            '-1,0,0': Vec3Unit.Unit_X_NEG,
            '0,1,0': Vec3Unit.Unit_Y,
            '0,-1,0': Vec3Unit.Unit_Y_NEG,
            '0,0,1': Vec3Unit.Unit_Z,
            '0,0,-1': Vec3Unit.Unit_Z_NEG,
        }
        return map[key];
    }
}
class Vec3Unit extends Vec3Int {
    /** East */
    static Unit_X = new Vec3Unit(1, 0, 0);
    /** North */
    static Unit_Y = new Vec3Unit(0, 1, 0);
    /** Up */
    static Unit_Z = new Vec3Unit(0, 0, 1);
    /** West */
    static Unit_X_NEG = new Vec3Unit(-1, 0, 0);
    /** South */
    static Unit_Y_NEG = new Vec3Unit(0, -1, 0);
    /** Down */
    static Unit_Z_NEG = new Vec3Unit(0, 0, -1);
    /** 根据方向获取对应的向量 */
    static from_direction(direction: Direction): Vec3Unit {
        switch (direction) {
            case Direction.North:
                return Vec3Unit.Unit_Z;
            case Direction.East:
                return Vec3Unit.Unit_Z_NEG;
            case Direction.South:
                return Vec3Unit.Unit_X;
            case Direction.West:
                return Vec3Unit.Unit_X_NEG;
        }
    }
    /** 根据 facing 获取对应的向量 */
    static from_facing(facing: Facing): Vec3Unit {
        switch (facing) {
            case Facing.X:
                return Vec3Unit.Unit_X;
            case Facing.X_NEG:
                return Vec3Unit.Unit_X_NEG;
            case Facing.Y:
                return Vec3Unit.Unit_Y;
        }
    }
    /** 根据向量获取对应的 facing */
    static to_facing(facing: Vec3Int): Facing {
        switch (facing) {
            case Vec3Unit.Unit_X:
                return Facing.X;
            case Vec3Unit.Unit_X_NEG:
                return Facing.X_NEG;
            case Vec3Unit.Unit_Y:
                return Facing.Y;
            case Vec3Unit.Unit_Y_NEG:
                return Facing.Y_NEG;
            case Vec3Unit.Unit_Z:
                return Facing.Z;
            case Vec3Unit.Unit_Z_NEG:
                return Facing.Z_NEG;
        }
    }

    /** 将向量转换为 facing */
    to_facing(): Facing {
        return Vec3Unit.to_facing(this);
    }
}
// #endregion
// #region Blocks
abstract class Block implements Serializable {
    pos: Vec3Int;
    static item_slot_color: Color;
    constructor(pos: Vec3Int) {
        this.pos = pos;
    }
    /** 序列化 */
    serialize(writer: Writer) {
        this.pos.serialize(writer);
    }
    /** 反序列化 */
    deserialize(reader: Reader) {
        this.pos.deserialize(reader);
    }
    // FIXME: 下面 3 个都是 abstract method，但是现在先不写
    on_updated(world: World) { }
    on_placed(world: World) { }
    on_removed(world: World) { }
}

abstract class FactoryBlock extends Block {
    runtime_id: RuntimeBlockID;
    abstract on_turn(runtime_world: RuntimeTurn): void;
    in_structure_id: RuntimeStructureID;
    override on_placed(world: World) {
        world.blocks.add_block(this);
    }
}
interface Alternateable {
    /** 替换方块 */
    on_alternate(): void;
}

abstract class MaterialBlock extends Block {
    in_structure_id: RuntimeStructureID;
}

abstract class VirtualBlock extends Block {

}


class BlockManager {
    blocks_poss: Map<Vec3Int, Block>;
    blocks: Map<RuntimeBlockID, Block>;
    blocks_ids: WeakMap<Block, RuntimeBlockID>;
    world: World;
    constructor(world: World) {
        this.world = world;
        this.blocks = new Map();
        this.blocks_ids = new WeakMap();
    }
    add_block(block: Block): void {
        this.blocks_poss.set(block.pos, block);
        // 工厂方块、材料方块 才有必要有 runtime_id
        if ('runtime_id' in block) {
            const runtime_id = block.runtime_id as RuntimeBlockID;
            this.blocks.set(runtime_id, block);
            this.blocks_ids.set(block, runtime_id);
        }
    }
    /** 通过 坐标 查找存在的方块 */
    get_block_by_pos(pos: Vec3Int): Block | null {
        return this.blocks_poss.get(pos);
    }
    get_block_by_id(id: RuntimeBlockID): Block {
        return this.blocks.get(id)!;
    }
    /** 通过 坐标 查找存在的方块的邻居 */
    find_neighbors(pos: Vec3Int): Block[] {
        return [
            this.get_block_by_pos(pos.add(Vec3Unit.Unit_X)),
            this.get_block_by_pos(pos.add(Vec3Unit.Unit_X_NEG)),
            this.get_block_by_pos(pos.add(Vec3Unit.Unit_Y)),
            this.get_block_by_pos(pos.add(Vec3Unit.Unit_Y_NEG)),
            this.get_block_by_pos(pos.add(Vec3Unit.Unit_Z)),
        ];
    }
    /** 通过 坐标 销毁存在的方块 */
    destroy_block(pos: Vec3Int): void {
        const block = this.get_block_by_pos(pos);
        this.blocks_poss.delete(pos);
        if ('runtime_id' in block) {
            const runtime_id = block.runtime_id as RuntimeBlockID;
            this.blocks.delete(runtime_id);
            this.blocks_ids.delete(block);
        }
        // FIXME: 这里要进行结构重建
    }

}
// #endregion