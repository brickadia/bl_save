/// A single brick in a save file, including extended attributes.
#[derive(Debug, Clone)]
pub struct Brick<S = String> {
	/// Basic brick data excluding extended attributes.
	pub base: BrickBase<S>,
	/// Extra brick data associated with this brick but not supported by the
	/// library.
	pub unknown_extra: Vec<S>,
}

/// Basic brick data excluding extended attributes such as owner, events, etc.
#[derive(Debug, Clone)]
pub struct BrickBase<S = String> {
	/// The `uiName` of the `fxDTSBrickData` datablock used by the brick.
	pub ui_name: S,
	/// The position of the brick.
	pub position: (f32, f32, f32),
	/// The rotation of the brick.
	/// Valid values range from `0` through `3`.
	pub angle: u8,
	/// Whether the `fxDTSBrickData` datablock is a baseplate.
	pub is_baseplate: bool,
	/// Index into the colorset.
	/// Valid values range from `0` through `63`.
	pub color_index: u8,
	/// Name of the print to use for print bricks. "" represents none.
	pub print: S,
	/// Color effect (such as glow, rainbow).
	pub color_fx: u8,
	/// Shape effect (such as undulo, water).
	pub shape_fx: u8,
	/// Whether the brick can be raycasted against.
	pub raycasting: bool,
	/// Whether objects collide with the brick.
	pub collision: bool,
	/// Whether the brick is visible.
	pub rendering: bool,
}
