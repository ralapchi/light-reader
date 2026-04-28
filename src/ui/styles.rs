/*!
主题系统统一入口

本模块作为主题系统的统一入口点，仅对外暴露活跃运行路径
真正需要使用的主题配置与服务类型。

用法：
- 初始化主题： `ThemeService::init_fonts(ctx); ThemeService::apply_theme(ctx, &config);`
- 获取主题：   `ThemeConfig::from(kind)` 或 `ThemeConfig::light()` / `dark()` / `sepia()` / `paper()`
*/

pub use crate::ui::theme::ThemeConfig;
pub use crate::ui::theme_service::ThemeService;
