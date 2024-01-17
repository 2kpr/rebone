use clap::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};

#[derive(Parser)]
struct Cli {
    input_prim: std::path::PathBuf,
    from_borg: std::path::PathBuf,
    to_borg: std::path::PathBuf,
    output_prim: std::path::PathBuf,
}

fn read_u8(buffer: &Vec<u8>, position: usize) -> u8 {
    u8::from_le_bytes(buffer[position..position + 1].try_into().unwrap())
}

fn read_u16(buffer: &Vec<u8>, position: usize) -> u16 {
    u16::from_le_bytes(buffer[position..position + 2].try_into().unwrap())
}

fn read_u32(buffer: &Vec<u8>, position: usize) -> u32 {
    u32::from_le_bytes(buffer[position..position + 4].try_into().unwrap())
}

struct Borg {
    buffer: Vec<u8>,
    header: usize,
    bones: Vec<String>,
    bones_map: HashMap<String, u8>,
}

impl Borg {
    fn new() -> Borg {
        Borg {
            buffer: Vec::new(),
            header: 0,
            bones: Vec::new(),
            bones_map: HashMap::new(),
        }
    }

    fn from_file(&mut self, path: &str) {
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(err) => {
                println!("Error opening file {}: {}", path, err);
                std::process::exit(1);
            }
        };
        self.buffer = vec![0 as u8; file.metadata().unwrap().len() as usize];
        Read::read(&mut file, &mut self.buffer).unwrap();
        self.header = read_u32(&self.buffer, 0) as usize;
        //println!("header: {:#08x}", self.header);
        let bone_count = read_u32(&self.buffer, self.header);
        //println!("bone_count: {:#08x}", bone_count);
        let bones_offset = read_u32(&self.buffer, self.header + 8) as usize;
        //println!("bones_offset: {:#08x}", bones_offset);
        for i in 0..bone_count {
            let offset = bones_offset + i as usize * 0x40 + 0x1C;
            let bone_name = String::from_utf8(self.buffer[offset..offset + 0x22].to_vec())
                .unwrap()
                .trim_matches(char::from(0))
                .to_string();
            self.bones.push(bone_name.clone());
            self.bones_map.insert(bone_name, i as u8);
        }
        //println!("bones: {:?}", self.bones);
        //for bone in &self.bones {
        //println!("{}", bone);
        //}
    }
}

struct Mesh {
    main_type: u16,
    sub_type: u8,
    sub_offset: usize,
    properties: u8,
    vertex_count: u32,
    vertex_offset: usize,
    weights_offset: usize,
}

impl Mesh {
    fn new() -> Mesh {
        Mesh {
            main_type: 0,
            sub_type: 0,
            sub_offset: 0,
            properties: 0,
            vertex_count: 0,
            vertex_offset: 0,
            weights_offset: 0,
        }
    }

    fn from_buffer(&mut self, buffer: &Vec<u8>, position: usize) {
        self.main_type = read_u16(buffer, position + 2);
        //println!("Main type: {}", self.main_type);
        self.sub_type = read_u8(buffer, position + 4);
        //println!("Sub type: {}", self.sub_type);
        self.sub_offset = read_u32(buffer, position + 0x2C) as usize;
        //println!("Sub offset: {:#08x}", self.sub_offset);
        self.sub_offset = read_u32(buffer, self.sub_offset) as usize;
        //println!("Sub offset: {:#08x}", self.sub_offset);
        self.properties = read_u8(buffer, self.sub_offset + 5);
        //println!("properties: {}", self.properties);
        self.vertex_count = read_u32(buffer, self.sub_offset + 0x2C);
        //println!("vertex_count: {:#08x}", self.vertex_count);
        self.vertex_offset = read_u32(buffer, self.sub_offset + 0x30) as usize;
        //println!("vertex_offset: {:#08x}", self.vertex_offset);
        if self.is_high_resolution() {
            self.weights_offset = self.vertex_offset + self.vertex_count as usize * 0xC;
        } else {
            self.weights_offset = self.vertex_offset + self.vertex_count as usize * 8;
        }
    }

    fn is_weighted_mesh(&self) -> bool {
        return self.main_type == 2 && self.sub_type == 2;
    }

    fn is_high_resolution(&self) -> bool {
        return self.properties & 8 == 8;
    }
}

struct Prim {
    buffer: Vec<u8>,
    header: usize,
    property_flags: u32,
    mesh: Vec<Mesh>,
}

impl Prim {
    fn new() -> Prim {
        Prim {
            buffer: Vec::new(),
            header: 0,
            property_flags: 0,
            mesh: Vec::new(),
        }
    }

