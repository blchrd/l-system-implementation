use image::{Rgba, RgbaImage};
use rand::Rng;
use imageproc::{self, drawing::draw_antialiased_line_segment};
use serde::{Serialize, Deserialize};
use std::{collections::HashMap, f32::consts::PI, fs::File, io::Read};

struct Parameters {
    file: String,
    json: String,
    output: String,
}

fn get_arguments() -> Parameters {
    let args: Vec<_> = std::env::args().collect();
    let mut params = Parameters {
        file: "".to_string(),
        json: "".to_string(),
        output: "./render/test.png".to_string(),
    };

    if args.len() < 3 {
        return params;
    }

    if args[1] == "-f" {
        params.file = args[2].clone();
    } else if args[1] == "-j" {
        params.json = args[2].clone();
    }

    if args.contains(&"-o".to_string()) {
        let index = args.iter().position(|r| r == "-o").unwrap();
        if args.len() > index+1 {
            params.output = args[index+1].clone();
        }
    }

    params
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde()]
struct LSystem {
    start: String,
    rules: HashMap<char, String>,
    angle: f32,
    iter: u32,
}

#[derive(Debug, Clone)]
struct Segment {
    start: (f32, f32),
    end: (f32, f32),
}

impl LSystem {
    pub fn build_random_lsystem(&mut self) {
        let mut rand_gen: rand::rngs::ThreadRng = rand::thread_rng();

        let angles = [36.0, 45.0, 60.0, 90.0, 120.0];
        self.iter = rand_gen.gen_range(4..=8);
        self.angle = angles[rand_gen.gen_range(0..angles.len())];
        let alphabet: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().collect();
        let mut control_char_set: Vec<char> = "+-]".chars().collect();
        let mut char_set = Vec::<char>::new();
        
        let char_set_length = rand_gen.gen_range(0..8);
        char_set.push('F');
        for _ in 0..char_set_length {
            char_set.push(alphabet[rand_gen.gen_range(0..alphabet.len())]);
        }

        let start_length = rand_gen.gen_range(0..4);
        for _ in 0..=start_length {
            self.start += &char_set[rand_gen.gen_range(0..char_set.len())].to_string();
        }

        let mut rules_char_set = char_set.clone();
        rules_char_set.append(&mut control_char_set);
        for char in char_set {
            let rules_length = rand_gen.gen_range(0..=10);
            if rules_length > 0 {
                let mut rule = "".to_string();
                for _ in 0..=rules_length {
                    let rule_char = rules_char_set[rand_gen.gen_range(0..rules_char_set.len())];
                    rule += &rule_char.to_string();

                    if rule_char == ']' {
                        let position = rand_gen.gen_range(0..rule.len());
                        rule.insert(position, '[');
                    }
                }
                self.rules.insert(char, rule.clone());
            }
        }

        // Trash L-System that draw nothing
        if !self.build_render_string().contains(&alphabet[..]) {
            self.build_random_lsystem();
            return;
        }
    }

    pub fn build_render_string(&self) -> String {
        let mut render_string = self.clone().start;
        for _ in 0..self.clone().iter {
            let mut temp = "".to_string();
            for char in render_string.chars() {
                let rules = self.rules.get(&char);
                if rules.is_some() {
                    temp += rules.unwrap();
                } else {
                    temp += &char.to_string();
                }
            }
            render_string = temp;
        }

        render_string
    }
}


