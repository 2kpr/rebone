use clap::Parser;
use eframe::egui;
use egui::{FontFamily, FontId, TextStyle};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct HashDepend {
    hash: String,
    flag: String,
}

#[derive(Serialize, Deserialize)]
struct Meta {
    hash_value: String,
    hash_path: String,
    hash_offset: u32,
    hash_size: u32,
    hash_resource_type: String,
    hash_reference_table_size: u32,
    hash_reference_table_dummy: u32,
    hash_size_final: u32,
    hash_size_in_memory: u32,
    hash_size_in_video_memory: u32,
    hash_reference_data: Vec<HashDepend>,
}

#[derive(Default)]
struct MyApp {
    input_prim_path: String,
    from_borg_path: String,
    to_borg_path: String,
    output_prim_path: String,
    rebone: Rebone,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>, rebone: Rebone) -> Self {
        configure_text_styles(&cc.egui_ctx);
        Self {
            rebone: rebone,
            input_prim_path: String::new(),
            from_borg_path: String::new(),
            to_borg_path: String::new(),
            output_prim_path: String::new(),
        }
    }
}

fn configure_text_styles(ctx: &egui::Context) {
    use FontFamily::{Monospace, Proportional};

    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (TextStyle::Heading, FontId::new(42.0, Proportional)),
        (
            TextStyle::Name("Heading2".into()),
            FontId::new(22.0, Proportional),
        ),
        (
            TextStyle::Name("ContextHeading".into()),
            FontId::new(19.0, Proportional),
        ),
        (TextStyle::Body, FontId::new(16.0, Proportional)),
        (TextStyle::Monospace, FontId::new(12.0, Monospace)),
        (TextStyle::Button, FontId::new(16.0, Proportional)),
        (TextStyle::Small, FontId::new(8.0, Proportional)),
    ]
    .into();
    ctx.set_style(style);
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Rebone");
                ui.label("Select input and output files and click Rebone!");
                ui.label("CLI Usage: rebone.exe <input_prim> <from_borg> <to_borg> <output_prim>");
            });

            ui.separator();

            if ui.button("Select Input PRIM file…").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.input_prim_path = path.display().to_string();
                }
            }
            ui.label("Input PRIM file:");
            ui.add_sized(
                [640.0, 0.0],
                egui::TextEdit::singleline(&mut self.input_prim_path),
            );
            ui.add_space(10.0);

            if ui.button("Select From BORG file…").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.from_borg_path = path.display().to_string();
                }
            }
            ui.label("From BORG file:");
            ui.add_sized(
                [640.0, 0.0],
                egui::TextEdit::singleline(&mut self.from_borg_path),
            );
            ui.add_space(10.0);

            if ui.button("Select To BORG file…").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.to_borg_path = path.display().to_string();
                }
            }
            ui.label("To BORG file:");
            ui.add_sized(
                [640.0, 0.0],
                egui::TextEdit::singleline(&mut self.to_borg_path),
            );
            ui.add_space(10.0);

            if ui.button("Select Output PRIM file…").clicked() {
                if let Some(path) = rfd::FileDialog::new().save_file() {
                    self.output_prim_path = path.display().to_string();
                }
            }
            ui.label("Output PRIM file:");
            ui.add_sized(
                [640.0, 0.0],
                egui::TextEdit::singleline(&mut self.output_prim_path),
            );

            ui.separator();

            ui.vertical_centered(|ui| {
                let button = ui.add_sized([100.0, 40.0], egui::Button::new("Rebone!"));
                if button.clicked() {
                    self.rebone.input_prim_path = PathBuf::from(&self.input_prim_path);
                    self.rebone.from_borg_path = PathBuf::from(&self.from_borg_path);
                    self.rebone.to_borg_path = PathBuf::from(&self.to_borg_path);
                    self.rebone.output_prim_path = PathBuf::from(&self.output_prim_path);
                    self.rebone.execute();
                }
            });
        });
    }
}

