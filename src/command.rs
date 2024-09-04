/*
根据读取出来的配置文件，生成命令行语句
*/

use crate::config::{Mode, Project};
use ansi_rgb::{cyan_blue, red, Background};
use std::{fs, io, process::Command,process::Stdio};

struct OneLineCommand {
    meta_data: String,
    bin: String,
    args: Vec<String>,
}

impl OneLineCommand {
    fn new(data: String) -> OneLineCommand {
        //分解成为程序和参数
        let words: Vec<String> = data
            .split_ascii_whitespace()
            .map(|w| w.to_string())
            .collect();
        return OneLineCommand {
            meta_data: data,
            bin: words[0].clone(),
            args: words[1..].to_vec(),
        };
    }
    //执行命令
    fn execute(&self) {
        Command::new(&self.bin)
            .args(self.args.clone())
            //     .output()
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect(
                format!("Excuting {} Failed", self.meta_data)
                    .bg(red())
                    .to_string()
                    .as_str(),
            );
        // // 将输出转换为字符串并打印
        // let result = String::from_utf8_lossy(&output.stdout);
        // println!("{}", result.bg(green()));
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
        match fs::metadata(current_path.clone()) {
            Ok(metadata) => {
                if metadata.is_file() {
                    panic!("A file named .sm!");
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
                // 目录不存在，创建它
                fs::create_dir(current_path.clone()).unwrap();
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
                    let mut cmd = format!(
                        "{} -std=c++{} -O{} -c {} -o {}",
                        project.complier.cxx,
                        project.complier.std,
                        project.complier.ol,
                        src_file.to_str().unwrap(),
                        current_path.to_str().unwrap(),
                    );
                    //判断是否添加-Wall参数
                    if project.complier.wall {
                        cmd.push_str(" -Wall ");
                    }
                    //添加额外参数
                    cmd.push_str(project.complier.extra.join(" ").as_str());
                    //存入
                    all_command.cmds.push(OneLineCommand::new(cmd));
                    current_path.pop();
                    current_path.pop();
                }
                //2.收集所有目标文件
                current_path.push(".sm");
                let mut obj_files: Vec<String> = Vec::new();
                for entry in fs::read_dir(current_path.clone()).unwrap() {
                    //如果是文件
                    match entry {
                        Ok(en) => {
                            if en.path().is_file() {
                                //记录文件路径
                                let obj_file = en.path();
                                //这里日后要修改，毕竟要支持多系统
                                if obj_file.extension().unwrap() == "o" {
                                    obj_files.push(obj_file.to_str().unwrap().to_string());
                                }
                            }
                        }
                        _ => {}
                    }
                }
                current_path.pop();
                current_path.push(project.target.lib.clone());
                //3.打包成静态库
                let mut ar_cmd = format!(
                    "ar rcs {}/lib{}.a ",
                    current_path.to_str().unwrap(),
                    project.target.name
                );
                for obj_file in obj_files {
                    ar_cmd.push_str(&obj_file);
                    ar_cmd.push(' ');
                }
                all_command.cmds.push(OneLineCommand::new(ar_cmd));
                current_path.pop();
                current_path.push(project.target.bin.clone());
                //4.编译二进制文件
                let mut complie_cmd = format!(
                    "{} -std=c++{} -O{} {} -o {}/{} -L{} -l{} -I{}",
                    project.complier.cxx,
                    project.complier.std,
                    project.complier.ol,
                    project.target.entrance,
                    current_path.to_str().unwrap().to_string(),
                    project.target.name,
                    project.target.lib,
                    project.target.name,
                    project.target.inc,
                );
                //判断是否添加-Wall参数
                if project.complier.wall {
                    complie_cmd.push_str(" -Wall ");
                }
                //链接系统的静态库
                for l in project.complier.link.clone() {
                    complie_cmd.push_str(format!(" -l{} ", l).as_str());
                }
                //添加额外的参数
                complie_cmd.push_str(project.complier.extra.join(" ").as_str());
                all_command.cmds.push(OneLineCommand::new(complie_cmd));
            }
            Mode::Dynamic => {}
            Mode::Invalid => {
                panic!("Unsupported mode!");
            }
        }
        all_command
    }
    pub fn run(&self) {
        let length = self.cmds.len();
        //迭代所有命令
        for (index, cmd) in self.cmds.iter().enumerate() {
            //打印命令头部
            //优化打印效果
            let header = format!("[{}/{}]", index, length);
            println!("{}: {}", header.bg(cyan_blue()), cmd.meta_data.clone());
            //执行
            cmd.execute();
        }
    }
}
