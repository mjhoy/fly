use crate::error::Error;
use std::io::Read;

#[derive(Debug)]
pub struct Migration {
    pub path: String,
    pub identifier: String,
}

impl Migration {
    pub fn up_down(&self) -> Result<(String, String), Error> {
        let mut file = std::fs::File::open(&self.path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let mut statements = contents.split('\n');
        let mut up = String::new();
        let mut down = String::new();
        for line in &mut statements {
            if line == "-- up" {
                break;
            }
        }
        for line in &mut statements {
            if line == "-- down" {
                break;
            }
            up.push_str(line);
            up.push('\n');
        }
        for line in &mut statements {
            down.push_str(line);
            down.push('\n');
        }
        Ok((up, down))
    }
}
