mod alignment;
mod factory;

use alignment::get_aglenv_file_info;
use factory::factory_contains;

#[cxx::bridge(namespace = "oead::sarc")]
mod ffi {
    extern "Rust" {
        #[cxx_name = "FactoryContains"]
        fn factory_contains(name: &str) -> bool;

        #[cxx_name = "GetAglEnvFileInfo"]
        fn get_aglenv_file_info() -> &'static str;
    }

    unsafe extern "C++" {
        include!("roead/src/include/oead/sarc.h");
    }
}
