use super::{
    rgb, rgba, Block, BlockDefinition, BlockKind, BlockModel, BlockModelPart, ModelMaterial,
    ModelMesh, RenderBehavior, SystemBlock, WeldBehavior, WeldConnectorBehavior,
};

const MODEL: &[BlockModelPart] = &[
    BlockModelPart::new(ModelMesh::RodX, ModelMaterial::Welding, [0.0, 0.0, 0.0])
        .scaled([0.58, 0.42, 0.42]),
    BlockModelPart::new(ModelMesh::RodY, ModelMaterial::Welding, [0.0, 0.0, 0.0])
        .scaled([0.42, 0.58, 0.42]),
    BlockModelPart::new(ModelMesh::RodZ, ModelMaterial::Welding, [0.0, 0.0, 0.0])
        .scaled([0.42, 0.42, 0.58]),
];

pub struct WeldPointBlock;

pub static WELD_POINT: WeldPointBlock = WeldPointBlock;

impl Block for WeldPointBlock {
    fn id(&self) -> BlockKind {
        BlockKind::WeldPoint
    }

    fn definition(&self) -> BlockDefinition {
        BlockDefinition::marker(
            self.id(),
            "block.weld_point",
            "short.weld_point",
            rgba(1.0, 0.28, 0.18, 0.45),
            rgb(0.86, 0.16, 0.12),
        )
        .node()
        .transparent()
        .no_collision()
    }

    fn render_behavior(&self, _facing: super::Facing) -> RenderBehavior {
        RenderBehavior {
            weld_connector: Some(WeldConnectorBehavior::AllSides),
            ..Default::default()
        }
    }

    fn weld_behavior(&self) -> Option<WeldBehavior> {
        Some(WeldBehavior::Node)
    }

    fn model(&self) -> BlockModel {
        BlockModel::Parts(MODEL)
    }
}

impl SystemBlock for WeldPointBlock {}
