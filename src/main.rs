mod arguments;
mod commands;
mod helper;
mod logger;

use crate::arguments::Commands;
use crate::commands::dev::DevCommand;
use crate::helper::Helper;
use crate::logger::Logger;

/// Wasmcloud is self healing when an application is deployed.
/// The quickest way to run dev mode is to rebuild the component
/// The stop the actor corresponding to that component

fn main() {
   let arguments = arguments::Arguments::get_arguments();

   if !Helper::does_wash_cli_exist() {
      Logger::error_and_exit("Please install wash cli to use this tool".into());
   }

   match &arguments.command {
      Commands::Dev(args) => {
         let config = Helper::get_manifest_from_wadm_config(&args.config);
         Logger::info(format!("Starting dev mode for {:?} \n\n", args.config).into());
         match config {
            Ok(manifest) => {
               let mut dev = DevCommand::new(manifest, args);
               dev.start();
            }
            Err(error) => {
               Logger::error_and_exit(error.to_string());
            }
         }


      }
   }
}
