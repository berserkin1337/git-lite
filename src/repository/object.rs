#[derive(Copy, Clone, Debug)]
pub enum ObjType {
    Commit,
    Tree,
    Tag,
    Blob,
}

pub struct GitObject {
    pub obj_type: ObjType,
    pub data: Vec<u8>,
}

pub trait Serializable {
    fn serialize(&self) -> &[u8];
    fn deserialize(input: &[u8]) -> Self;
}

impl Serializable for GitObject {
    fn serialize(&self) -> &[u8] {
        match self.obj_type {
            ObjType::Commit => &self.data,
            ObjType::Tree => &self.data,
            ObjType::Tag => &self.data,
            ObjType::Blob => &self.data,
        }
    }

    fn deserialize(_input: &[u8]) -> Self {
        unimplemented!()
    }
}

impl Serializable for ObjType {
    fn serialize(&self) -> &[u8] {
        match *self {
            ObjType::Commit => b"commit",
            ObjType::Tree => b"tree",
            ObjType::Tag => b"tag",
            ObjType::Blob => b"blob",
        }
    }

    fn deserialize(input: &[u8]) -> ObjType {
        match input {
            b"commit" => ObjType::Commit,
            b"tree" => ObjType::Tree,
            b"tag" => ObjType::Tag,
            b"blob" => ObjType::Blob,
            _ => ObjType::Commit,
        }
    }
}

impl GitObject {
    pub fn new(obj_type: ObjType, data: &[u8]) -> GitObject {
        GitObject {
            obj_type,
            data: data.to_vec(),
        }
    }
}
