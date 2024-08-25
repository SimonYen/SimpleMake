/*
根据读取出来的配置文件，生成命令行语句
*/

use ansi_rgb::{ red,green,Background };
use std::process::Command;
use crate::config::{Mode, Project};

struct OneLineCommand{
    data:String,
}

impl OneLineCommand {
    fn new(data:String)->OneLineCommand{
        return OneLineCommand{data:data};
    }
    //执行命令
    fn execute(& self){
        let output = Command::new(&self.data)
        .output()
        .expect(format!("Failed to execute {}",&self.data).bg(red()).to_string().as_str());
        // 将输出转换为字符串并打印
        let result = String::from_utf8_lossy(&output.stdout);
        println!("{}", result.bg(green()));
    }
}

//所有需要执行的命令
pub struct AllCommand{
    cmds:Vec<OneLineCommand>,
}

impl AllCommand {
    pub fn new(project:&Project)->AllCommand{
        //这个构造函数实现很重要，需要慢慢写
        let mut all_command=AllCommand{cmds:Vec::<OneLineCommand>::new()};
        match project.get_mode() {
            //编译为静态库时
            Mode::Static=>{
                
            }
        }
        all_command
    }
}