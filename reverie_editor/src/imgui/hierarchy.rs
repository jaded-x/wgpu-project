use reverie::engine::{scene::Scene, components::{transform::Transform, name::Name, light::PointLight}};
use specs::{Entity, WorldExt, Join, Storage, shred::Fetch, storage::MaskedStorage, WriteStorage};

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
            let mut point_light_index = 0;
            let mut transforms = scene.world.write_component::<Transform>();
            for entity in scene.world.entities().join() {
                if transforms.get(entity).unwrap().data.parent == None {
                    self.create_node(ui, scene, explorer, entity, &mut point_light_index, &mut transforms);
                }
            }
        });
    }

    fn create_node<'a>(&mut self, ui: &'a imgui::Ui, scene: &Scene, explorer: &mut Explorer, entity: Entity, point_light_index: &mut usize, transforms: &mut WriteStorage<Transform>) {
        let names = scene.world.read_component::<Name>();
        let point_light_component = scene.world.read_component::<PointLight>();
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
                if let Some(payload) = ui.drag_drop_source_config("Object").begin_payload(entity) {
                    ui.text(name.0.as_str());
                    payload.end();
                }
                match ui.drag_drop_target() {
                    Some(target) => {
                        match target.accept_payload::<Entity, _>("Object", imgui::DragDropFlags::empty()) {
                            Some(Ok(payload_data)) => {
                                let child_transform = transforms.get_mut(payload_data.data);
                                let parent_option = child_transform.unwrap().data.parent;
                                if parent_option.is_some() {                  
                                    let old_parent = scene.world.entities().entity(parent_option.unwrap()).clone();
                                    let old_parent_transform = transforms.get_mut(old_parent).unwrap();
                                    old_parent_transform.data.children.retain(|&x| x != payload_data.data.id());
                                }
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
                if ui.is_item_clicked() && !ui.is_item_toggled_open() {
                    self.entity = Some(entity);
                    explorer.selected_file = None;
                    explorer.material = None;
                    match point_light_component.get(entity) {
                        Some(_) => self.point_light_index = Some(*point_light_index),
                        None => self.point_light_index = None,
                    }
                }

                for child in &transforms.get(entity).unwrap().data.children.clone() {
                    self.create_node(ui, scene, explorer, scene.world.entities().entity(*child), point_light_index, transforms);
                }
            });
        
        if !opened {
            if let Some(payload) = ui.drag_drop_source_config("Object").begin_payload(entity) {
                ui.text(name.0.as_str());
                payload.end();
            }
            match ui.drag_drop_target() {
                Some(target) => {
                    match target.accept_payload::<Entity, _>("Object", imgui::DragDropFlags::empty()) {
                        Some(Ok(payload_data)) => {
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
        

            if ui.is_item_clicked() && !ui.is_item_toggled_open() {
                self.entity = Some(entity);
                explorer.selected_file = None;
                explorer.material = None;
                match point_light_component.get(entity) {
                    Some(_) => self.point_light_index = Some(*point_light_index),
                    None => self.point_light_index = None,
                }
            }
        }

        if point_light_component.get(entity).is_some() {
            *point_light_index += 1;
        }
    }
}