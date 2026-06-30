// 声明式 UI 按钮表：`on_click(ctx) { ... }` 展开为 const 嵌套 fn。

#[macro_export]
macro_rules! list_ui_config {
    (
        $button:ident,
        ctx: $ctx:ty,
        $($entry:tt);* $(;)?
    ) => {
        &[
            $( list_ui_config!(@entry $button, $ctx, $entry) ),*
        ]
    };

    (@entry StartMenuButton, $ctx:ty, {
        key: $key:literal
        on_click($ctx_param:ident, $commands:ident) { $($click_body:tt)* }
    }) => {
        StartMenuButton {
            label_key: $key,
            on_click: {
                fn on_click($ctx_param: &mut $ctx, $commands: &mut Commands) {
                    $($click_body)*
                }
                on_click
            },
        }
    };

    (@entry PauseMenuButton, $ctx:ty, {
        key: $key:literal
        $(visible($vis_a:ident, $vis_b:ident) { $($vis_body:tt)* })?
        $(label($label_save:ident) { $($label_body:tt)* })?
        on_click($ctx_param:ident, $commands:ident) { $($click_body:tt)* }
    }) => {
        PauseMenuButton {
            label_key: $key,
            label: list_ui_config!(@label $(label($label_save) { $($label_body)* })?),
            visible: list_ui_config!(@visible $(visible($vis_a, $vis_b) { $($vis_body)* })?),
            on_click: {
                fn on_click($ctx_param: &mut $ctx, $commands: &mut Commands) {
                    $($click_body)*
                }
                on_click
            },
        }
    };

    (@entry SaveListToolbarButton, $ctx:ty, {
        for $action:path =>
        on_click($ctx_param:ident) { $($click_body:tt)* }
    }) => {
        SaveListToolbarButton {
            action: $action,
            on_click: {
                fn on_click($ctx_param: &mut $ctx) {
                    $($click_body)*
                }
                on_click
            },
        }
    };

    (@entry SettingsFooterButton, $ctx:ty, {
        for $action:path =>
        on_click($ctx_param:ident, $commands:ident) { $($click_body:tt)* }
    }) => {
        SettingsFooterButton {
            action: $action,
            on_click: {
                fn on_click($ctx_param: &mut $ctx, $commands: &mut Commands) {
                    $($click_body)*
                }
                on_click
            },
        }
    };

    (@visible visible($vis_a:ident, $vis_b:ident) { $($vis_body:tt)* }) => {{
        fn visible($vis_a: &SaveState, $vis_b: &SolutionState) -> bool {
            $($vis_body)*
        }
        visible
    }};

    (@visible) => {{
        fn visible(_save: &SaveState, _solution: &SolutionState) -> bool {
            true
        }
        visible
    }};

    (@label label($label_save:ident) { $($label_body:tt)* }) => {
        Some({
            fn label($label_save: &SaveState) -> String {
                $($label_body)*
            }
            label
        })
    };

    (@label) => {
        None
    };
}
