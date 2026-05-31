use gluon::{Context, Object};
use stardust_xr_fusion::{
	Result,
	client::{Client, ClientHandler},
	fields::Field,
	query::{QueryExt, QueryableObject},
	spatial::Spatial,
};
use stardust_xr_molecules_protocols::derezzable::{DerezzableHandler, EXTERNAL_PROTOCOL};
use std::any::Any;
use tokio::sync::mpsc;

#[derive(Debug, gluon::Handler)]
struct DerezzableInner(mpsc::Sender<()>);
impl DerezzableHandler for DerezzableInner {
	async fn derez(&self, _ctx: Context) {
		let _ = self.0.send(()).await;
	}
}

pub struct Derezzable {
	pub receiver: mpsc::Receiver<()>,
	_obj: Object<DerezzableInner>,
	_guard: Box<dyn Any + Send + Sync>,
}
impl Derezzable {
	pub async fn new<H: ClientHandler>(
		client: &Client<H>,
		spatial: Spatial,
		field: Field,
	) -> Result<Self> {
		let (tx, rx) = mpsc::channel(8);
		let obj = client.pion_device().register_object(DerezzableInner(tx));

		let queryable = QueryableObject::create(client, spatial, field).await?;
		let guard = queryable
			.add_interface(&obj, EXTERNAL_PROTOCOL.protocol_name)
			.await?;

		Ok(Derezzable {
			receiver: rx,
			_obj: obj,
			_guard: Box::new(guard) as Box<dyn Any + Send + Sync>,
		})
	}
}
