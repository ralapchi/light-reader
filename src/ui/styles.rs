/*!
主题系统统一入口

本模块作为主题系统的入口点，重新导出主题相关的全部类型和服务。
所有需要主题支持的模块应通过此模块间接引用主题能力，而非直接
依赖 theme.rs 或 theme_service.rs。

用法：
- 初始化主题： `ThemeService::init_fonts(ctx); ThemeService::apply_theme(ctx, &config);`
- 获取主题：   `ThemeConfig::from(kind)` 或 `ThemeConfig::light()` / `dark()` / `sepia()` / `paper()`
*/

pub use crate::ui::theme::{
    ColorValue, ThemeColors, ThemeConfig, ThemePanel, ThemeRadius, ThemeShadow, ThemeSpacing,
    ThemeTypography,
};
pub use crate::ui::theme_service::ThemeService;
