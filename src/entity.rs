use std::{
	any::{Any, TypeId},
	collections::HashMap,
};

use rustc_hash::FxHashMap;
use stardust_xr_fusion::{
	client::{Client, ClientHandler, FrameInfo},
	fields::{Field, FieldExt, FieldRef, Shape},
	query::{QueryableExt, QueryableObject},
	spatial::{Spatial, SpatialExt, SpatialRef, Transform},
};

// TODO: would this be better with refs internally?
pub struct ComponentContext {
	pub queryable: QueryableObject,
	pub spatial: SpatialRef,
	/// all spatial information exposed by components has to be in this space
	pub reference_space: SpatialRef,
	pub field: FieldRef,
	// pub field_shape: Shape,
}
pub trait Component: ComponentDyn + Sized {
	type Args: Send;
	type Err: Send;
	fn new_component(
		ctx: &ComponentContext,
		client: &Client<impl ClientHandler>,
		args: impl Into<Self::Args> + Send,
	) -> impl Future<Output = Result<Self, Self::Err>> + Send;
	fn handle_events(&mut self) -> bool;
	fn frame(&mut self, info: &FrameInfo);
}
impl<T: Component> ComponentDyn for T {
	fn handle_events(&mut self) -> bool {
		Component::handle_events(self)
	}

	fn frame(&mut self, info: &FrameInfo) {
		Component::frame(self, info);
	}
}
pub trait ComponentDyn: Any {
	fn handle_events(&mut self) -> bool;
	fn frame(&mut self, info: &FrameInfo);
}
pub struct Entity {
	components: FxHashMap<TypeId, Box<dyn ComponentDyn>>,
	ctx: ComponentContext,
}
impl Entity {
	pub async fn new(
		client: &Client<impl ClientHandler>,
        spatial: &Spatial,
		reference_space: &SpatialRef,
		field: &Field,
	) -> stardust_xr_fusion::Result<Self> {
        let spatial_ref = spatial.spatial_ref().await?;
		let field_ref = field.field_ref().await?;
		let queryable = QueryableObject::new(client, spatial.clone(), field.clone()).await?;
		let ctx = ComponentContext {
			queryable,
			spatial: spatial_ref,
			reference_space: reference_space.clone(),
			field: field_ref,
		};
		Ok(Self {
			components: HashMap::default(),
			ctx,
		})
	}
	pub fn get<T: Component>(&self) -> Option<&T> {
		self.components
			.get(&TypeId::of::<T>())
			.and_then(|v| (v as &dyn Any).downcast_ref())
	}
	pub fn get_mut<T: Component>(&mut self) -> Option<&mut T> {
		self.components
			.get_mut(&TypeId::of::<T>())
			.and_then(|v| (v as &mut dyn Any).downcast_mut())
	}
	pub async fn add_component<T: Component>(
		&mut self,
		client: &Client<impl ClientHandler>,
		args: T::Args,
	) -> Result<(), T::Err> {
		let comp = T::new_component(&self.ctx, client, args).await?;
		self.components.insert(TypeId::of::<T>(), Box::new(comp));
		Ok(())
	}
}
