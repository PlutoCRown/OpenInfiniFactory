type GeneratorTrigger = 'cycle' | 'signal'

type ConstructorType = {
    cycle_count: number;
    offset: number
} | {
    sign_runtime_id: RuntimeBlockID;
}
// 实际上他也 connectable , 因为相连同周期就会直接生成结构
class GeneratorBlock extends SystemBlock {
    trigger: GeneratorTrigger;
    cycle_count: number;
    offset: number;
    sign_runtime_id?: RuntimeBlockID;
    material_type: string

    on_turn(runtime_world: RuntimeTurn): void {
        if (this.trigger === 'cycle') {
            if (runtime_world.turn_world.turn % this.cycle_count === this.offset) {
                // 放置一个方块
            }
        } else {
            // 等到验收器验收了，才放置方块
        }
    }
}