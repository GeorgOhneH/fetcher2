use druid::Widget;

mod widgets;
mod ctypes;


impl From<crate::traveller::FileSpec> for druid::FileSpec {
    fn from(file_spec: crate::traveller::FileSpec) -> Self {
        druid::FileSpec {
            name: file_spec.name,
            extensions: file_spec.extensions,
        }
    }
}
