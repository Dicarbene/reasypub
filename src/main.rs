#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // 在 Windows 发布版本中隐藏控制台窗口

// 当编译为原生平台时（非 WebAssembly）:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    // 初始化日志记录器
    env_logger::init();

    // 配置原生应用窗口选项
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0]) // 设置窗口初始大小为 800x600
            .with_min_inner_size([300.0, 220.0]) // 设置窗口最小尺寸为 300x220
            .with_icon(
                // 注意：添加图标是可选的
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default() // 使用默认值填充其他选项
    };

    // 运行原生应用
    eframe::run_native(
        "Reasypub",                                              // 应用窗口标题
        native_options,                                          // 窗口配置选项
        Box::new(|cc| Ok(Box::new(reasypub::MainApp::new(cc)))), // 创建应用实例的闭包
    )
}

// 当编译为 Web 平台时（使用 trunk）:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // 将日志消息重定向到 console.log 和相关函数
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    // 配置 Web 应用选项
    let web_options = eframe::WebOptions::default();

    // 在 Web 环境中异步启动应用
    wasm_bindgen_futures::spawn_local(async {
        // 获取浏览器 window 对象
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        // 获取 HTML 中的 canvas 元素
        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        // 启动 Web 运行器
        let start_result = eframe::WebRunner::new()
            .start(
                canvas,                                                  // Canvas 元素
                web_options,                                             // Web 配置选项
                Box::new(|cc| Ok(Box::new(reasypub::MainApp::new(cc)))), // 创建应用实例的闭包
            )
            .await;

        // 移除加载文本和加载动画
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    // 启动成功，移除加载提示
                    loading_text.remove();
                }
                Err(e) => {
                    // 启动失败，显示错误信息
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
