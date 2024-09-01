/*
根据读取出来的配置文件，生成命令行语句
*/

use crate::config::{Mode, Project};
use ansi_rgb::{green, red, Background};
use std::{fs, io, process::Command};

struct OneLineCommand {
    data: String,
}

impl OneLineCommand {
    fn new(data: String) -> OneLineCommand {
        return OneLineCommand { data };
    }
    //执行命令
    fn execute(&self) {
        let output = Command::new(&self.data).output().expect(
            format!("Failed to execute {}", &self.data)
                .bg(red())
                .to_string()
                .as_str(),
        );
        // 将输出转换为字符串并打印
        let result = String::from_utf8_lossy(&output.stdout);
        println!("{}", result.bg(green()));
    }
}

//所有需要执行的命令
pub struct AllCommand {
    cmds: Vec<OneLineCommand>,
}

impl AllCommand {
    pub fn new(project: &Project) -> AllCommand {
        if !project.check_complier() {
            panic!("C++ complier: {} is invaild!", project.complier.cxx);
        }
        if !project.check_ol() {
            panic!("Opt {} is invaild!", project.complier.cxx);
        }
        if !project.check_std() {
            panic!("{} is invaild!", project.complier.cxx);
        }
        //这个构造函数实现很重要，需要慢慢写
        let mut all_command = AllCommand {
            cmds: Vec::<OneLineCommand>::new(),
        };
        //获取当前路径
        let mut current_path = std::env::current_dir().unwrap();
        //获取所有源文件
        let src_files = project.get_src_files();
        current_path.push(".sm");
        //0.创建.sm文件夹
        match fs::metadata(current_path) {
            Ok(metadata) => {
                if metadata.is_file() {
                    panic!("A file named .sm!");
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                // 目录不存在，创建它
                fs::create_dir(current_path).unwrap();
            }
            Err(e) => {
                // 处理其他错误
                eprintln!("{}", e);
            }
        }
        current_path.pop();
        match project.get_mode() {
            //编译为静态库时
            Mode::Static => {
                //1.将所有源文件编译成目标文件
                for (index, src_file) in src_files.iter().enumerate() {
                    //后期实现应该计算文件的md5，看看是否有所改变，不能简单的用序号代表文件
                    current_path.push(".sm");
                    current_path.push(format!("{}.o", index));
                    //类似于这种命令 g++ -std=c++11 -Wall -O2 -c file.cpp -o .sm/file1.o
                    let cmd = format!(
                        "{} -std=c++{} -O{} -c {} -o {}",
                        project.complier.cxx,
                        project.complier.std,
                        project.complier.ol,
                        src_file.to_str().unwrap(),
                        current_path.to_str().unwrap(),
                    );
                    //存入
                    all_command.cmds.push(OneLineCommand::new(cmd));
                    current_path.pop();
                    current_path.pop();
                }
            }
        }
        all_command
    }
}
