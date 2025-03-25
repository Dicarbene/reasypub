use crate::epubworker::{self, txt_build};
use crate::{ConversionMethod, ImageFileReader, PanelIndex, Pattern, TextFileReader, TextProcessor};
use bytes::Bytes;
use rfd::FileDialog;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct MainApp {
    //基础数据结构体
    // Example stuff:
    label: String,
    input_txt_path: String,
    input_image_path: String,
    book_name: String,
    author: String,
    // 转换配置
    available_methods: Vec<ConversionMethod>, // 使用枚举
    selected_method: ConversionMethod,        // 当前选中方法
    available_panels: Vec<PanelIndex>,
    panel_index: PanelIndex,
    cover_path: String,
    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
    #[serde(skip)]
    input_file: TextFileReader,
    #[serde(skip)]
    input_image: ImageFileReader,
}

impl Default for MainApp {
    //基础数据初始化
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            input_txt_path: "txt转换epub, 选择文件".to_owned(),
            input_image_path: "epub封面，选择文件".to_owned(),
            book_name: "输入书名".to_owned(),
            author: "输入作者".to_owned(),
            value: 2.7,
            available_methods: vec![
                ConversionMethod::SimpleRules,
                ConversionMethod::Regex,
                ConversionMethod::CustomConfig,
            ],
            selected_method: ConversionMethod::Regex,
            available_panels: vec![
                PanelIndex::Chapter,
                PanelIndex::Format,
                PanelIndex::Font,
                PanelIndex::PublishInfo,
                PanelIndex::CSS,
                PanelIndex::Images,
                PanelIndex::Misc,
            ],
            panel_index: PanelIndex::Chapter,
            cover_path: "输入封面图片地址".to_owned(),
            input_file: TextFileReader {
                content: "empty".to_owned(),
                path: None,
                error: None,
            },
            input_image: ImageFileReader {
                content: Bytes::new(),
                path: None,
                error: None,
                texture: None,
            },
        }
    }
}

impl MainApp {
    /// 在第一帧被调用一次
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        // 创建字体配置
        use std::sync::Arc;
        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "stdg".to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../assets/stdg-regular.ttf"
            ))), // 注意这里补全了括号
        );

        // 设置字体
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "stdg".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
/*         if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
 */
        Default::default()
    }
}

