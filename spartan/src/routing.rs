use crate::node::Node;
use tide::Server;

macro_rules! route {
    ($name:ident) => {
        crate::actions::$name::$name
    };
}

pub fn attach_routes(tide: &mut Server<Node>) {
    tide.at("/:queue").get(route!(pop));
    tide.at("/:queue").post(route!(push));
    tide.at("/:queue").delete(route!(delete));
    tide.at("/:queue/size").get(route!(size));
    tide.at("/:queue/requeue").post(route!(requeue));
    tide.at("/:queue/clear").post(route!(clear));
}
