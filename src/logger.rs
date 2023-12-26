use std::process::exit;
use colored::*;

pub struct Logger {}

impl Logger {
    pub fn error_and_exit(message: String) {
        println!("{}:", "Error".red().bold());
        println!("{} \n \n", message);
        exit(1);
    }

    pub fn error(message: String) {
        println!("{}:", "Error".red().bold());
        println!("{} \n \n", message);
    }

    pub fn info(message: String) {
        println!("{}:", "Info".bold());
        println!("{} \n", message);
    }
}