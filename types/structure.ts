enum StructureType {
    Factory,
    Material,
    Scene
}
type RuntimeBlockID = Number;
type RuntimeStructureID = Number;
type RuntimeNetworkID = Number;
enum MovementType {
    Thrust,
    Lift,
    Gravity,
    Rotate,
    Translate,
}
type MovementTag = {
    source: RuntimeBlockID;
    facing: Facing;
    type: MovementType;
}
// #region Structure
abstract class Structure {
    id: RuntimeStructureID;
    blocks: Block[];
    abstract type: StructureType;
}
abstract class MoveableStructure extends Structure {
    movementTags: MovementTag[];
    translateHistory: Map<RuntimeBlockID, number>;
    can_move_to(_: Facing): boolean {
        return false;
    }
    create_movement_tag(source: RuntimeBlockID, facing: Facing, type: MovementType): void {
        if (this.can_move_to(facing)) this.movementTags.push({ source, facing, type });
    }
}
class FactoryStructure extends MoveableStructure {
    type: StructureType.Factory;
}
class MaterialStructure extends MoveableStructure {
    type: StructureType.Material;
    merge(other: MaterialStructure): void {
        this.blocks.push(...other.blocks);
        this.blocks.forEach(block => {
            (block as MaterialBlock).in_structure_id = this.id;
        });
    }
}
class SceneStructure extends Structure {
    type: StructureType.Scene;
}
// #endregion
// #region World
class World {
    blocks: Map<Vec3Int, Block>;
    structures: Map<RuntimeStructureID, Structure>;
    signal_networks: Map<RuntimeNetworkID, SignalNetwork>;
    welder_networks: Map<RuntimeNetworkID, WelderNetwork>;
}
class RuntimeWorld extends World {
    turn: number;
    materials: MaterialStructure[];
    get_structure_by_pos(pos: Vec3Int): Structure | null {
        const block = this.blocks.get(pos)
        if (block instanceof FactoryBlock || block instanceof MaterialBlock) {
            return this.structures.get(block.in_structure_id);
        }
        return null;
    }
    get_block_by_pos(pos: Vec3Int): Block | null {
        return this.blocks.get(pos);
    }

    find_neighbors(pos: Vec3Int): Block[] {
        return [
            this.get_block_by_pos(pos.add(Vec3Unit.Unit_X)),
            this.get_block_by_pos(pos.add(Vec3Unit.Unit_X_NEG)),
            this.get_block_by_pos(pos.add(Vec3Unit.Unit_Y)),
            this.get_block_by_pos(pos.add(Vec3Unit.Unit_Y_NEG)),
            this.get_block_by_pos(pos.add(Vec3Unit.Unit_Z)),
        ];
    }
    destroy_block(pos: Vec3Int): void {
        const block = this.get_block_by_pos(pos);
        this.blocks.delete(pos);
        // FIXME: 这里要进行结构重建
    }
}

class RuntimeTurn {
    /** 保存的 world 快照 */
    solution: World;
    /** 当前的 world 状态 */
    turn_world: RuntimeWorld;
    constructor(solution: World, turn_world: RuntimeWorld) {
        this.solution = solution;
        this.turn_world = turn_world;
    }
}
// #endregion
type MovetimeTurn = RuntimeTurn & {
    snapshot: RuntimeWorld
}