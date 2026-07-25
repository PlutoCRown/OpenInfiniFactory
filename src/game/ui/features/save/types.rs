use bevy::prelude::*;
use bevy::tasks::Task;

#[derive(Component, Clone, Debug, Eq, PartialEq)]
pub enum SaveListAction {
    NewPuzzle,
    NewSolution,
    /// 选中左侧谜题
    SelectPuzzle(String),
    /// 选中上方方案卡片
    SelectSolution(String),
    /// 页脚左：编辑当前选中谜题
    EditSelectedPuzzle,
    /// 页脚左：重命名当前选中谜题
    RenameSelectedPuzzle,
    /// 页脚左：删除当前选中谜题
    DeleteSelectedPuzzle,
    /// 页脚右：重命名当前选中方案
    RenameSelectedSolution,
    /// 页脚右：删除当前选中方案
    DeleteSelectedSolution,
    /// 页脚右：用当前选中方案开始游戏
    StartGame,
    Back,
}

#[derive(Component)]
pub struct SaveListCloseButton;

#[derive(Component)]
pub struct SaveListPanel;

/// 左侧谜题列表行容器
#[derive(Component, Clone, Copy)]
pub struct SaveListPuzzleRows;

/// 右侧方案横滑内容容器
#[derive(Component, Clone, Copy)]
pub struct SaveListSolutionRows;

/// 方案横向滚动视口
#[derive(Component)]
pub struct SaveListSolutionScroll {
    pub offset: f32,
    pub max_offset: f32,
}

/// 谜题纵向滚动视口
#[derive(Component)]
pub struct SaveListPuzzleScroll {
    pub offset: f32,
    pub max_offset: f32,
}

/// 封面展示宿主（裁剪区）
#[derive(Component)]
pub struct SaveListCoverHost;

/// 封面 ImageNode
#[derive(Component)]
pub struct SaveListCoverImage;

/// 封面加载中提示
#[derive(Component)]
pub struct SaveListCoverLoading;

#[derive(Component)]
pub struct SaveListTitleText;

#[derive(Resource, Default)]
pub struct SaveListRenderState {
    pub puzzle_keys: Vec<String>,
    pub solution_keys: Vec<String>,
    /// 行重建后下一帧再刷按钮样式/标签
    pub paint_buttons: bool,
    /// 本帧刚重建过行（样式系统勿清 paint_buttons）
    pub rows_rebuilt: bool,
    pub last_hover: Option<Entity>,
    /// 当前请求中的封面存档路径（storage_path），无选中时为 ""
    pub cover_key: Option<String>,
    /// 后台读盘/解码封面；(路径, 解码结果)
    pub cover_task: Option<Task<(String, Option<Image>)>>,
}
