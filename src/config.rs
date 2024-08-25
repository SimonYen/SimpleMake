/*
主要负责把project.toml里面的内容给读出来
*/
use std::path::PathBuf;
use std::fs::read_to_string;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Project {
    pub target:Target,
    pub complier:Complier,
}

#[derive(Deserialize)]
pub struct Target{
    pub name:String,
    pub inc:String,
    pub src:String,
    pub entrance:String,
    pub mode:String,
    pub lib:String,
    pub bin:String,
}

#[derive(Deserialize)]
pub struct Complier{
    pub cxx:String,
    pub std:i64,
    pub wall:bool,
    pub ol:i8,
    pub link:Vec<String>,
    pub extra:Vec<String>,
}

impl Project {
    pub fn new(config_path:&PathBuf)->Project{
        //读取文件内容
        let content=read_to_string(config_path).unwrap();
        return toml::from_str(&content).unwrap();
    }
}