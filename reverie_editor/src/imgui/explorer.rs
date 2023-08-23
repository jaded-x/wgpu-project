use std::{path::PathBuf, sync::Arc};

use reverie::engine::{registry::{Registry, AssetType}, gpu::Gpu, asset::material::Material};

pub struct Explorer {
    current_folder: PathBuf,
    is_first_frame: bool,
    text_input: String,
    pub selected_file: Option<PathBuf>,
    pub material: Option<Arc<Gpu<Material>>>,
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

    pub fn ui<'a>(&mut self, ui: &'a imgui::Ui, registry: &mut Registry) {
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
                if ui.button("Create Scene") {
                    let file_name = format!("{}{}", &self.text_input, ".revscene");
                    std::fs::write(&self.current_folder.join(file_name), "").unwrap();
                }
            });

            ui.child_window("child").build(|| {
                ui.columns((ui.content_region_avail()[0] / 80.0) as i32, "content split", false);
                self.get_files(ui, registry);
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
                                self.create_node(ui, entry.path(), depth);
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
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let mut flags = imgui::TreeNodeFlags::DEFAULT_OPEN | imgui::TreeNodeFlags::OPEN_ON_ARROW | imgui::TreeNodeFlags::SPAN_AVAIL_WIDTH;
        if self.current_folder == path {
            flags |= imgui::TreeNodeFlags::SELECTED;
        }
        let mut opened = false;
        ui.tree_node_config(&file_name).flags(flags).build(|| {
            opened = true;
            if ui.is_item_clicked() && !ui.is_item_toggled_open() {
                self.current_folder = path.clone();
            }
            self.get_inner_dirs(ui, path.clone(), depth + 1);
        });
        if ui.is_item_clicked() && !ui.is_item_toggled_open() && !opened {
            self.current_folder = path.clone();
        }
    }

    fn get_files<'a>(&mut self, ui: &'a imgui::Ui, registry: &mut Registry) {
        match std::fs::read_dir(self.current_folder.clone()) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            if entry.file_type().unwrap().is_dir() {
                                ui.image_button(entry.file_name().to_str().unwrap(), imgui::TextureId::new(11893785222860336258), [64.0, 64.0]);
                                if ui.is_item_hovered() && ui.is_mouse_double_clicked(imgui::MouseButton::Left) {
                                    self.current_folder = entry.path();
                                }
                                
                                ui.spacing();
                                ui.text(entry.file_name().to_str().unwrap());
                                
                                ui.next_column();
                            }
                            if entry.file_type().unwrap().is_file() {
                                let mut texture_id = registry.get_id(entry.path());
                                if registry.metadata.get(&texture_id).unwrap().asset_type == AssetType::Texture {
                                    let texture = registry.get_texture(texture_id, false);
                                } else {
                                    texture_id = 7403896815389001851;
                                }
                                

                                if ui.image_button(entry.file_name().to_str().unwrap(), imgui::TextureId::new(texture_id), [64.0, 64.0]) {
                                    self.selected_file = Some(entry.path());
                                    if let Some(extension) = entry.path().extension() {
                                        if AssetType::from_extension(extension) == AssetType::Material {
                                            self.material = Some(registry.get_material_from_path(entry.path()).unwrap());
                                        }
                                    }
                                }

                                if let Some(payload) = ui.drag_drop_source_config(AssetType::from_extension(entry.path().extension().unwrap()).to_string()).begin_payload(Some(registry.get_id(entry.path()))) {
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