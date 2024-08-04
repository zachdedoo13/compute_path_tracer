#[macro_export]
macro_rules! defaults_and_sliders_gui {
    ($name:ident, $($field_name:ident: $field_type:ty = $default:expr => $range:expr),*) => {
        #[repr(C)]
        #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
        pub struct $name {
            $(pub $field_name: $field_type,)*
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    $($field_name: $default,)*
                }
            }
        }

        impl $name {
            pub fn ui(&mut self, ui: &mut egui::Ui) {
                egui::CollapsingHeader::new(stringify!($name))
                    .default_open(true)
                    .show(ui, |ui| {
                        if ui.add(egui::Button::new("Reset")).clicked() { *self = Self::default() }

                        $(ui.add(
                            egui::Slider::new(&mut self.$field_name, $range)
                            .text(stringify!($field_name))
                        );)*
                    });
            }
        }
    };
}


#[macro_export]
macro_rules! defaults_and_drag_value_gui {
    ($name:ident, $($field_name:ident: $field_type:ty = $default:expr => $range:expr),*) => {
        #[repr(C)]
        #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
        pub struct $name {
            $(pub $field_name: $field_type,)*
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    $($field_name: $default,)*
                }
            }
        }

        impl $name {
            pub fn ui(&mut self, ui: &mut egui::Ui) {
                egui::CollapsingHeader::new(stringify!($name))
                    .default_open(true)
                    .show(ui, |ui| {
                        if ui.add(egui::Button::new("Reset")).clicked() { *self = Self::default() }

                        $(ui.add(
                            egui::DragValue::new(&mut self.$field_name)
                            .clamp_range($range)
                            .prefix(stringify!($field_name))
                        );)*
                    });
            }
        }
    };
}

#[macro_export]
macro_rules! defaults_only_gui {
    ($name:ident, $($field_name:ident: $field_type:ty = $default:expr),*) => {
        #[repr(C)]
        #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, PartialEq)]
        pub struct $name {
            $(pub $field_name: $field_type,)*
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    $($field_name: $default,)*
                }
            }
        }

       impl $name {
            pub fn ui(&self, ui: &mut egui::Ui) {
                egui::CollapsingHeader::new(stringify!($name))
                    .default_open(false)
                    .show(ui, |ui| {
                        $(
                           ui.add(egui::Label::new(
                              format!("{}, {}", stringify!($field_name), self.$field_name)
                           ));
                        )*
                    });
            }
        }
    };
}


#[macro_export]
macro_rules! if_is_type {
    ($name: ident, $input:expr, $field_type:ty, $code:block) => {
        if let Some($name) = $input.as_any().downcast_ref::<$field_type>() {
            $code
        }
    };
}

#[macro_export]
macro_rules! enum_egui_dropdown {
    ($name: ident, $($option:tt),* ) => {
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        pub enum $name {
            $($option,)*
        }
        impl $name {
            pub fn dropdown(&mut self, ui: &mut Ui) {
                ComboBox::from_label("")
                .selected_text(format!("{:?}", self))
                .show_ui(ui, |ui| {
                    $(ui.selectable_value(self, $name::$option, stringify!($option));)*
                });
            }
        }
    };
}