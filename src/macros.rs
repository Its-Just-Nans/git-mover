//! This module contains the macros used in the project.

/// automatically generate the input path
macro_rules! config_value {
    ($config:ident, $setting_name:ident, $struct_name:ident, $key_name:ident, $string:expr) => {
        match &$config.config_data.$setting_name {
            Some($struct_name {
                $key_name: Some(value),
                ..
            }) => value.clone(),
            _ => {
                println!(concat!("Please enter ", $string, ":"));
                let value = $crate::utils::input();
                let cloned_value = value.clone();
                $config.update(|config_data| {
                    if let Some(local_config) = config_data.$setting_name.as_mut() {
                        local_config.$key_name = Some(cloned_value);
                    } else {
                        config_data.$setting_name = Some($struct_name {
                            $key_name: Some(cloned_value),
                            ..Default::default()
                        });
                    }
                    config_data
                });
                value
            }
        }
    };
}

/// automatically generate the input path by wrapping config_value macro
macro_rules! config_value_wrap {
    ($config:ident, $setting_name:ident, $struct_name:ident, $key_name:ident, $string:expr) => {
        match &$config.config_data.$setting_name {
            Some(c) => match &c.$key_name {
                Some(u) => u.clone(),
                None => {
                    $crate::config_value!($config, $setting_name, $struct_name, $key_name, $string)
                }
            },
            None => $crate::config_value!($config, $setting_name, $struct_name, $key_name, $string),
        }
    };
}

pub(crate) use config_value;
pub(crate) use config_value_wrap;
