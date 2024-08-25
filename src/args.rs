/*
主要存放与命令行参数相关的函数，因为程序比较简单，
就直接用标准库里的东西了，没有引用额外的包
*/

use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
//终端彩色输出
use ansi_rgb::{ red,green,Background };


//定义project.toml标准模板
const  PROJECT_TXT:&str=r#"
[target]
name="demo"
#Include file directory
inc="./inc"
#Source file 
src="./src"
#Cpp file that contained main function
main="main.cpp"
#Supported Library type:static(sta),dynamic(dyn),null
type="null"
#Library output directory
lib="./lib"
#Execute output directory
bin="./bin"

[complier]
#Supported C++ complier:g++ clang++
cxx="g++"
#Supported C++ standard:98 11 14 17 20
std=11
#enable Wall mode
wall=true
#Setting Optimization Level(0,1,2,3)
ol=1
#Linking extra system library(e.g. liba+libb)
link=""
#Adding more arguments
extra=""
"#;


//读取命令行，并根据输入依次调用相应的子函数
pub fn read_console_input(){
    //收集命令行参数
    let args: Vec<String> = env::args().collect();
    //如果为空，直接打印帮助信息
    if args.len() < 2{
        print_help_infomation();
        return;
    }
    let command=&args[1];
    //判断命令参数
    match command.as_str() {
        "new"=>{
            //需要判断是否有第二个参数，即项目名
            if args.len()<3 {
                println!("{}","Need project name!".bg(red()));
                return;
            }
            let project_name=&args[2];
            //调用新建项目函数
            create_new_project(project_name);
        },
        "init"=>{init_existed_project();},
        "help"=>{print_help_infomation();},
        _=>{
            println!("{}","Unspported arguments!".bg(red()));
            print_help_infomation();
        },
    }
}


//打印帮助信息
fn print_help_infomation(){
    //定义帮助信息
    let help_infomation=r#"
    sm new [project_name]   Creating a brand new project.
    sm init                 Initializing a existed project.
    sm build                Building the project.
    sm run                  Building it, and running it.
    sm clean                Clean up the project(deleting the bin and lib).
    sm help                 Printing the help infomation.
    "#;
    println!("{}",help_infomation);
}

//新建一个项目
fn create_new_project(project_name:&String){
    //按道理是要先检查项目名是否正确的，但是先不管了
    //首先获取当前目录
    let mut current_path=env::current_dir().expect("Can't get current dir!");
    //拼接
    current_path.push(project_name);
    match fs::create_dir(&current_path) {
        Ok(_)=>{
            println!("Creating {} successfully, adding more directory...",current_path.clone().as_os_str().to_str().unwrap().bg(green()));
            //创建include文件夹
            current_path.push("inc");
            fs::create_dir(&current_path).unwrap();
            println!("Creating {} directory successfully.","include".bg(green()));
            //回退
            current_path.pop();
            //创建source文件夹
            current_path.push("src");
            fs::create_dir(&current_path).unwrap();
            println!("Creating {} directory successfully.","source".bg(green()));
            //回退
            current_path.pop();
            //写入project.toml文件
            current_path.push("project.toml");
            let mut file=File::create(current_path).unwrap();
            file.write_all(PROJECT_TXT.as_bytes()).unwrap();
            println!("Creating {} successfully.","project.toml".bg(green()));
            println!("{}","All done, enjoying yourself!");
        },
        Err(e)=>{
            println!("Error:{}",e);
        }
    }
}

//初始化一个已经存在的项目
fn init_existed_project(){
    //首先获取当前目录
    let mut current_path=env::current_dir().expect("Can't get current dir!");
    //拼接
    current_path.push("project.toml");
    //写入
    let mut file=File::create(current_path).unwrap();
    file.write_all(PROJECT_TXT.as_bytes()).unwrap();
    println!("Creating {} successfully.","project.toml".bg(green()));
}