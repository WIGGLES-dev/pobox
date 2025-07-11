#[macro_export]
macro_rules! message {
    (
		$(#[$enum_attr:meta])*
		$vis:vis
		$ident:ident
		<$generic_channel:ident: Output $(, $($generic:ty),* $(,)?)?>
		$(where $($where_clause:tt)+)?
		{
			$(#[$variant_attr:meta])*
			$($variant:ident $({ $( $arg:ident: $arg_ty:ty),* $(,)? })? -> $output_ty:ty),* $(,)?
		}
	) => {
		$( #[ $enum_attr ] )*
		$vis enum $ident<$generic_channel $(,$generic:ty)*> $($($generic),*)?
		where
		$generic_channel: ${concat($ident, Output)},
		$($($where_clause)+)? {
			$( #[ $variant_attr ] )*
			$( $variant { $( $( $arg: $arg_ty ),* ,)?  output: $generic_channel::$variant },)*
		}

		$(
			$vis struct ${concat($variant, Output)}(pub $output_ty);
		)*
		$vis trait ${concat($ident, Output)} {
			$( type $variant: ::pobox::Sender<${concat($variant, Output)}>; )*
		}
	};
	(@variant $ident:ident ()) => {};
	(@variant $ident:ident {}) => {};
	(@variant $ident:ident) => {};
}
