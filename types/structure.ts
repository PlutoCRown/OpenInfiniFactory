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
/** 运行时这的总世界！包含全部方块、结构、网络等快捷查找信息 */
class World {
    // 方块快速查询表
    blocks: BlockManager;
    // 其他内容
    virtual_blocks: Map<Vec3Int, VirtualBlock>;
    structures: Map<RuntimeStructureID, Structure>;
    signal_networks: NetworkManager;
    constructor() {
        this.blocks = new BlockManager(this);
        this.virtual_blocks = new Map();
        this.structures = new Map();
        this.signal_networks = new NetworkManager(this);
    }
    /** 通过 坐标 查找存在的结构 */
    get_structure_by_pos(pos: Vec3Int): Structure | null {
        const block = this.blocks.get_block_by_pos(pos)
        if (block instanceof FactoryBlock || block instanceof MaterialBlock) {
            return this.structures.get(block.in_structure_id);
        }
        return null;
    }


}
/** 模拟期的世界，在世界的基础上还包含模拟期信息 */
class RuntimeWorld extends World {
    turn: number;
    materials: MaterialStructure[];
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