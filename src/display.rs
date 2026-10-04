#[allow(unused)]
#[derive(Debug)]
pub struct Display {
    pub id: DisplayId,
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
    pub scale: f32,
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayId(pub String);