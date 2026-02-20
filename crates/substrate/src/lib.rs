pub mod arena;

pub struct DrawCommand {
    _mesh_id: u64,
    material_id: u64,
}

pub fn build_frame_draw_list() -> Vec<DrawCommand> {
    let mut v: Vec<DrawCommand> = Vec::new();
    for i in 0..100 {
        v.push(DrawCommand { _mesh_id: i, material_id: (100 - i) % 5 });
    }
    v.sort_by_key(|dc| dc.material_id);
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = build_frame_draw_list();
        assert_eq!(100, result.len());
    }
}