impl eframe::App for MainApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// 每帧更新都调用
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("菜单     ", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.allocate_ui_with_layout(
                egui::Vec2::new(700.0, 50.0),
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    ui.heading("Reasypub 电子书转换");
                },
            );
            //文本文件
            readtxt(ui, &mut self.input_file, &mut self.input_txt_path);
            ui.horizontal(|ui| {
                if ui.button("🖻 修改封面").clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("图片", &["jpeg", "png", "webp", "jpg"])
                        .pick_file()
                    {
                        match std::fs::metadata(&path) {
                            Ok(metadata) => {
                                // 检查文件大小（示例：限制 10MB）
                                if metadata.len() > 10 * 1024 * 1024 {
                                    self.input_image.error =
                                        Some("❌ 文件过大（超过 10MB）".to_string());
                                    return;
                                }
                            }
                            Err(err) => {
                                self.input_image.error =
                                    Some(format!("❌ 获取文件元数据失败: {}", err));
                                return;
                            }
                        }
                        match std::fs::read(&path) {
                            Ok(content) => {
                                self.input_image.content = Bytes::from_owner(content);
                                self.input_image.error = None;
                                self.input_image.path = Some(path.clone());
                                self.input_image_path = path.to_str().unwrap().to_string();
                                self.input_image.texture = None;
                            }
                            Err(e) => {
                                if let Some(s) = path.to_str() {
                                    self.input_image_path = s.to_string();
                                } else {
                                    self.input_image.error = Some(format!("❌ 读取失败: {}", e));
                                }
                            }
                        }
                    }
                }
                if self.input_image.error.is_none() {
                    ui.label(&self.input_image_path);
                }
                if let Some(err) = &self.input_image.error {
                    ui.label(egui::RichText::new(err).color(egui::Color32::RED));
                }
            });

            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.label("书名 ");
                ui.text_edit_singleline(&mut self.book_name);
                ui.label("作者 ");
                ui.text_edit_singleline(&mut self.author);
            });
            ui.horizontal(|ui| {
                if ui.button("编辑txt文件").clicked() {};
                if ui.button("开始转换").clicked() {
                    match &self.selected_method {
                        ConversionMethod::Regex => {
                            
                            let tasker: TextProcessor = TextProcessor::new(
                                Pattern::ChineseChapter,
                                self.input_file.content.clone(),
                            );
                            let split_result = tasker.split_by_pattern();
                            let _ = txt_build(&split_result,Pattern::ChineseChapter);
                        }
                        ConversionMethod::CustomConfig => todo!(),
                        ConversionMethod::SimpleRules => todo!(),
                    }
                };
            });
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 70.0;
                if ui.button("章节").clicked() {
                    self.panel_index = PanelIndex::Chapter;
                };
                if ui.button("版式").clicked() {
                    self.panel_index = PanelIndex::Format;
                };
                if ui.button("字体").clicked() {
                    self.panel_index = PanelIndex::Font;
                };
                if ui.button("出版信息").clicked() {
                    self.panel_index = PanelIndex::PublishInfo;
                };
                if ui.button("CSS和HTML").clicked() {
                    self.panel_index = PanelIndex::CSS;
                };
                if ui.button("插图").clicked() {
                    self.panel_index = PanelIndex::Images;
                };
                if ui.button("杂项").clicked() {
                    self.panel_index = PanelIndex::Misc;
                };
            });

            ui.separator();
            //各面板切换配置
            match &self.panel_index {
                PanelIndex::Chapter => {
                    ui.vertical(|ui| {
                        ui.label("选择转换方法:");
                        for method in &self.available_methods {
                            match method {
                                ConversionMethod::CustomConfig => {
                                    //自定义正则
                                    ui.vertical(|ui| {
                                        ui.radio_value(
                                            &mut self.selected_method,
                                            method.clone(),
                                            method.to_string(), // 使用Display trait
                                        );
                                    });
                                }
                                ConversionMethod::Regex => {
                                    //正则选择
                                    ui.radio_value(
                                        &mut self.selected_method,
                                        method.clone(),
                                        method.to_string(),
                                    );
                                }
                                ConversionMethod::SimpleRules => {
                                    // 简单规则
                                    ui.radio_value(
                                        &mut self.selected_method,
                                        method.clone(),
                                        method.to_string(),
                                    );
                                }
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.button("章节编辑").clicked() {};
                    });
                }
                PanelIndex::Format => {}
                PanelIndex::CSS => {}
                PanelIndex::Font => {}
                PanelIndex::Images => {}
                PanelIndex::PublishInfo => {}
                PanelIndex::Misc => {}
            }

            // 底部信息
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                ui.add(egui::github_link_file!(
                    "https://github.com/Dicarbene/reasypub/",
                    "源代码 Made by Dicarbene 期望star⭐"
                ));
                egui::warn_if_debug_build(ui);
            });
        });
        egui::SidePanel::right("my_right_panel").show(ctx, |ui| match &self.input_image.error {
            None => {
                ui.label("封面图片：");
                show_image_ui(ui, &mut self.input_image);
            }
            Some(t) => {
                ui.label(t);
                ui.add(
                    egui::Image::new(egui::include_image!("../assets/icon-256.png"))
                        .max_width(200.0)
                        .rounding(10.0),
                );
            }
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}

fn readtxt(ui: &mut egui::Ui, input_txt: &mut TextFileReader, input_txt_path: &mut String) {
    ui.horizontal(|ui| {
        if ui.button("📂 打开文本文件").clicked() {
            if let Some(path) = FileDialog::new()
                .add_filter("文本文件", &["txt"])
                .pick_file()
            {
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        input_txt.content = content;
                        input_txt.error = None;
                        input_txt.path = Some(path.clone());
                        //txt路径显示
                        *input_txt_path = path.to_str().unwrap().to_string();
                    }
                    Err(e) => {
                        input_txt.error = Some(format!("❌ 读取失败: {}", e));
                    }
                }
            }
        }

        // 显示错误信息
        if let Some(err) = &input_txt.error {
            *input_txt_path = err.clone();
        }
        if input_txt.error.is_none() {
            ui.label(input_txt_path.clone());
        } else {
            ui.label(egui::RichText::new(input_txt_path.clone()).color(egui::Color32::RED));
        }
    });
}

fn show_image_ui(ui: &mut egui::Ui, reader: &mut ImageFileReader) {
    // 固定宽高的容器
    let desired_size = egui::vec2(300.0, 200.0); // 设置宽高

    // 带边框的容器
    let frame = egui::Frame::none().inner_margin(4.0).rounding(4.0);

    frame.show(ui, |ui| {
        // 更新纹理
        reader.update_texture(ui.ctx());

        if let Some(texture) = &reader.texture {
            // 显示图片（自动缩放填充容器）
            ui.add(egui::Image::new(texture).max_width(200.0).rounding(10.0));
        } else if let Some(err) = &reader.error {
            // 显示错误信息
            ui.colored_label(egui::Color32::RED, err);
        } else {
            // 显示占位符
            ui.label("🖻 暂无图片");
        }
    });
}