#[derive(Parser)]
struct Cli {
    input_prim: Option<PathBuf>,
    from_borg: Option<PathBuf>,
    to_borg: Option<PathBuf>,
    output_prim: Option<PathBuf>,
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

fn read_u64(buffer: &Vec<u8>, position: usize) -> u64 {
    u64::from_le_bytes(buffer[position..position + 8].try_into().unwrap())
}

fn write_u64(buffer: &mut Vec<u8>, position: usize, value: u64) {
    let bytes = u64::to_le_bytes(value);
    for i in 0..8 {
        buffer[position + i] = bytes[i];
    }
}

fn find_file(dir: &PathBuf, search: &str) -> Option<PathBuf> {
    let files = std::fs::read_dir(dir.parent().unwrap()).unwrap();
    for file in files {
        if file
            .as_ref()
            .unwrap()
            .file_name()
            .to_str()
            .unwrap()
            .to_lowercase()
            == search
        {
            return Some(PathBuf::from(&file.unwrap().path()));
        }
    }
    None
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
        println!("Remapped PRIM file output to:\n  - {}", path);
        Ok(())
    }
}
#[derive(Default)]
struct Rebone {
    gui: bool,
    input_prim_path: PathBuf,
    from_borg_path: PathBuf,
    to_borg_path: PathBuf,
    output_prim_path: PathBuf,
}

impl Rebone {
    fn new() -> Rebone {
        Rebone {
            gui: false,
            input_prim_path: PathBuf::new(),
            from_borg_path: PathBuf::new(),
            to_borg_path: PathBuf::new(),
            output_prim_path: PathBuf::new(),
        }
    }

