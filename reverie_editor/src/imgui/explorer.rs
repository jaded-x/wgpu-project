use std::path::PathBuf;

use reverie::engine::model::Material;

pub struct Explorer {
    current_folder: PathBuf,
    is_first_frame: bool,
    text_input: String,
    pub selected_file: Option<PathBuf>,
    pub material: Option<Material>,
}

impl Explorer {
    pub fn new() -> Self {
        Self {
            current_folder: std::env::current_dir().unwrap().join("res"),
            is_first_frame: true,
            text_input: String::new(),
            selected_file: None,
            material: None,
        }
    }

    pub fn ui<'a>(&mut self, ui: &'a imgui::Ui) {
        ui.window("Explorer").build(|| {
            ui.columns(2, "explorer_split", true);
            if self.is_first_frame {
                ui.set_column_width(0, 100.0);
                self.is_first_frame = false;
            }
            self.create_node(ui, std::env::current_dir().unwrap().join("res"), 0);

            ui.next_column();
            ui.popup("explorer_popup", || {
                ui.input_text("##material name", &mut self.text_input).build();
                if ui.button("Create Material") {
                    Material::create(&self.current_folder, &self.text_input);
                    self.text_input = String::new();
                }
            });

            ui.child_window("child").build(|| {
                ui.columns((ui.content_region_avail()[0] / 80.0) as i32, "content split", false);
                self.get_files(ui);
            });

            if ui.is_item_hovered() && ui.is_mouse_clicked(imgui::MouseButton::Right) {
                ui.open_popup("explorer_popup");
            }

        });
    }

    fn get_inner_dirs<'a>(&mut self, ui: &'a imgui::Ui, path: PathBuf, depth: i32) {
        match std::fs::read_dir(path) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            if entry.file_type().unwrap().is_dir() {
                                self.create_node(ui, entry.path(), depth)
                            }
                        },
                        Err(_) => {}
                    }
                }
            },
            Err(_) => {},
        }
    }

    fn create_node<'a>(&mut self, ui: &'a imgui::Ui, path: PathBuf, depth: i32) {
        let indent = ui.clone_style().indent_spacing;
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let mut flags = imgui::TreeNodeFlags::OPEN_ON_ARROW | imgui::TreeNodeFlags::ALLOW_ITEM_OVERLAP;
        if self.current_folder == path {
            flags = imgui::TreeNodeFlags::OPEN_ON_ARROW | imgui::TreeNodeFlags::ALLOW_ITEM_OVERLAP | imgui::TreeNodeFlags::SELECTED;
        }
        if ui.tree_node_config(&file_name).default_open(true).flags(flags).build(|| {
            ui.same_line_with_pos(28.0 + indent * depth as f32);
            if ui.invisible_button(format!("##{}", &file_name), [ui.content_region_avail()[0], ui.calc_text_size(file_name)[1]]) {
                self.current_folder = path.clone();
            }
            self.get_inner_dirs(ui, path.clone(), depth + 1);
        }).is_none() {
            ui.same_line_with_pos(28.0 + indent * depth as f32);
            if ui.invisible_button(format!("##{}", &file_name), [ui.content_region_avail()[0], ui.calc_text_size(file_name)[1]]) {
                self.current_folder = path.clone();
            }
        }
    }

    fn get_files<'a>(&mut self, ui: &'a imgui::Ui) {
        match std::fs::read_dir(self.current_folder.clone()) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            if entry.file_type().unwrap().is_dir() {
                                ui.image_button(entry.file_name().to_str().unwrap(), imgui::TextureId::new(3), [64.0, 64.0]);
                                if ui.is_item_hovered() && ui.is_mouse_double_clicked(imgui::MouseButton::Left) {
                                    self.current_folder = entry.path();
                                }
                                
                                ui.spacing();
                                ui.text(entry.file_name().to_str().unwrap());
                                
                                ui.next_column();
                            }
                            if entry.file_type().unwrap().is_file() {
                                if ui.image_button(entry.file_name().to_str().unwrap(), imgui::TextureId::new(4), [64.0, 64.0]) {
                                    self.selected_file = Some(entry.path());
                                    self.material = Some(Material::load(&entry.path()));
                                }
                                
                                if let Some(payload) = ui.drag_drop_source_config("texture").begin_payload(0) {
                                    ui.text(entry.file_name().to_str().unwrap());
                                    payload.end();
                                }
                                ui.spacing();
                                ui.text(entry.file_name().to_str().unwrap());

                                ui.next_column();
                            }
                        },
                        Err(_) => {}
                    }
                }
            },
            Err(_) => {},
        }
    }
}