fn rendering_lsystem(lsystem: LSystem, output: String) {
    let draw_variable = "ABCDEFGHIJKLMNOPQRSTUVWZ";
    let move_variable = "abcdefghijklmnopqrstuvwz";
    let starting_point = (0.0, 0.0);
    let segment_length = 10.0;

    println!("Building rendering string...");
    let render_string = lsystem.build_render_string();

    let mut current_point = starting_point.clone();
    let mut angle = 180.0;
    let mut stack = Vec::<((f32, f32), f32)>::new();
    
    let mut segments = Vec::<Segment>::new();

    print!("Rendering...     ");
    // Get all the segment of the rendering (actual rendering is done later)
    for char in render_string.chars() {
        let next_point: Option<(f32, f32)>;
        
        match char {
            '+' => {next_point = None; angle -= lsystem.angle},
            '-' => {next_point = None; angle += lsystem.angle},
            '[' => {next_point = None; stack.push((current_point, angle));},
            ']' => {next_point = None; (current_point, angle) = stack.pop().unwrap();},
            _ => {
                if draw_variable.contains(char) {
                    next_point = Some((
                        current_point.0 + ((angle*(PI/180.0)).sin() * segment_length), 
                        current_point.1 + ((angle*(PI/180.0)).cos() * segment_length)
                    ))
                } else if move_variable.contains(char) {
                    current_point = (
                        current_point.0 + ((angle*(PI/180.0)).sin() * segment_length), 
                        current_point.1 + ((angle*(PI/180.0)).cos() * segment_length)
                    );
                    next_point = None;
                } else {
                    next_point = None;
                }
            },
        }

        if next_point.is_some() {
            segments.push(Segment{start: current_point, end: next_point.unwrap()});
            current_point = next_point.unwrap().clone();
        }
    }

    // Getting the bound of the image
    let min_x = segments.iter().map(|s| f32::min(s.start.0, s.end.0)).into_iter().fold(0.0f32, |min_val, val| val.min(min_val));
    let min_y = segments.iter().map(|s| f32::min(s.start.1, s.end.1)).into_iter().fold(0.0f32, |min_val, val| val.min(min_val));
    let max_x = segments.iter().map(|s| f32::max(s.start.0, s.end.0)).into_iter().fold(0.0f32, |max_val, val| val.max(max_val));
    let max_y = segments.iter().map(|s| f32::max(s.start.1, s.end.1)).into_iter().fold(0.0f32, |max_val, val| val.max(max_val));
    let upleft_point = (min_x - 50.0, min_y - 50.0);
    let downright_point = (max_x + 50.0, max_y + 50.0);

    let width = (downright_point.0 - upleft_point.0) as u32;
    let height = (downright_point.1 - upleft_point.1) as u32;
    if width > 3000 || height > 3000 {
        println!("\nKill switch, remove one iteration");
        let mut lsystem_temp = lsystem.clone();
        lsystem_temp.iter -= 1;
        rendering_lsystem(lsystem_temp, output);
        return();
    }

    // Get the drawing starting point to center the figure in the image (translation by up left point)
    let start_point = ((min_x - starting_point.0).abs() + 50.0, (min_y - starting_point.1).abs() + 50.0);

    //Render the actual image
    let mut buffer = RgbaImage::from_pixel(width, height, Rgba([0 as u8, 0 as u8, 0 as u8, 0]));
    let color = Rgba([128 as u8, 128 as u8, 128 as u8, 255]);
    let mut index = 1;
    let segment_count = segments.len();
    for segment in segments {
        print!("\rRendering... {:.1} %  ", (index as f32 / segment_count as f32)*100.0);
        index += 1;
        buffer = draw_antialiased_line_segment(
            &buffer, 
            (segment.start.0 as i32 + start_point.0 as i32, segment.start.1 as i32 + start_point.1 as i32), 
            (segment.end.0 as i32 + start_point.0 as i32, segment.end.1 as i32 + start_point.1 as i32), 
            color, 
            |_, _, _| Rgba([128 as u8, 128 as u8, 128 as u8, 255])
        );
    }

    //Add the LSystem in the image (not working well, so I comment it)
    // let font_data: &[u8] = include_bytes!("../fonts/DejaVuSansMono.ttf");
    // let font: Font<'static> = Font::try_from_bytes(font_data).unwrap();
    // buffer = draw_text(
    //     &buffer,
    //     Rgba([128,128,128,255]), 
    //     10, 
    //     10, 
    //     Scale{x: height as f32/75.0, y: width as f32/75.0}, 
    //     &font, 
    //     &serde_json::to_string(&lsystem).unwrap()
    // );

    let _ = buffer.save(output);
    println!("\rRendering done         \n");
}

fn main() {
    let params = get_arguments();

    let mut lsystem: LSystem = LSystem::default();
    
    if params.file != "" {
        let mut file = File::open(params.file).unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();
        lsystem = serde_json::from_str(&data).expect("JSON");
    } else if params.json != "" {
        let data = params.json;
        lsystem = serde_json::from_str(&data).expect("JSON");
    } else {
        lsystem.build_random_lsystem();
    }

    let lsystem_json = serde_json::to_string(&lsystem).unwrap();
    println!("{}", lsystem_json);
    
    rendering_lsystem(lsystem, params.output);
}

#[test]
fn test_string_building() {
    let mut lsystem = LSystem{
        start: "F".to_string(), 
        rules: [('F', "F+F-F-F+F".to_string())].iter().cloned().collect(),
        angle: 90.0,
        iter: 2,
    };

    assert!(lsystem.build_render_string() == "F+F-F-F+F+F+F-F-F+F-F+F-F-F+F-F+F-F-F+F+F+F-F-F+F");

    lsystem.iter = 3;

    assert!(lsystem.build_render_string() == "F+F-F-F+F+F+F-F-F+F-F+F-F-F+F-F+F-F-F+F+F+F-F-F+F+F+F-F-F+F+F+F-F-F+F-F+F-F-F+F-F+F-F-F+F+F+F-F-F+F-F+F-F-F+F+F+F-F-F+F-F+F-F-F+F-F+F-F-F+F+F+F-F-F+F-F+F-F-F+F+F+F-F-F+F-F+F-F-F+F-F+F-F-F+F+F+F-F-F+F+F+F-F-F+F+F+F-F-F+F-F+F-F-F+F-F+F-F-F+F+F+F-F-F+F");
}