use std::env;
/*
主要负责把project.toml里面的内容给读出来
*/
use serde::Deserialize;
use std::fs::{read_dir, read_to_string};
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Project {
    pub target: Target,
    pub complier: Complier,
}

#[derive(Deserialize)]
pub struct Target {
    pub name: String,
    pub inc: String,
    pub src: String,
    pub entrance: String,
    pub mode: String,
    pub lib: String,
    pub bin: String,
}

#[derive(Deserialize)]
pub struct Complier {
    pub cxx: String,
    pub std: i64,
    pub wall: bool,
    pub ol: i8,
    pub link: Vec<String>,
    pub extra: Vec<String>,
}

//编译模式
pub enum Mode {
    Static,
    Dynamic,
    Invalid,
}

impl Project {
    pub fn new(config_path: &PathBuf) -> Project {
        //读取文件内容
        let content = read_to_string(config_path).unwrap();
        return toml::from_str(&content).unwrap();
    }
    //获取编译模式
    pub fn get_mode(&self) -> Mode {
        match self.target.mode.as_str() {
            "sta" => Mode::Static,
            "dyn" => Mode::Dynamic,
            _ => Mode::Invalid,
        }
    }
    //检查c++标准是否正确
    pub fn check_std(&self) -> bool {
        match self.complier.std {
            98 | 11 | 14 | 17 | 20 => true,
            _ => false,
        }
    }
    //检查代码优化等级是否正确
    pub fn check_ol(&self) -> bool {
        match self.complier.ol {
            0..=3 => true,
            _ => false,
        }
    }
    //检查编译器是否设置正确
    pub fn check_complier(&self) -> bool {
        match self.complier.cxx.as_str() {
            "g++" | "clang++" => true,
            _ => false,
        }
    }
    //递归遍历函数
    fn visit_dirs(&self, dir: &PathBuf, src_files: &mut Vec<PathBuf>) {
        //递归退出条件
        if dir.is_file() {
            //收集c++源文件
            match dir.extension() {
                Some(s) => {
                    if s == "cxx" || s == "cpp" || s == "hpp" {
                        //添加到数组
                        src_files.push(dir.to_path_buf())
                    }
                }
                None => {}
            }
            return;
        }
        //递归遍历
        for d in read_dir(dir).expect("Can't read dir!") {
            match d {
                Ok(p) => {
                    self.visit_dirs(&p.path(), src_files);
                }
                _ => {}
            }
        }
    }
    //获取需要编译的源文件数组
    pub fn get_src_files(&self) -> Vec<PathBuf> {
        let mut result: Vec<PathBuf> = Vec::new();
        //获取当前目录
        let mut current_path = env::current_dir().unwrap();
        current_path.push(self.target.src.clone());
        self.visit_dirs(&current_path, &mut result);
        result
    }
}