    fn execute(&mut self) {
        let mut from_borg = Borg::new();
        from_borg.from_file(self.from_borg_path.to_str().unwrap());
        println!(
            "{} has {} bones",
            self.from_borg_path.file_name().unwrap().to_str().unwrap(),
            from_borg.bones.len()
        );
        let mut to_borg = Borg::new();
        to_borg.from_file(self.to_borg_path.to_str().unwrap());
        println!(
            "{} has {} bones",
            self.to_borg_path.file_name().unwrap().to_str().unwrap(),
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
                self.from_borg_path.file_name().unwrap().to_str().unwrap()
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
                self.to_borg_path.file_name().unwrap().to_str().unwrap()
            );
            for bone in bones {
                println!("  - {}", bone);
            }
        }
        let mut prim = Prim::new();
        prim.from_file(self.input_prim_path.to_str().unwrap());
        if !prim.is_weighted() {
            println!(
                "Error: PRIM file {} is not weighted!",
                self.input_prim_path.to_str().unwrap()
            );
            std::process::exit(0);
        }
        println!(
            "{} has {} meshes",
            self.input_prim_path.file_name().unwrap().to_str().unwrap(),
            prim.mesh.len()
        );
        let prim_meta_path = find_file(
            &self.input_prim_path,
            (self
                .input_prim_path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_lowercase()
                + ".meta.json")
                .as_str(),
        );
        if prim_meta_path.is_some() {
            let prim_meta_path = prim_meta_path.unwrap();
            let meta_json_string = match std::fs::read_to_string(&prim_meta_path) {
                Ok(meta_json_string) => meta_json_string,
                Err(err) => {
                    println!(
                        "Error opening file {}: {}",
                        prim_meta_path.to_str().unwrap(),
                        err
                    );
                    std::process::exit(1);
                }
            };
            let mut meta: Meta = match serde_json::from_str(&meta_json_string) {
                Ok(meta) => meta,
                Err(err) => {
                    println!(
                        "Error parsing meta json file {}: {}",
                        prim_meta_path.to_str().unwrap(),
                        err
                    );
                    std::process::exit(1);
                }
            };
            if meta.hash_reference_data.len() > 0 {
                let borg_hash = self
                    .to_borg_path
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                if borg_hash.starts_with("00") && borg_hash.len() == 16 {
                    meta.hash_reference_data[0].hash = borg_hash;
                }
            }
            let prim_meta_path =
                PathBuf::from(self.output_prim_path.to_str().unwrap().to_string() + ".meta.json");
            let file = match File::create(&prim_meta_path) {
                Ok(file) => file,
                Err(err) => {
                    println!(
                        "Error creating file {}: {}",
                        prim_meta_path.to_str().unwrap(),
                        err
                    );
                    std::process::exit(1);
                }
            };
            match serde_json::to_writer_pretty(&file, &meta) {
                Ok(_) => println!(
                    "Modified input PRIM meta json output to:\n  - {}",
                    prim_meta_path.to_str().unwrap()
                ),
                Err(_) => println!(
                    "Error writing modified input PRIM meta json output to:\n  - {}",
                    prim_meta_path.to_str().unwrap()
                ),
            };
        } else {
            let prim_meta_path = find_file(
                &self.input_prim_path,
                (self
                    .input_prim_path
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_lowercase()
                    + ".meta")
                    .as_str(),
            );
            if prim_meta_path.is_some() {
                let prim_meta_path = prim_meta_path.unwrap();
                let mut file = match File::open(&prim_meta_path) {
                    Ok(file) => file,
                    Err(err) => {
                        println!(
                            "Error opening file {}: {}",
                            prim_meta_path.to_str().unwrap(),
                            err
                        );
                        std::process::exit(1);
                    }
                };
                let mut buffer = vec![0 as u8; file.metadata().unwrap().len() as usize];
                Read::read(&mut file, &mut buffer).unwrap();
                let depends_size = read_u32(&buffer, 0x18) as usize;
                if depends_size > 0 {
                    let depends_count = (read_u32(&buffer, 0x2C) & 0x3FFFFFFF) as usize;
                    //println!("depends_count: {}", depends_count);
                    let borg_hash = u64::from_str_radix(
                        self.to_borg_path.file_stem().unwrap().to_str().unwrap(),
                        16,
                    )
                    .unwrap();
                    write_u64(&mut buffer, 0x30 + depends_count, borg_hash);
                    let prim_meta_path = PathBuf::from(
                        self.output_prim_path.to_str().unwrap().to_string() + ".meta",
                    );
                    let mut file = match File::create(&prim_meta_path) {
                        Ok(file) => file,
                        Err(err) => {
                            println!(
                                "Error creating file {}: {}",
                                prim_meta_path.to_str().unwrap(),
                                err
                            );
                            std::process::exit(1);
                        }
                    };
                    match Write::write_all(&mut file, buffer.as_slice()) {
                        Ok(_) => println!(
                            "Modified input PRIM meta output to:\n  - {}",
                            prim_meta_path.to_str().unwrap()
                        ),
                        Err(_) => println!(
                            "Error writing modified input PRIM meta output to:\n  - {}",
                            prim_meta_path.to_str().unwrap()
                        ),
                    }
                }
            }
        }
        let mut bone_remap = HashMap::new();
        for bone in &from_borg.bones_map {
            if to_borg.bones_map.contains_key(bone.0) {
                bone_remap.insert(*bone.1, to_borg.bones_map[bone.0]);
            }
        }
        match prim.output_with_remap(bone_remap, self.output_prim_path.to_str().unwrap()) {
            Ok(_) => (),
            Err(joint) => {
                println!(
                    "Error: Vertex weighted to bone {} from BORG {} is not within BORG {}",
                    from_borg.bones[joint as usize],
                    self.from_borg_path.file_name().unwrap().to_str().unwrap(),
                    self.to_borg_path.file_name().unwrap().to_str().unwrap(),
                );
                std::process::exit(0);
            }
        }
    }

    fn process_args(&mut self, args: &Cli) {
        if args.input_prim.is_none()
            || args.from_borg.is_none()
            || args.to_borg.is_none()
            || args.output_prim.is_none()
        {
            self.gui = true;
            return;
        }
        self.input_prim_path = args.input_prim.clone().unwrap_or_else(|| PathBuf::new());
        self.from_borg_path = args.from_borg.clone().unwrap_or_else(|| PathBuf::new());
        self.to_borg_path = args.to_borg.clone().unwrap_or_else(|| PathBuf::new());
        self.output_prim_path = args.output_prim.clone().unwrap_or_else(|| PathBuf::new());
    }
}

fn main() {
    let args = Cli::parse();
    let mut rebone = Rebone::new();
    rebone.process_args(&args);
    if !rebone.gui {
        rebone.execute();
    } else {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
            ..Default::default()
        };
        eframe::run_native(
            "Rebone v0.3.0",
            options,
            Box::new(|_cc| Box::new(MyApp::new(_cc, rebone))),
        )
        .unwrap();
    }
}