    fn from_file(&mut self, path: &str) {
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(err) => {
                println!("Error opening file {}: {}", path, err);
                std::process::exit(1);
            }
        };
        self.buffer = vec![0 as u8; file.metadata().unwrap().len() as usize];
        Read::read(&mut file, &mut self.buffer).unwrap();
        self.header = read_u32(&self.buffer, 0) as usize;
        self.property_flags = read_u32(&self.buffer, self.header + 4);
        let mesh_count = read_u32(&self.buffer, self.header + 0xC) as usize;
        let mesh_table = read_u32(&self.buffer, self.header + 0x10) as usize;
        for i in 0..mesh_count {
            let mesh_offset = read_u32(&self.buffer, mesh_table + i * 4) as usize;
            let mut mesh = Mesh::new();
            mesh.from_buffer(&self.buffer, mesh_offset);
            if !mesh.is_weighted_mesh() {
                println!(
                    "Error: Mesh {} in PRIM file {} is not a weighted mesh!",
                    i, path
                );
                std::process::exit(0);
            }
            self.mesh.push(mesh);
        }
    }

    fn is_weighted(&self) -> bool {
        return self.property_flags & 8 == 8;
    }

    fn output_with_remap(&mut self, bone_remap: HashMap<u8, u8>, path: &str) -> Result<(), u8> {
        let mut buffer = self.buffer.clone();
        let mut remapped = 0;
        for mesh in &self.mesh {
            let mut offset = mesh.weights_offset;
            for _ in 0..mesh.vertex_count {
                offset += 4;
                for _ in 0..4 {
                    let joint = read_u8(&buffer, offset);
                    if !bone_remap.contains_key(&joint) {
                        return Err(joint);
                    }
                    if joint != bone_remap[&joint] {
                        remapped += 1;
                        //println!("Changed joint from {} to {}", joint, bone_remap[&joint]);
                        buffer[offset] = bone_remap[&joint];
                    }
                    offset += 1;
                }
                offset += 2;
                for _ in 0..2 {
                    let joint = read_u8(&buffer, offset);
                    if !bone_remap.contains_key(&joint) {
                        return Err(joint);
                    }
                    if joint != bone_remap[&joint] {
                        remapped += 1;
                        //println!("Changed joint from {} to {}", joint, bone_remap[&joint]);
                        buffer[offset] = bone_remap[&joint];
                    }
                    offset += 1;
                }
            }
        }
        let mut file = match File::create(path) {
            Ok(file) => file,
            Err(err) => {
                println!("Error creating file {}: {}", path, err);
                std::process::exit(1);
            }
        };
        Write::write_all(&mut file, buffer.as_slice()).unwrap();
        println!("Remapped {} joints in {} meshes", remapped, self.mesh.len());
        println!("Remapped PRIM file output to {}", path);
        Ok(())
    }
}

fn main() {
    let args = Cli::parse();
    let mut from_borg = Borg::new();
    from_borg.from_file(args.from_borg.to_str().unwrap());
    println!(
        "{} has {} bones",
        args.from_borg.file_name().unwrap().to_str().unwrap(),
        from_borg.bones.len()
    );
    let mut to_borg = Borg::new();
    to_borg.from_file(args.to_borg.to_str().unwrap());
    println!(
        "{} has {} bones",
        args.to_borg.file_name().unwrap().to_str().unwrap(),
        to_borg.bones.len()
    );
    let mut bones = Vec::new();
    for bone in &from_borg.bones {
        if !to_borg.bones_map.contains_key(bone) {
            bones.push(bone);
        }
    }
    if bones.len() > 0 {
        println!(
            "Unique bones in {}:",
            args.from_borg.file_name().unwrap().to_str().unwrap()
        );
        for bone in bones {
            println!("  - {}", bone);
        }
    }
    let mut bones = Vec::new();
    for bone in &to_borg.bones {
        if !from_borg.bones_map.contains_key(bone) {
            bones.push(bone);
        }
    }
    if bones.len() > 0 {
        println!(
            "Unique bones in {}:",
            args.to_borg.file_name().unwrap().to_str().unwrap()
        );
        for bone in bones {
            println!("  - {}", bone);
        }
    }
    let mut prim = Prim::new();
    prim.from_file(args.input_prim.to_str().unwrap());
    if !prim.is_weighted() {
        println!(
            "Error: PRIM file {} is not weighted!",
            args.input_prim.to_str().unwrap()
        );
        std::process::exit(0);
    }
    println!(
        "{} has {} meshes",
        args.input_prim.file_name().unwrap().to_str().unwrap(),
        prim.mesh.len()
    );
    let mut bone_remap = HashMap::new();
    for bone in &from_borg.bones_map {
        if to_borg.bones_map.contains_key(bone.0) {
            bone_remap.insert(*bone.1, to_borg.bones_map[bone.0]);
        }
    }
    match prim.output_with_remap(bone_remap, args.output_prim.to_str().unwrap()) {
        Ok(_) => (),
        Err(joint) => {
            println!(
                "Error: Vertex weighted to bone {} from BORG {} is not within BORG {}",
                from_borg.bones[joint as usize],
                args.from_borg.file_name().unwrap().to_str().unwrap(),
                args.to_borg.file_name().unwrap().to_str().unwrap(),
            );
            std::process::exit(0);
        }
    }
}
