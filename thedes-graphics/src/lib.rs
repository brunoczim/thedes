pub mod camera;
pub mod game_screen;
pub mod tile;
pub mod view;

#[derive(Debug, Clone)]
pub struct SessionData {
    pub selected_inventory_slot: usize,
}
