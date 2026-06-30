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

    to_facing(): Facing {
        return Vec3Unit.to_facing(this);
    }
    inverse(): Vec3Unit {
        return new Vec3Unit(-this.x, -this.y, -this.z);
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
    serialize(writer: Writer) {
        this.pos.serialize(writer);
    }
    deserialize(reader: Reader) {
        this.pos.deserialize(reader);
    }
    // FIXME: 下面 3 个都是 abstract method，但是现在先不写
    on_updated(runtime_world: RuntimeTurn) { }
    on_placed(runtime_world: RuntimeTurn) { }
    on_removed(runtime_world: RuntimeTurn) { }
}

abstract class FactoryBlock extends Block {
    runtime_id: RuntimeBlockID;
    abstract on_turn(runtime_world: RuntimeTurn): void;
    in_structure_id: RuntimeStructureID;
}
interface Alternateable {
    on_alternate(): void;
}

abstract class MaterialBlock extends Block {
    in_structure_id: RuntimeStructureID;
}

abstract class VirtualBlock extends Block {

}
// #endregion