// Source: https://stackoverflow.com/a/50334049/4933158
// There are whole crates that do this as well, but this seems more lightweight

#[macro_export]
macro_rules! enum_str {
    ($( #[$enum_attr:meta] )* $v: vis enum $name:ident {
        $(
            $( #[$cfgs:meta] )*
            $variant:ident = $val:expr
        ),*,
    }) => {
        $(#[$enum_attr]),*
        $v enum $name {
            $($variant = $val),*
        }

        impl $name {
            $v fn to_string(&self) -> &'static str {
                match self {
                    $($name::$variant => stringify!($variant)),*
                }
            }
        }
    };
}
