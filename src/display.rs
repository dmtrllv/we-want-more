#[allow(unused)]
#[derive(Debug)]
pub struct Display {
    pub id: DisplayId,
    pub x: i64,
    pub y: i64,
    pub width: u64,
    pub height: u64,
    pub scale: f32,
}

#[allow(unused)]
#[derive(Debug)]
pub struct DisplayId(pub String);