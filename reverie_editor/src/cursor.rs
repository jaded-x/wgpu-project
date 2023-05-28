pub fn set_cursor(window: &winit::window::Window, ui: &imgui::Ui) {
    if ui.io().want_capture_mouse {
        let imgui_cursor = ui.mouse_cursor();

        let winit_cursor = match imgui_cursor {
            Some(imgui::MouseCursor::Arrow) => winit::window::CursorIcon::Arrow,
            Some(imgui::MouseCursor::TextInput) => winit::window::CursorIcon::Text,
            Some(imgui::MouseCursor::ResizeAll) => winit::window::CursorIcon::Move,
            Some(imgui::MouseCursor::ResizeNS) => winit::window::CursorIcon::NsResize,
            Some(imgui::MouseCursor::ResizeEW) => winit::window::CursorIcon::EwResize,
            Some(imgui::MouseCursor::ResizeNESW) => winit::window::CursorIcon::NeswResize,
            Some(imgui::MouseCursor::ResizeNWSE) => winit::window::CursorIcon::NwseResize,
            Some(imgui::MouseCursor::Hand) => winit::window::CursorIcon::Hand,
            _ => winit::window::CursorIcon::Default,
        };

        window.set_cursor_icon(winit_cursor);
    } else {
        window.set_cursor_icon(winit::window::CursorIcon::Arrow);
    }
}