/*
根据读取出来的配置文件，生成命令行语句
*/

use crate::config::{Mode, Project};
use ansi_rgb::{cyan_blue, red, Background};
use duct::cmd;
use std::{fs, io, path::PathBuf};

//有限状态机
#[derive(PartialEq)]
enum State {
    Start,
    Failed,
    Obj,
    Lib,
    Bin,
    End,
}

struct OneLineCommand {
    meta_data: String,
    bin: String,
    args: Vec<String>,
}

impl OneLineCommand {
    fn new(data: String) -> OneLineCommand {
        //分解成为程序和参数
        let words: Vec<_> = data
            .split_ascii_whitespace()
            .map(|w| w.to_string())
            .collect();
        return OneLineCommand {
            meta_data: data,
            bin: words[0].clone(),
            args: words[1..].to_vec(),
        };
    }
    //阻塞执行命令
    fn execute(&self) -> bool {
        match cmd(&self.bin, &self.args).run() {
            Ok(output) => {
                //检测命令是否成功执行
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    //打印出来
                    println!("{}", stdout);
                    true
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!("{}", stderr.bg(red()));
                    false
                }
            }
            Err(e) => {
                eprintln!("Excuting {} Failed: {}", self.meta_data, e);
                false
            }
        }
    }
}

//所有需要执行的命令
pub struct AllCommand {
    //命令需要细分，先编译目标文件，然后再打包成库，最后才是可执行文件
    obj_cmds: Vec<OneLineCommand>,
    lib_cmd: OneLineCommand,
    bin_cmd: OneLineCommand,
    state: State,
    mode: Mode,
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
            obj_cmds: Vec::<OneLineCommand>::new(),
            lib_cmd: OneLineCommand::new("ls".to_string()),
            bin_cmd: OneLineCommand::new("ls".to_string()),
            state: State::Start,
            mode: Mode::Invalid,
        };
        //获取当前路径
        let mut current_path = std::env::current_dir().unwrap();
        //获取所有源文件
        let src_files = project.get_src_files();
        current_path.push(".sm");
        //0.创建必要的文件夹
        mkdir(&current_path);
        current_path.pop();
        current_path.push(&project.target.lib);
        mkdir(&current_path);
        current_path.pop();
        current_path.push(&project.target.bin);
        mkdir(&current_path);
        current_path.pop();
        all_command.mode = project.get_mode();
        match all_command.mode {
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
                    all_command.obj_cmds.push(OneLineCommand::new(cmd));
                    current_path.pop();
                    current_path.pop();
                }
                //2.收集所有目标文件
                current_path.push(".sm");
                let mut obj_files: Vec<String> = Vec::new();
                for entry in fs::read_dir(&current_path).unwrap() {
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
                current_path.push(&project.target.lib);
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
                all_command.lib_cmd = OneLineCommand::new(ar_cmd);
                current_path.pop();
                current_path.push(&project.target.bin);
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
                for l in &project.complier.link {
                    complie_cmd.push_str(format!(" -l{} ", l).as_str());
                }
                //添加额外的参数
                complie_cmd.push_str(project.complier.extra.join(" ").as_str());
                all_command.bin_cmd = OneLineCommand::new(complie_cmd);
            }
            Mode::Dynamic => {
                let srcs: Vec<_> = src_files
                    .iter()
                    .map(|s| s.to_str().unwrap().to_string())
                    .collect();
                let srcs = srcs.join(" ");
                current_path.push(&project.target.lib);
                //1.源代码直接生成动态库文件
                let mut lib_cmd = format!(
                    "{} -shared -fPIC -std=c++{} -O{} {} -o {}/lib{}.so",
                    project.complier.cxx,
                    project.complier.std,
                    project.complier.ol,
                    srcs,
                    current_path.to_str().unwrap().to_string(),
                    project.target.name,
                );
                current_path.pop();
                //判断是否添加-Wall参数
                if project.complier.wall {
                    lib_cmd.push_str(" -Wall ");
                }
                //添加额外的参数
                lib_cmd.push_str(project.complier.extra.join(" ").as_str());
                all_command.lib_cmd = OneLineCommand::new(lib_cmd);
                //2.编译二进制文件
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
                for l in &project.complier.link {
                    complie_cmd.push_str(format!(" -l{} ", l).as_str());
                }
                //添加额外的参数
                complie_cmd.push_str(project.complier.extra.join(" ").as_str());
                all_command.bin_cmd = OneLineCommand::new(complie_cmd);
            }
            Mode::Invalid => {
                panic!("Unsupported mode!");
            }
        }
        all_command
    }
    pub fn run(&mut self) {
        match self.mode {
            Mode::Static => {
                self.sta_run();
            }
            Mode::Dynamic => {
                self.dyn_run();
            }
            Mode::Invalid => {
                panic!("Unsupported mode!");
            }
        }
    }
    fn sta_run(&mut self) {
        let length = self.obj_cmds.len() + 2;
        //FSM，有限状态机
        loop {
            match self.state {
                //初始状态，需要编译源代码为目标文件，切换状态为obj
                State::Start => {
                    self.state = State::Obj;
                }
                //失败状态，输出提示信息，并切换状态为结束状态
                State::Failed => {
                    println!("{}", "Command aborting!".bg(red()));
                    self.state = State::End;
                }
                State::End => {
                    break;
                }
                //obj状态，编译源代码
                State::Obj => {
                    let mut result = true;
                    for (index, cmd) in self.obj_cmds.iter().enumerate() {
                        let header = format!("[{}/{}]", index + 1, length);
                        println!("{}: {}", header.bg(cyan_blue()), &cmd.meta_data);
                        //执行
                        let tmp = cmd.execute();
                        //检查是否执行成功
                        if !tmp {
                            result = false;
                            break;
                        }
                    }
                    if result {
                        //成功执行，没有报错，切换下一个状态
                        self.state = State::Lib;
                    } else {
                        self.state = State::Failed;
                    }
                }
                //lib状态，打包成库
                State::Lib => {
                    let header = format!("[{}/{}]", length - 1, length);
                    println!("{}: {}", header.bg(cyan_blue()), self.lib_cmd.meta_data);
                    //执行
                    if self.lib_cmd.execute() {
                        self.state = State::Bin;
                    } else {
                        self.state = State::Failed;
                    }
                }
                //bin状态，编译成二进制文件
                State::Bin => {
                    let header = format!("[{}/{}]", length, length);
                    println!("{}: {}", header.bg(cyan_blue()), &self.bin_cmd.meta_data);
                    //执行
                    if self.bin_cmd.execute() {
                        self.state = State::End;
                    } else {
                        self.state = State::Failed;
                    }
                }
            }
        }
    }
    fn dyn_run(&mut self) {
        let length = self.obj_cmds.len() + 2;
        //FSM，有限状态机
        loop {
            match self.state {
                //初始状态，需要编译源代码为动态库文件，切换状态为lib
                State::Start => {
                    self.state = State::Lib;
                }
                //失败状态，输出提示信息，并切换状态为结束状态
                State::Failed => {
                    println!("{}", "Command aborting!".bg(red()));
                    self.state = State::End;
                }
                //lib状态，打包成库
                State::Lib => {
                    let header = format!("[{}/{}]", length - 1, length);
                    println!("{}: {}", header.bg(cyan_blue()), self.lib_cmd.meta_data);
                    //执行
                    if self.lib_cmd.execute() {
                        self.state = State::Bin;
                    } else {
                        self.state = State::Failed;
                    }
                }
                //bin状态，编译成二进制文件
                State::Bin => {
                    let header = format!("[{}/{}]", length, length);
                    println!("{}: {}", header.bg(cyan_blue()), &self.bin_cmd.meta_data);
                    //执行
                    if self.bin_cmd.execute() {
                        self.state = State::End;
                    } else {
                        self.state = State::Failed;
                    }
                }
                _ => {
                    break;
                }
            }
        }
    }
}

//功能性函数，创建文件夹
fn mkdir(p: &PathBuf) {
    match fs::metadata(p) {
        Ok(metadata) => {
            if metadata.is_file() {
                panic!("Naming conflict!");
            }
        }
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            // 目录不存在，创建它
            match fs::create_dir(p) {
                Ok(_) => {
                    println!("Creating {} directory successfully.", p.to_str().unwrap())
                }
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                    println!("Skipping {}", p.to_str().unwrap());
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
        }
        Err(e) => {
            // 处理其他错误
            eprintln!("{}", e);
        }
    }
}
