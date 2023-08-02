use reverie::engine::{scene::Scene, components::{transform::Transform, name::Name, light::PointLight}};
use specs::{Entity, WorldExt, Join};

use super::explorer::Explorer;

pub struct Hierarchy {
    pub entity: Option<Entity>,
    pub point_light_index: Option<usize>,
}

impl Hierarchy {
    pub fn new() -> Self {
        Self {
            entity: None,
            point_light_index: None,
        }
    }

    pub fn ui<'a>(&mut self, ui: &'a imgui::Ui, scene: &mut Scene, explorer: &mut Explorer) {
        ui.window("Hierarchy").build(|| {
            let names = scene.world.read_component::<Name>();
            let point_light_component = scene.world.read_component::<PointLight>();
            
            let mut point_light_index = 0;
            for entity in scene.world.entities().join() {
                let mut transforms = scene.world.write_component::<Transform>();
                let parent_transform = transforms.get_mut(entity).unwrap();
                if parent_transform.data.parent == None {
                    let name = names.get(entity).unwrap();

                    let mut flags = imgui::TreeNodeFlags::DEFAULT_OPEN | 
                        imgui::TreeNodeFlags::FRAME_PADDING | 
                        imgui::TreeNodeFlags::OPEN_ON_ARROW | 
                        imgui::TreeNodeFlags::SPAN_AVAIL_WIDTH;

                    if self.entity == Some(entity) {
                        flags |= imgui::TreeNodeFlags::SELECTED
                    }

                    let mut opened = false;
                    ui.tree_node_config(name.0.as_str())
                        .flags(flags)
                        .build(|| {
                            opened = true;
                            if ui.is_item_clicked() && !ui.is_item_toggled_open() {
                                self.entity = Some(entity);
                                explorer.selected_file = None;
                                explorer.material = None;
                                match point_light_component.get(entity) {
                                    Some(_) => self.point_light_index = Some(point_light_index),
                                    None => self.point_light_index = None,
                                }
                            }
                            for child in &parent_transform.data.children {
                                ui.tree_node_config(names.get(scene.world.entities().entity(*child)).unwrap().0.as_str())
                                    .flags(flags)
                                    .build(||{});
                            }
                        });
                    
                    drop(&parent_transform);
                    drop(transforms);

                    if let Some(payload) = ui.drag_drop_source_config("Object").begin_payload(entity) {
                        ui.text(name.0.as_str());
                        payload.end();
                    }

                    match ui.drag_drop_target() {
                        Some(target) => {
                            match target.accept_payload::<Entity, _>("Object", imgui::DragDropFlags::empty()) {
                                Some(Ok(payload_data)) => {
                                    let mut transforms = scene.world.write_component::<Transform>();
                                    let child_transform = transforms.get_mut(payload_data.data);
                                    child_transform.unwrap().data.parent = Some(entity.id());
                                    let parent_transform = transforms.get_mut(entity);
                                    parent_transform.unwrap().data.children.push(payload_data.data.id());
                                },
                                Some(Err(e)) => {
                                    println!("{}", e);
                                },
                                _ => {},
                            }
                        }
                        _ => {},
                    }

                    if ui.is_item_clicked() && !ui.is_item_toggled_open() && !opened {
                        self.entity = Some(entity);
                        explorer.selected_file = None;
                        explorer.material = None;
                        match point_light_component.get(entity) {
                            Some(_) => self.point_light_index = Some(point_light_index),
                            None => self.point_light_index = None,
                        }
                    }

                    if point_light_component.get(entity).is_some() {
                        point_light_index += 1;
                    }
                }
            }
        });
    }
}