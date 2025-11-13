// Example plugin for Kandil Code
// This would be compiled as a separate binary

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Hello from example plugin!");
        return;
    }
    
    let command = &args[1];
    match command.as_str() {
        "hello" => println!("Hello from plugin!"),
        "version" => println!("Plugin version: 1.0.0"),
        _ => println!("Unknown command: {}. Available: hello, version", command),
    }
}