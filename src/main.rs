use image::Rgb;
use imageproc::{self, definitions::Image, drawing::draw_line_segment};
use serde::{Serialize, Deserialize};
use std::{collections::HashMap, f32::consts::PI, fs::File, io::Read};

struct Parameters {
    file: String,
    json: String,
}

fn get_arguments() -> Parameters {
    let args: Vec<_> = std::env::args().collect();
    let mut params = Parameters {
        file: "".to_string(),
        json: "".to_string(),
    };

    if args.len() < 2 {
        return params;
    }

    if args[1] == "-f" {
        params.file = args[2].clone();
    } else if args[2] == "-j" {
        params.json = args[2].clone();
    }

    params
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde()]
struct LSystem {
    start: String,
    rules: HashMap<char, String>,
    angle: f32,
    iter: u32,
}

impl LSystem {
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

fn main() {
    let draw_variable = "ABCDEFGHIJKLMNOPQRSTUVWZ";
    let move_variable = "abcdefghijklmnopqrstuvwz";
    let width=500;
    let height=500;
    let starting_point = (250.0,250.0);
    let segment_length = 15.0;
    let params = get_arguments();

    let lsystem: LSystem;
    
    if params.file != "" {
        let mut file = File::open(params.file).unwrap();
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();
        lsystem = serde_json::from_str(&data).expect("JSON");
    } else if params.json != "" {
        let data = params.json;
        lsystem = serde_json::from_str(&data).expect("JSON");
    } else {
        lsystem = LSystem {
            start: "F".to_string(),
            rules: [('F', "F+F-F-F+F".to_string())].iter().cloned().collect(),
            angle: 90.0,
            iter: 3,
        }
    }
    

    let render_string = lsystem.build_render_string();

    let mut buffer = Image::new(width, height);
    let color = Rgb([255 as u8, 255 as u8, 255 as u8]);

    let mut current_point = starting_point.clone();
    let mut angle = lsystem.angle;
    let mut stack = Vec::<((f32, f32), f32)>::new();
    
    let mut char_index = 1;
    let render_string_length = render_string.len();
    print!("Rendering...     ");
    for char in render_string.chars() {
        print!("\rRendering... {:.1} %  ", (char_index as f32 / render_string_length as f32)*100.0);
        char_index += 1;

        let next_point: Option<(f32, f32)>;
        
        match char {
            '+' => {next_point = None; angle += lsystem.angle},
            '-' => {next_point = None; angle -= lsystem.angle},
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
            buffer = draw_line_segment(&buffer, current_point,next_point.unwrap(), color);
            current_point = next_point.unwrap().clone();
        }
    }

    let _ = buffer.save("./render/test.png");
    println!("\rRendering done         ");
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