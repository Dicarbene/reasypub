#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // 在 Windows 发布版本中隐藏控制台窗口

// 原生平台入口（非 WebAssembly）。
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    // 初始化日志系统。
    env_logger::init();

    // 配置原生窗口选项。
    let native_options = eframe::NativeOptions {
        viewport: {
            let viewport = egui::ViewportBuilder::default()
                .with_inner_size([800.0, 600.0])
                .with_min_inner_size([300.0, 220.0]);
            match eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon.png")[..]) {
                Ok(icon) => viewport.with_icon(icon),
                Err(err) => {
                    eprintln!("Failed to load icon: {err}");
                    viewport
                }
            }
        },
        ..Default::default() // 其余选项使用默认值。
    };

    // 启动原生应用。
    eframe::run_native(
        "Reasypub",                                              // 应用窗口标题。
        native_options,                                          // 窗口配置。
        Box::new(|cc| Ok(Box::new(reasypub::MainApp::new(cc)))), // 应用实例构造闭包。
    )
}

// Web 平台入口（使用 trunk）。
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // 将日志输出重定向到浏览器控制台。
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    // 配置 Web 运行选项。
    let web_options = eframe::WebOptions::default();

    /// wasm 启动阶段的兜底错误提示。
    ///
    /// 该函数保持“尽力而为”：即便 DOM 节点缺失，也不让启动流程
    /// 再次 panic，确保用户至少能看到可理解的失败信息。
    fn show_loading_error(message: &str) {
        if let Some(window) = web_sys::window()
            && let Some(document) = window.document()
            && let Some(loading_text) = document.get_element_by_id("loading_text")
        {
            loading_text.set_inner_html(&format!("<p>{message}</p>"));
        }
    }

    // 在 Web 环境中异步启动应用。
    wasm_bindgen_futures::spawn_local(async {
        let Some(window) = web_sys::window() else {
            log::error!("No window available");
            show_loading_error("Unable to start app: no browser window.");
            return;
        };

        let Some(document) = window.document() else {
            log::error!("No document available");
            show_loading_error("Unable to start app: no document.");
            return;
        };

        let Some(canvas_el) = document.get_element_by_id("the_canvas_id") else {
            log::error!("Failed to find canvas element: the_canvas_id");
            show_loading_error("Unable to start app: canvas not found.");
            return;
        };

        let Ok(canvas) = canvas_el.dyn_into::<web_sys::HtmlCanvasElement>() else {
            log::error!("the_canvas_id was not a HtmlCanvasElement");
            show_loading_error("Unable to start app: invalid canvas element.");
            return;
        };

        // 启动 Web 运行器。
        let start_result = eframe::WebRunner::new()
            .start(
                canvas,                                                  // Canvas 元素。
                web_options,                                             // Web 配置。
                Box::new(|cc| Ok(Box::new(reasypub::MainApp::new(cc)))), // 应用实例构造闭包。
            )
            .await;

        // 处理加载提示：成功则移除，失败则显示错误。
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    // 启动成功：移除加载提示。
                    loading_text.remove();
                }
                Err(e) => {
                    log::error!("Failed to start eframe: {e:?}");
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                }
            }
        }
    });
}
