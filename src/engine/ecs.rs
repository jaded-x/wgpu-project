use std::cell::{RefCell, RefMut};

trait Component {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn push_none(&mut self);
}

impl<T: 'static> Component for RefCell<Vec<Option<T>>> {
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }

    fn push_none(&mut self) {
        self.get_mut().push(None);
    }
}

pub struct ECS {
    pub entity_count: usize,
    pub components: Vec<Box<dyn Component>>,
}

impl ECS {
    pub fn new() -> Self {
        Self {
            entity_count: 0,
            components: Vec::new()
        }
    }

    pub fn new_entity(&mut self) -> usize {
        let id = self.entity_count;
        for component in self.components.iter_mut() {
            component.push_none();
        }
        self.entity_count += 1;

        id
    }

    pub fn add_component<ComponentType: 'static>(&mut self, entity: usize, component: ComponentType) {
        for component_vec in self.components.iter_mut() {
            if let Some(component_vec) = component_vec
                .as_any_mut()
                .downcast_mut::<RefCell<Vec<Option<ComponentType>>>>()
            {
                component_vec.get_mut()[entity] = Some(component);
                return;
            }
        }

        let mut new_component_vec: Vec<Option<ComponentType>> = Vec::with_capacity(self.entity_count);
        for _ in 0..self.entity_count {
            new_component_vec.push(None);
        }

        new_component_vec[entity] = Some(component);
        self.components.push(Box::new(RefCell::new(new_component_vec)));
    }

    pub fn borrow_component_vec<ComponentType: 'static>(&self) -> Option<RefMut<Vec<Option<ComponentType>>>> {
        for component_vec in self.components.iter() {
            if let Some(component_vec) = component_vec
                .as_any()
                .downcast_ref::<RefCell<Vec<Option<ComponentType>>>>()
            {
                return Some(component_vec.borrow_mut());
            }
        }
        None
    }
}