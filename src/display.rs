#[derive(Debug)]
pub struct Display {
    id: DisplayId,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    scale: f32,
}

#[derive(Debug)]
pub struct DisplayId(pub usize);