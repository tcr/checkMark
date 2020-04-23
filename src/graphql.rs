use crate::*;
use juniper::{FieldResult, RootNode, ID};
use lazy_static::*;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

lazy_static! {
    pub static ref PUBLIC_NOTEBOOKS: RwLock<Vec<Notebook>> = RwLock::new(vec![]);
}

juniper::graphql_object!(Page: () |&self| {
    field id() -> ID {
        ID::new(self.id.clone())
    }

    field svg() -> Option<String> {
        self.svg.clone()
    }

    field png() -> Option<String> {
        self.svg.clone().map(|x| x.replace(".svg", ".png"))
    }

    field modified() -> String { // TODO time
        self.modified.clone()
    }
});

juniper::graphql_object!(Notebook: () |&self| {
    field id() -> ID {
        ID::new(self.id.clone())
    }

    field name() -> FieldResult<String> {
        Ok(self.name.to_string())
    }

    field pages() -> FieldResult<Vec<Page>> {
        Ok(self.pages.clone())
    }
});

pub struct Root;

juniper::graphql_object!(Root: () |&self| {
    field viewer_id() -> FieldResult<String> {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
        Ok(format!("{:?}", since_the_epoch))
    }

    field notebooks() -> FieldResult<Vec<Notebook>> {
        Ok((*PUBLIC_NOTEBOOKS.read()?).clone())
    }
});

pub struct Mutations;

juniper::graphql_object!(Mutations: () | &self | {
    field viewer_id() -> FieldResult<String> {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
        Ok(format!("{:?}", since_the_epoch))
    }
});

pub type Schema = RootNode<'static, Root, Mutations>;

pub fn schema() -> Schema {
    Schema::new(Root, Mutations)
}
